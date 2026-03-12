//recursive sine oscillator, envelope, voice pool
use std::f32::consts::PI;

pub const MAX_VOICES: usize = 32;
const ATTACK_TIME_MS: f32 = 5.0;
const RELEASE_TIME_MS: f32 = 200.0;

#[derive(Clone, Copy, PartialEq)]
pub enum VoiceState {
    Off,
    Attack,
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

    pub fn start(&mut self, note: u8, sample_rate: f32, current_time: u64) {
        let freq = midi_to_freq(note);
        let omega = 2.0 * PI * freq / sample_rate;

        self.coeff = 2.0 * omega.cos();
        self.y1 = omega.sin();
        self.y2 = 0.0;
        self.env = 0.0;
        self.state = VoiceState::Attack;
        self.note_id = note;
        self.age = current_time;

        let attack_samples = (ATTACK_TIME_MS / 1000.0) * sample_rate;
        let release_samples = (RELEASE_TIME_MS / 1000.0) * sample_rate;
        self.attack_step = 1.0 / attack_samples;
        self.release_step = 1.0 / release_samples;
    }

    pub fn release(&mut self) {
        if self.state == VoiceState::Attack {
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
                    self.state = VoiceState::Release;
                }
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

fn midi_to_freq(note: u8) -> f32 {
    440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
}

pub struct VoiceManager {
    voices: [Voice; MAX_VOICES],
    time: u64,
}

impl VoiceManager {
    pub fn new() -> Self {
        Self {
            voices: [Voice::new(); MAX_VOICES],
            time: 0,
        }
    }

    pub fn note_on(&mut self, note: u8, sample_rate: f32) {
        self.time += 1;

        //find a free voice or steal the oldest
        let voice_idx = self.find_free_voice().unwrap_or_else(|| self.find_oldest_voice());

        self.voices[voice_idx].start(note, sample_rate, self.time);
    }

    pub fn note_off(&mut self, note: u8) {
        for voice in &mut self.voices {
            if voice.note_id == note && voice.state == VoiceState::Attack {
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
            .min_by_key(|(_, v)| v.age())
            .map(|(i, _)| i)
            .unwrap_or(0)
    }
}
