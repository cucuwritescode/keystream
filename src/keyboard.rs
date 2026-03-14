//global keyboard capture via device_query (event-based)
use crate::mapping::{key_to_mapping, KeyMapping};
use crossbeam_channel::Sender;
use device_query::{DeviceEvents, DeviceEventsHandler, Keycode};
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub enum KeyEvent {
    Down(u8),
    Up(u8),
}

pub struct KeyboardListener {
    _handler: DeviceEventsHandler,
    _down_guard: Box<dyn std::any::Any + Send>,
    _up_guard: Box<dyn std::any::Any + Send>,
}

pub fn start_keyboard_listener(sender: Sender<KeyEvent>) -> Option<KeyboardListener> {
    let handler = DeviceEventsHandler::new(Duration::from_millis(10))?;

    let sender_down = sender.clone();
    let down_guard = handler.on_key_down(move |key: &Keycode| {
        match key_to_mapping(*key) {
            KeyMapping::Note(note) => {
                let _ = sender_down.try_send(KeyEvent::Down(note));
            }
            KeyMapping::Chord(notes) => {
                for &note in notes {
                    let _ = sender_down.try_send(KeyEvent::Down(note));
                }
            }
            KeyMapping::None => {}
        }
    });

    let sender_up = sender;
    let up_guard = handler.on_key_up(move |key: &Keycode| {
        match key_to_mapping(*key) {
            KeyMapping::Note(note) => {
                let _ = sender_up.try_send(KeyEvent::Up(note));
            }
            KeyMapping::Chord(notes) => {
                for &note in notes {
                    let _ = sender_up.try_send(KeyEvent::Up(note));
                }
            }
            KeyMapping::None => {}
        }
    });

    Some(KeyboardListener {
        _handler: handler,
        _down_guard: Box::new(down_guard),
        _up_guard: Box::new(up_guard),
    })
}
