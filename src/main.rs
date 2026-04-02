//daemon lifecycle, cli dispatch, signal handling
mod audio;
mod keyboard;
mod mapping;
mod voice;

use audio::AudioEngine;
use crossbeam_channel::bounded;
use dialoguer::{theme::SimpleTheme, Select};
use keyboard::start_keyboard_listener;
use mapping::{KeyMapper, ScaleMode};
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[cfg(target_os = "macos")]
use core_foundation::base::TCFType;
#[cfg(target_os = "macos")]
use core_foundation::boolean::CFBoolean;
#[cfg(target_os = "macos")]
use core_foundation::dictionary::CFDictionary;
#[cfg(target_os = "macos")]
use core_foundation::string::CFString;

const EVENT_QUEUE_SIZE: usize = 256;

#[cfg(target_os = "macos")]
extern "C" {
    fn AXIsProcessTrustedWithOptions(options: core_foundation::dictionary::CFDictionaryRef)
        -> bool;
    fn AXIsProcessTrusted() -> bool;
}

#[cfg(target_os = "macos")]
fn request_accessibility_permission() -> bool {
    let key = CFString::new("AXTrustedCheckOptionPrompt");
    let value = CFBoolean::true_value();
    let options = CFDictionary::from_CFType_pairs(&[(key.as_CFType(), value.as_CFType())]);
    unsafe { AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef()) }
}

#[cfg(target_os = "macos")]
fn has_accessibility_permission() -> bool {
    unsafe { AXIsProcessTrusted() }
}

fn pid_file_path() -> PathBuf {
    PathBuf::from("/tmp/keystream.pid")
}

fn mode_file_path() -> PathBuf {
    PathBuf::from("/tmp/keystream.mode")
}

fn read_pid() -> Option<u32> {
    let mut file = fs::File::open(pid_file_path()).ok()?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).ok()?;
    contents.trim().parse().ok()
}

//write to temp file then rename to prevent readers seeing partial content
fn write_pid(pid: u32) {
    let path = pid_file_path();
    let tmp = path.with_extension("tmp");
    if let Ok(mut file) = fs::File::create(&tmp) {
        if write!(file, "{}", pid).is_ok() {
            let _ = fs::rename(&tmp, &path);
            return;
        }
    }
    let _ = fs::remove_file(&tmp);
}

fn remove_pid() {
    let _ = fs::remove_file(pid_file_path());
    let _ = fs::remove_file(mode_file_path());
}

fn read_mode() -> Option<String> {
    let mut file = fs::File::open(mode_file_path()).ok()?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).ok()?;
    Some(contents.trim().to_string())
}

fn write_mode(mode: &str) {
    if let Ok(mut file) = fs::File::create(mode_file_path()) {
        let _ = write!(file, "{}", mode);
    }
}

fn is_process_running(pid: u32) -> bool {
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

fn pause() {
    thread::sleep(Duration::from_millis(300));
}

fn mode_name(mode: ScaleMode) -> &'static str {
    match mode {
        ScaleMode::Pentatonic => "pentatonic",
        ScaleMode::Lydian => "lydian",
    }
}

fn mode_scale(mode: ScaleMode) -> &'static str {
    match mode {
        ScaleMode::Pentatonic => "C D E G A",
        ScaleMode::Lydian => "C D E F# G A B",
    }
}

fn print_header() {
    println!();
    println!("KEYSTREAM 0.1");
    println!("--------------");
    println!();
}

fn print_usage() {
    print_header();
    println!("usage: keystream <command> [mode]");
    println!();
    println!("commands:");
    println!("  start [mode]  initiate daemon");
    println!("  stop          terminate daemon");
    println!("  status        query state");
    println!("  run [mode]    foreground mode");
    println!();
    println!("modes:");
    println!("  pentatonic    C D E G A (default)");
    println!("  lydian        C D E F# G A B");
    println!();
}

fn cmd_start(mode: ScaleMode, header_shown: bool) {
    if let Some(pid) = read_pid() {
        if is_process_running(pid) {
            if !header_shown {
                print_header();
            }
            println!("daemon already running");
            println!("pid       : {}", pid);
            println!();
            return;
        }
    }

    let exe = env::current_exe().expect("failed to get executable path");

    let child = Command::new(&exe)
        .arg("--daemon")
        .arg(mode_name(mode))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    match child {
        Ok(child) => {
            let pid = child.id();
            if !header_shown {
                print_header();
            }
            println!("mode      : {}", mode_name(mode));
            println!("scale     : {}", mode_scale(mode));
            println!("voices    : 32");
            println!();
            print!("starting daemon...");
            std::io::stdout().flush().ok();
            pause();
            println!();
            print!("attaching keyboard stream...");
            std::io::stdout().flush().ok();
            pause();
            println!();
            print!("opening audio device...");
            std::io::stdout().flush().ok();
            pause();
            println!();
            println!();
            println!("ONLINE");
            println!();
            write_mode(mode_name(mode));
            let _ = pid;
        }
        Err(e) => {
            if !header_shown {
                print_header();
            }
            println!("failed to start daemon: {}", e);
            println!();
        }
    }
}

fn cmd_stop() {
    print_header();
    match read_pid() {
        Some(pid) => {
            if is_process_running(pid) {
                print!("sending termination signal...");
                std::io::stdout().flush().ok();
                unsafe {
                    libc::kill(pid as i32, libc::SIGTERM);
                }
                pause();
                println!();
                print!("waiting for daemon...");
                std::io::stdout().flush().ok();
                pause();
                println!();
                remove_pid();
                println!();
                println!("OFFLINE");
                println!();
            } else {
                remove_pid();
                println!("daemon not running");
                println!();
            }
        }
        None => {
            println!("daemon not running");
            println!();
        }
    }
}

