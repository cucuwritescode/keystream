//recursive sine oscillator, envelope, voice pool
use std::f32::consts::PI;

pub const MAX_VOICES: usize = 32;
const ATTACK_TIME_MS: f32 = 5.0;
const RELEASE_TIME_MS: f32 = 200.0;

//precomputed coefficients for all 128 MIDI notes
pub struct NoteTable {
    coeffs: [f32; 128],
    y1_init: [f32; 128],
}

impl NoteTable {
    pub fn new(sample_rate: f32) -> Self {
        let mut coeffs = [0.0f32; 128];
        let mut y1_init = [0.0f32; 128];

        for note in 0..128 {
            let freq = 440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0);
            let omega = 2.0 * PI * freq / sample_rate;
            coeffs[note] = 2.0 * omega.cos();
            y1_init[note] = omega.sin();
        }

        Self { coeffs, y1_init }
    }

    pub fn coeff(&self, note: u8) -> f32 {
        self.coeffs[note as usize]
    }

    pub fn y1_init(&self, note: u8) -> f32 {
        self.y1_init[note as usize]
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum VoiceState {
    Off,
    Attack,
    Sustain,
    Release,
}

#[derive(Clone, Copy)]
pub struct Voice {
    y1: f32,
    y2: f32,
    coeff: f32,
    env: f32,
    attack_step: f32,
    release_step: f32,
    pub state: VoiceState,
    pub note_id: u8,
    age: u64,
}

impl Voice {
    pub fn new() -> Self {
        Self {
            y1: 0.0,
            y2: 0.0,
            coeff: 0.0,
            env: 0.0,
            attack_step: 0.0,
            release_step: 0.0,
            state: VoiceState::Off,
            note_id: 0,
            age: 0,
        }
    }

    //no trig here - uses precomputed values
    pub fn start(&mut self, note: u8, table: &NoteTable, attack_step: f32, release_step: f32, current_time: u64) {
        self.coeff = table.coeff(note);
        self.y1 = table.y1_init(note);
        self.y2 = 0.0;
        self.env = 0.0;
        self.state = VoiceState::Attack;
        self.note_id = note;
        self.age = current_time;
        self.attack_step = attack_step;
        self.release_step = release_step;
    }

    pub fn release(&mut self) {
        if self.state == VoiceState::Attack || self.state == VoiceState::Sustain {
            self.state = VoiceState::Release;
        }
    }

    pub fn process(&mut self) -> f32 {
        if self.state == VoiceState::Off {
            return 0.0;
        }

        //recursive sine oscillator
        let y = self.coeff * self.y1 - self.y2;
        self.y2 = self.y1;
        self.y1 = y;

        //envelope processing
        match self.state {
            VoiceState::Attack => {
                self.env += self.attack_step;
                if self.env >= 1.0 {
                    self.env = 1.0;
                    self.state = VoiceState::Sustain;
                }
            }
            VoiceState::Sustain => {
                //hold at 1.0 until release
            }
            VoiceState::Release => {
                self.env -= self.release_step;
                if self.env <= 0.0 {
                    self.env = 0.0;
                    self.state = VoiceState::Off;
                }
            }
            VoiceState::Off => {}
        }

        y * self.env
    }

    pub fn age(&self) -> u64 {
        self.age
    }
}

pub struct VoiceManager {
    voices: [Voice; MAX_VOICES],
    table: NoteTable,
    attack_step: f32,
    release_step: f32,
    time: u64,
}

impl VoiceManager {
    pub fn new(sample_rate: f32) -> Self {
        let attack_samples = (ATTACK_TIME_MS / 1000.0) * sample_rate;
        let release_samples = (RELEASE_TIME_MS / 1000.0) * sample_rate;

        Self {
            voices: [Voice::new(); MAX_VOICES],
            table: NoteTable::new(sample_rate),
            attack_step: 1.0 / attack_samples,
            release_step: 1.0 / release_samples,
            time: 0,
        }
    }

    pub fn note_on(&mut self, note: u8) {
        self.time += 1;

        //find a free voice or steal the oldest
        let voice_idx = self.find_free_voice().unwrap_or_else(|| self.find_oldest_voice());

        self.voices[voice_idx].start(note, &self.table, self.attack_step, self.release_step, self.time);
    }

    pub fn note_off(&mut self, note: u8) {
        for voice in &mut self.voices {
            if voice.note_id == note && (voice.state == VoiceState::Attack || voice.state == VoiceState::Sustain) {
                voice.release();
            }
        }
    }

    pub fn process(&mut self) -> f32 {
        let mut output = 0.0;
        for voice in &mut self.voices {
            output += voice.process();
        }
        output
    }

    fn find_free_voice(&self) -> Option<usize> {
        self.voices
            .iter()
            .position(|v| v.state == VoiceState::Off)
    }

    fn find_oldest_voice(&self) -> usize {
        self.voices
            .iter()
            .enumerate()
            .max_by_key(|(_, v)| v.age())
            .map(|(i, _)| i)
            .unwrap_or(0)
    }
}
