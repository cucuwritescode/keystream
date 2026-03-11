//entry point, Ctrl+C handler
mod audio;
mod keyboard;
mod mapping;
mod voice;

use audio::AudioEngine;
use crossbeam_channel::bounded;
use keyboard::start_keyboard_listener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use core_foundation::base::TCFType;
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::CFDictionary;
use core_foundation::string::CFString;

const EVENT_QUEUE_SIZE: usize = 256;

extern "C" {
    fn AXIsProcessTrustedWithOptions(options: core_foundation::dictionary::CFDictionaryRef)
        -> bool;
}

fn request_accessibility_permission() -> bool {
    let key = CFString::new("AXTrustedCheckOptionPrompt");
    let value = CFBoolean::true_value();
    let options = CFDictionary::from_CFType_pairs(&[(key.as_CFType(), value.as_CFType())]);
    unsafe { AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef()) }
}

extern "C" {
    fn AXIsProcessTrusted() -> bool;
}

fn has_accessibility_permission() -> bool {
    unsafe { AXIsProcessTrusted() }
}

fn main() {
    if !has_accessibility_permission() {
        eprintln!("Requesting Accessibility permission...");
        request_accessibility_permission(); // opens System Settings dialog
        eprintln!("Please grant permission, then restart the app.");
        std::process::exit(1);
    }

    if let Err(e) = listen(handle_event) {
        eprintln!("Failed to listen: {:?}", e);
    }

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let (sender, receiver) = bounded(EVENT_QUEUE_SIZE);

    let _audio = match AudioEngine::new(receiver) {
        Ok(engine) => engine,
        Err(e) => {
            eprintln!("Failed to initialise audio: {}", e);
            return;
        }
    };

    let _keyboard = match start_keyboard_listener(sender) {
        Some(listener) => listener,
        None => {
            eprintln!("Failed to initialise keyboard listener");
            return;
        }
    };

    println!("keystream running. Press Ctrl+C to exit.");

    while running.load(Ordering::SeqCst) {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    println!("\nExiting.");
}
