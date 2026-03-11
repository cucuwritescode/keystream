//letter keys to MIDI notes
use device_query::Keycode;

pub fn key_to_note(key: Keycode) -> Option<u8> {
    match key {
        //home row, C4 to D5
        Keycode::A => Some(60), // C4
        Keycode::S => Some(62), // D4
        Keycode::D => Some(64), // E4
        Keycode::F => Some(65), // F4
        Keycode::G => Some(67), // G4
        Keycode::H => Some(69), // A4
        Keycode::J => Some(71), // B4
        Keycode::K => Some(72), // C5
        Keycode::L => Some(74), // D5

        //top row, one octave higher
        Keycode::Q => Some(72), // C5
        Keycode::W => Some(74), // D5
        Keycode::E => Some(76), // E5
        Keycode::R => Some(77), // F5
        Keycode::T => Some(79), // G5
        Keycode::Y => Some(81), // A5
        Keycode::U => Some(83), // B5
        Keycode::I => Some(84), // C6
        Keycode::O => Some(86), // D6
        Keycode::P => Some(88), // E6

        //bottom row, one octave lower
        Keycode::Z => Some(48), // C3
        Keycode::X => Some(50), // D3
        Keycode::C => Some(52), // E3
        Keycode::V => Some(53), // F3
        Keycode::B => Some(55), // G3
        Keycode::N => Some(57), // A3
        Keycode::M => Some(59), // B3

        _ => None,
    }
}
