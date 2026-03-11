//global keyboard capture via rdev
use crate::mapping::key_to_note;
use crossbeam_channel::Sender;
use rdev::{listen, Event, EventType, Key};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone, Copy)]
pub enum KeyEvent {
    Down(u8),
    Up(u8),
}

pub fn start_keyboard_listener(sender: Sender<KeyEvent>) {
    let held_keys: Arc<Mutex<HashSet<Key>>> = Arc::new(Mutex::new(HashSet::new()));

    thread::spawn(move || {
        let held = held_keys.clone();
        let callback = move |event: Event| {
            match event.event_type {
                EventType::KeyPress(key) => {
                    if is_modifier(key) {
                        return;
                    }

                    //filter auto-repeat
                    {
                        let mut held_guard = held.lock().unwrap();
                        if held_guard.contains(&key) {
                            return;
                        }
                        held_guard.insert(key);
                    }

                    if let Some(note) = key_to_note(key) {
                        let _ = sender.try_send(KeyEvent::Down(note));
                    }
                }
                EventType::KeyRelease(key) => {
                    if is_modifier(key) {
                        return;
                    }

                    {
                        let mut held_guard = held.lock().unwrap();
                        held_guard.remove(&key);
                    }

                    if let Some(note) = key_to_note(key) {
                        let _ = sender.try_send(KeyEvent::Up(note));
                    }
                }
                _ => {}
            }
        };

        if let Err(e) = listen(callback) {
            eprintln!("Keyboard listener error: {:?}", e);
        }
    });
}

fn is_modifier(key: Key) -> bool {
    matches!(
        key,
        Key::ShiftLeft
            | Key::ShiftRight
            | Key::ControlLeft
            | Key::ControlRight
            | Key::Alt
            | Key::AltGr
            | Key::MetaLeft
            | Key::MetaRight
    )
}
