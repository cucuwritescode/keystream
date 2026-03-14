//scale modes with circle-of-fifths row shifts
use device_query::Keycode;

#[derive(Clone, Copy, Debug)]
pub enum ScaleMode {
    Pentatonic,  //C D E G A
    Lydian,      //C D E F# G A B 
}

impl ScaleMode {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "pentatonic" | "penta" | "p" => Some(Self::Pentatonic),
            "lydian" | "lyd" | "l" => Some(Self::Lydian),
            _ => None,
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
            //enter,cadence (C E G D)
            Keycode::Enter => return KeyMapping::Chord(&[48, 52, 55, 62]), // C3 E3 G3 D4

            //space,drone (C2 + G2)
            Keycode::Space => return KeyMapping::Chord(&[36, 43]), // C2 G2

            //backspace,descending minor third (E C)
            Keycode::Backspace => return KeyMapping::Chord(&[64, 60]), // E4 C4

            //tab,harmonic shift (G + D)
            Keycode::Tab => return KeyMapping::Chord(&[67, 62]), // G4 D4

            _ => {}
        }

        //scale-dependent mapping
        match self.mode {
            ScaleMode::Pentatonic => self.map_pentatonic(key),
            ScaleMode::Lydian => self.map_lydian(key),
        }
    }

    //pentatonic: letters=melody, numbers=high accents, punctuation=gestures
    fn map_pentatonic(&self, key: Keycode) -> KeyMapping {
        match key {
            //letters,melody

            //top row,C pentatonic
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

            //home row,G pentatonic (fifth below)
            Keycode::A => KeyMapping::Note(55),  // G3
            Keycode::S => KeyMapping::Note(57),  // A3
            Keycode::D => KeyMapping::Note(59),  // B3
            Keycode::F => KeyMapping::Note(62),  // D4
            Keycode::G => KeyMapping::Note(64),  // E4
            Keycode::H => KeyMapping::Note(67),  // G4
            Keycode::J => KeyMapping::Note(69),  // A4
            Keycode::K => KeyMapping::Note(71),  // B4
            Keycode::L => KeyMapping::Note(74),  // D5

            //bottom row,D pentatonic (another fifth below)
            Keycode::Z => KeyMapping::Note(50),  // D3
            Keycode::X => KeyMapping::Note(52),  // E3
            Keycode::C => KeyMapping::Note(54),  // F#3
            Keycode::V => KeyMapping::Note(57),  // A3
            Keycode::B => KeyMapping::Note(59),  // B3
            Keycode::N => KeyMapping::Note(62),  // D4
            Keycode::M => KeyMapping::Note(64),  // E4

            //numbers,high register accents
            Keycode::Key1 => KeyMapping::Note(72),  // C5
            Keycode::Key2 => KeyMapping::Note(74),  // D5
            Keycode::Key3 => KeyMapping::Note(76),  // E5
            Keycode::Key4 => KeyMapping::Note(79),  // G5
            Keycode::Key5 => KeyMapping::Note(81),  // A5
            Keycode::Key6 => KeyMapping::Note(84),  // C6
            Keycode::Key7 => KeyMapping::Note(86),  // D6
            Keycode::Key8 => KeyMapping::Note(88),  // E6
            Keycode::Key9 => KeyMapping::Note(91),  // G6
            Keycode::Key0 => KeyMapping::Note(93),  // A6

            //punctuation,gestures

            //period,. - upward flourish G->A
            Keycode::Dot => KeyMapping::Chord(&[67, 69]), // G4 A4

            //comma, - downward flourish A->G
            Keycode::Comma => KeyMapping::Chord(&[69, 67]), // A4 G4

            //semicolon, - small dyad E+A
            Keycode::Semicolon => KeyMapping::Chord(&[64, 69]), // E4 A4

            //slash, - descending gesture A->G->E
            Keycode::Slash => KeyMapping::Chord(&[69, 67, 64]), // A4 G4 E4

            //backslash, - ascending gesture C->D->E
            Keycode::BackSlash => KeyMapping::Chord(&[60, 62, 64]), // C4 D4 E4

            //brackets,harmonic markers

            //left bracket - C+G open fifth
            Keycode::LeftBracket => KeyMapping::Chord(&[60, 67]), // C4 G4

            //right bracket - E+A resolution
            Keycode::RightBracket => KeyMapping::Chord(&[64, 69]), // E4 A4

            //left brace (shift+[) - big harmonic hit C+E+A
            //right brace (shift+]) - closing chord D+G+A
            //(device_query doesn't distinguish shift, so using [ and ] for now)

            //minus,soft low note
            Keycode::Minus => KeyMapping::Note(48), // C3

            //equals,bright high note
            Keycode::Equal => KeyMapping::Note(84), // C6

            //grave/backtick,deep bass
            Keycode::Grave => KeyMapping::Note(36), // C2

            //quote,gentle dyad
            Keycode::Apostrophe => KeyMapping::Chord(&[60, 64]), // C4 E4

            _ => KeyMapping::None,
        }
    }

    //lydian: C D E F# G A B - bright, cinematic
    fn map_lydian(&self, key: Keycode) -> KeyMapping {
        match key {
            //top row,C lydian
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

            //home row,G lydian (fifth below)
            Keycode::A => KeyMapping::Note(55),  // G3
            Keycode::S => KeyMapping::Note(57),  // A3
            Keycode::D => KeyMapping::Note(59),  // B3
            Keycode::F => KeyMapping::Note(60),  // C4
            Keycode::G => KeyMapping::Note(61),  // C#4
            Keycode::H => KeyMapping::Note(62),  // D4
            Keycode::J => KeyMapping::Note(64),  // E4
            Keycode::K => KeyMapping::Note(66),  // F#4
            Keycode::L => KeyMapping::Note(67),  // G4

            //bottom row,D lydian (another fifth below)
            Keycode::Z => KeyMapping::Note(50),  // D3
            Keycode::X => KeyMapping::Note(52),  // E3
            Keycode::C => KeyMapping::Note(54),  // F#3
            Keycode::V => KeyMapping::Note(55),  // G3
            Keycode::B => KeyMapping::Note(56),  // G#3
            Keycode::N => KeyMapping::Note(57),  // A3
            Keycode::M => KeyMapping::Note(59),  // B3

            //numbers,high lydian
            Keycode::Key1 => KeyMapping::Note(72),  // C5
            Keycode::Key2 => KeyMapping::Note(74),  // D5
            Keycode::Key3 => KeyMapping::Note(76),  // E5
            Keycode::Key4 => KeyMapping::Note(78),  // F#5
            Keycode::Key5 => KeyMapping::Note(79),  // G5
            Keycode::Key6 => KeyMapping::Note(81),  // A5
            Keycode::Key7 => KeyMapping::Note(83),  // B5
            Keycode::Key8 => KeyMapping::Note(84),  // C6
            Keycode::Key9 => KeyMapping::Note(86),  // D6
            Keycode::Key0 => KeyMapping::Note(88),  // E6

            //punctuation,lydian gestures
            Keycode::Dot => KeyMapping::Chord(&[66, 69]), // F#4 A4 (lydian color)
            Keycode::Comma => KeyMapping::Chord(&[69, 66]), // A4 F#4
            Keycode::Semicolon => KeyMapping::Chord(&[64, 66]), // E4 F#4
            Keycode::Slash => KeyMapping::Chord(&[71, 69, 67]), // B4 A4 G4
            Keycode::BackSlash => KeyMapping::Chord(&[60, 62, 66]), // C4 D4 F#4
            Keycode::LeftBracket => KeyMapping::Chord(&[60, 66]), // C4 F#4 (tritone)
            Keycode::RightBracket => KeyMapping::Chord(&[64, 71]), // E4 B4
            Keycode::Minus => KeyMapping::Note(48), // C3
            Keycode::Equal => KeyMapping::Note(84), // C6
            Keycode::Grave => KeyMapping::Note(36), // C2
            Keycode::Apostrophe => KeyMapping::Chord(&[60, 66]), // C4 F#4

            _ => KeyMapping::None,
        }
    }
}
