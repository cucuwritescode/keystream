//pentatonic layout with circle-of-fifths row shifts
use device_query::Keycode;

pub enum KeyMapping {
    None,
    Note(u8),
    Chord(&'static [u8]),
}

pub fn key_to_mapping(key: Keycode) -> KeyMapping {
    match key {
        //top row - C pentatonic (C D E G A)
        Keycode::Q => KeyMapping::Note(60),  // C4
        Keycode::W => KeyMapping::Note(62),  // D4
        Keycode::E => KeyMapping::Note(64),  // E4
        Keycode::R => KeyMapping::Note(67),  // G4
        Keycode::T => KeyMapping::Note(69),  // A4
        Keycode::Y => KeyMapping::Note(72),  // C5
        Keycode::U => KeyMapping::Note(74),  // D5
        Keycode::I => KeyMapping::Note(76),  // E5
        Keycode::O => KeyMapping::Note(79),  // G5
        Keycode::P => KeyMapping::Note(81),  // A5

        //home row - G pentatonic (G A B D E) - fifth below
        Keycode::A => KeyMapping::Note(55),  // G3
        Keycode::S => KeyMapping::Note(57),  // A3
        Keycode::D => KeyMapping::Note(59),  // B3
        Keycode::F => KeyMapping::Note(62),  // D4
        Keycode::G => KeyMapping::Note(64),  // E4
        Keycode::H => KeyMapping::Note(67),  // G4
        Keycode::J => KeyMapping::Note(69),  // A4
        Keycode::K => KeyMapping::Note(71),  // B4
        Keycode::L => KeyMapping::Note(74),  // D5

        //bottom row - D pentatonic (D E F# A B) - another fifth below
        Keycode::Z => KeyMapping::Note(50),  // D3
        Keycode::X => KeyMapping::Note(52),  // E3
        Keycode::C => KeyMapping::Note(54),  // F#3
        Keycode::V => KeyMapping::Note(57),  // A3
        Keycode::B => KeyMapping::Note(59),  // B3
        Keycode::N => KeyMapping::Note(62),  // D4
        Keycode::M => KeyMapping::Note(64),  // E4

        //special keys
        //Enter - maj7#11 lydian chord (C G D E F# B)
        Keycode::Enter => KeyMapping::Chord(&[48, 55, 62, 64, 66, 71]), // C3 G3 D4 E4 F#4 B4

        //Space - sustain drone (C2 + G2)
        Keycode::Space => KeyMapping::Chord(&[36, 43]), // C2 G2

        //Backspace - descending minor third (E4 then C4 - handled as chord, release creates gesture)
        Keycode::Backspace => KeyMapping::Chord(&[64, 60]), // E4 C4

        //Tab - harmonic shift (G D)
        Keycode::Tab => KeyMapping::Chord(&[67, 62]), // G4 D4

        _ => KeyMapping::None,
    }
}
