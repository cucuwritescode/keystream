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

const EVENT_QUEUE_SIZE: usize = 256;

fn main() {
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
            eprintln!("Failed to initialize audio: {}", e);
            return;
        }
    };

    start_keyboard_listener(sender);

    println!("keystream running. Press Ctrl+C to exit.");

    while running.load(Ordering::SeqCst) {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    println!("\nExiting.");
}