fn cmd_status() {
    print_header();
    match read_pid() {
        Some(pid) => {
            if is_process_running(pid) {
                let mode = read_mode().unwrap_or_else(|| "pentatonic".to_string());
                let scale = if mode == "lydian" {
                    "C D E F# G A B"
                } else {
                    "C D E G A"
                };
                println!("daemon    : online");
                println!("mode      : {}", mode);
                println!("scale     : {}", scale);
                println!("voices    : 32");
                println!("pid       : {}", pid);
            } else {
                remove_pid();
                println!("daemon    : offline");
            }
        }
        None => {
            println!("daemon    : offline");
        }
    }
    println!();
}

fn cmd_run(mode: ScaleMode, header_shown: bool) {
    #[cfg(target_os = "macos")]
    if !has_accessibility_permission() {
        if !header_shown {
            print_header();
        }
        println!("requesting accessibility permission...");
        request_accessibility_permission();
        println!("grant permission and restart");
        println!();
        std::process::exit(1);
    }

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("failed to set signal handler");

    let (sender, receiver) = bounded(EVENT_QUEUE_SIZE);

    let audio = match AudioEngine::new(receiver) {
        Ok(engine) => engine,
        Err(e) => {
            if !header_shown {
                print_header();
            }
            println!("audio initialisation failed: {}", e);
            println!();
            return;
        }
    };

    let mapper = Arc::new(KeyMapper::new(mode));

    let _keyboard = match start_keyboard_listener(sender, mapper) {
        Some(listener) => listener,
        None => {
            if !header_shown {
                print_header();
            }
            println!("keyboard initialisation failed");
            println!();
            return;
        }
    };

    if !header_shown {
        print_header();
    }
    println!("mode      : {}", mode_name(mode));
    println!("scale     : {}", mode_scale(mode));
    println!("voices    : 32");
    println!();
    print!("attaching keyboard stream...");
    std::io::stdout().flush().ok();
    pause();
    println!();
    print!("opening audio device...");
    std::io::stdout().flush().ok();
    pause();
    println!();
    println!();
    println!("READY");
    println!();
    println!("terminate with ctrl+c");
    println!();

    while running.load(Ordering::SeqCst) {
        if audio.has_error() {
            println!();
            println!("audio stream error detected");
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }

    println!();
    println!("SHUTDOWN");
    println!();
}

fn daemon_run(mode: ScaleMode) {
    write_pid(std::process::id());
    write_mode(mode_name(mode));

    #[cfg(target_os = "macos")]
    if !has_accessibility_permission() {
        remove_pid();
        return;
    }

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .ok();

    let (sender, receiver) = bounded(EVENT_QUEUE_SIZE);

    let audio = match AudioEngine::new(receiver) {
        Ok(engine) => engine,
        Err(_) => {
            remove_pid();
            return;
        }
    };

    let mapper = Arc::new(KeyMapper::new(mode));

    let _keyboard = match start_keyboard_listener(sender, mapper) {
        Some(listener) => listener,
        None => {
            remove_pid();
            return;
        }
    };

    while running.load(Ordering::SeqCst) {
        if audio.has_error() {
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }

    remove_pid();
}

fn parse_mode(s: &str) -> Option<ScaleMode> {
    ScaleMode::from_str(s)
}

fn prompt_mode() -> ScaleMode {
    print_header();
    let modes = &["pentatonic   C D E G A", "lydian       C D E F# G A B"];

    let selection = Select::with_theme(&SimpleTheme)
        .with_prompt("select mode")
        .items(modes)
        .default(0)
        .interact()
        .unwrap_or(0);

    match selection {
        0 => ScaleMode::Pentatonic,
        1 => ScaleMode::Lydian,
        _ => ScaleMode::Pentatonic,
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() >= 2 && args[1] == "--daemon" {
        let mode = if args.len() >= 3 {
            parse_mode(&args[2]).unwrap_or(ScaleMode::Pentatonic)
        } else {
            ScaleMode::Pentatonic
        };
        daemon_run(mode);
        return;
    }

    if args.len() < 2 {
        print_usage();
        return;
    }

    let cmd = args[1].as_str();

    if cmd == "-h" || cmd == "--help" {
        print_usage();
        return;
    }

    match cmd {
        "start" => {
            let (mode, header_shown) = if args.len() >= 3 {
                match parse_mode(&args[2]) {
                    Some(m) => (m, false),
                    None => {
                        eprintln!("unknown mode: {}", args[2]);
                        return;
                    }
                }
            } else {
                (prompt_mode(), true)
            };
            cmd_start(mode, header_shown);
        }
        "stop" => {
            cmd_stop();
        }
        "status" => {
            cmd_status();
        }
        "run" => {
            let (mode, header_shown) = if args.len() >= 3 {
                match parse_mode(&args[2]) {
                    Some(m) => (m, false),
                    None => {
                        eprintln!("unknown mode: {}", args[2]);
                        return;
                    }
                }
            } else {
                (prompt_mode(), true)
            };
            cmd_run(mode, header_shown);
        }
        _ => {
            eprintln!("unknown command: {}", cmd);
            print_usage();
        }
    }
}
