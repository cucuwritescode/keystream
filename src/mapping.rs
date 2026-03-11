//letter keys to MIDI notes
use rdev::Key;

///maps a key to a MIDI note number.
///returns None for keys that should not produce sound.
pub fn key_to_note(key: Key) -> Option<u8> {
    match key {
        //home row,C4 to D5
        Key::KeyA => Some(60), // C4
        Key::KeyS => Some(62), // D4
        Key::KeyD => Some(64), // E4
        Key::KeyF => Some(65), // F4
        Key::KeyG => Some(67), // G4
        Key::KeyH => Some(69), // A4
        Key::KeyJ => Some(71), // B4
        Key::KeyK => Some(72), // C5
        Key::KeyL => Some(74), // D5

        //top row, one octave higher
        Key::KeyQ => Some(72), // C5
        Key::KeyW => Some(74), // D5
        Key::KeyE => Some(76), // E5
        Key::KeyR => Some(77), // F5
        Key::KeyT => Some(79), // G5
        Key::KeyY => Some(81), // A5
        Key::KeyU => Some(83), // B5
        Key::KeyI => Some(84), // C6
        Key::KeyO => Some(86), // D6
        Key::KeyP => Some(88), // E6

        //bottom row, one octave lower
        Key::KeyZ => Some(48), // C3
        Key::KeyX => Some(50), // D3
        Key::KeyC => Some(52), // E3
        Key::KeyV => Some(53), // F3
        Key::KeyB => Some(55), // G3
        Key::KeyN => Some(57), // A3
        Key::KeyM => Some(59), // B3

        _ => None,
    }
}
