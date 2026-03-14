//scale modes with circle-of-fifths row shifts
use device_query::Keycode;

#[derive(Clone, Copy, Debug)]
pub enum ScaleMode {
    Pentatonic,  // C D E G A
    Lydian,      // C D E F# G A B - bright, cinematic
}

impl ScaleMode {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "pentatonic" | "penta" | "p" => Some(Self::Pentatonic),
            "lydian" | "lyd" | "l" => Some(Self::Lydian),
            _ => None,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Pentatonic => "pentatonic (C D E G A)",
            Self::Lydian => "lydian (C D E F# G A B)",
        }
    }
}

pub enum KeyMapping {
    None,
    Note(u8),
    Chord(&'static [u8]),
}

pub struct KeyMapper {
    mode: ScaleMode,
}

impl KeyMapper {
    pub fn new(mode: ScaleMode) -> Self {
        Self { mode }
    }

    pub fn map(&self, key: Keycode) -> KeyMapping {
        //special keys (same for all modes)
        match key {
            //Enter - Cadd9 (C E G D)
            Keycode::Enter => return KeyMapping::Chord(&[48, 52, 55, 62]), // C3 E3 G3 D4

            //Space - sustain drone (C2 + G2)
            Keycode::Space => return KeyMapping::Chord(&[36, 43]), // C2 G2

            //Backspace - minor third (E + C)
            Keycode::Backspace => return KeyMapping::Chord(&[64, 60]), // E4 C4

            //Tab - harmonic shift (G + D)
            Keycode::Tab => return KeyMapping::Chord(&[67, 62]), // G4 D4

            _ => {}
        }

        //scale-dependent mapping
        match self.mode {
            ScaleMode::Pentatonic => self.map_pentatonic(key),
            ScaleMode::Lydian => self.map_lydian(key),
        }
    }

    //pentatonic: C D E G A per row, fifths apart
    fn map_pentatonic(&self, key: Keycode) -> KeyMapping {
        match key {
            //top row - C pentatonic
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

            //home row - G pentatonic (fifth below)
            Keycode::A => KeyMapping::Note(55),  // G3
            Keycode::S => KeyMapping::Note(57),  // A3
            Keycode::D => KeyMapping::Note(59),  // B3
            Keycode::F => KeyMapping::Note(62),  // D4
            Keycode::G => KeyMapping::Note(64),  // E4
            Keycode::H => KeyMapping::Note(67),  // G4
            Keycode::J => KeyMapping::Note(69),  // A4
            Keycode::K => KeyMapping::Note(71),  // B4
            Keycode::L => KeyMapping::Note(74),  // D5

            //bottom row - D pentatonic (another fifth below)
            Keycode::Z => KeyMapping::Note(50),  // D3
            Keycode::X => KeyMapping::Note(52),  // E3
            Keycode::C => KeyMapping::Note(54),  // F#3
            Keycode::V => KeyMapping::Note(57),  // A3
            Keycode::B => KeyMapping::Note(59),  // B3
            Keycode::N => KeyMapping::Note(62),  // D4
            Keycode::M => KeyMapping::Note(64),  // E4

            _ => KeyMapping::None,
        }
    }

    //lydian: C D E F# G A B - bright, cinematic
    fn map_lydian(&self, key: Keycode) -> KeyMapping {
        match key {
            //top row - C lydian
            Keycode::Q => KeyMapping::Note(60),  // C4
            Keycode::W => KeyMapping::Note(62),  // D4
            Keycode::E => KeyMapping::Note(64),  // E4
            Keycode::R => KeyMapping::Note(66),  // F#4
            Keycode::T => KeyMapping::Note(67),  // G4
            Keycode::Y => KeyMapping::Note(69),  // A4
            Keycode::U => KeyMapping::Note(71),  // B4
            Keycode::I => KeyMapping::Note(72),  // C5
            Keycode::O => KeyMapping::Note(74),  // D5
            Keycode::P => KeyMapping::Note(76),  // E5

            //home row - G lydian (fifth below)
            Keycode::A => KeyMapping::Note(55),  // G3
            Keycode::S => KeyMapping::Note(57),  // A3
            Keycode::D => KeyMapping::Note(59),  // B3
            Keycode::F => KeyMapping::Note(60),  // C4
            Keycode::G => KeyMapping::Note(61),  // C#4
            Keycode::H => KeyMapping::Note(62),  // D4
            Keycode::J => KeyMapping::Note(64),  // E4
            Keycode::K => KeyMapping::Note(66),  // F#4
            Keycode::L => KeyMapping::Note(67),  // G4

            //bottom row - D lydian (another fifth below)
            Keycode::Z => KeyMapping::Note(50),  // D3
            Keycode::X => KeyMapping::Note(52),  // E3
            Keycode::C => KeyMapping::Note(54),  // F#3
            Keycode::V => KeyMapping::Note(55),  // G3
            Keycode::B => KeyMapping::Note(56),  // G#3
            Keycode::N => KeyMapping::Note(57),  // A3
            Keycode::M => KeyMapping::Note(59),  // B3

            _ => KeyMapping::None,
        }
    }
}
