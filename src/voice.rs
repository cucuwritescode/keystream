//recursive sine oscillator, envelope, voice pool
use std::f32::consts::PI;

pub const MAX_VOICES: usize = 32;
const ATTACK_MS: f32 = 5.0;
const RELEASE_MS: f32 = 200.0;

//the gordon-smith oscillator drifts in f32 precision;
//correct every 512 samples to maintain amplitude invariant
const RENORM_INTERVAL: u32 = 512;

//precomputed coefficients for all 128 midi notes
pub struct NoteTable {
    coeffs: [f32; 128],
    sin_omega: [f32; 128],
}

impl NoteTable {
    pub fn new(sample_rate: f32) -> Self {
        let mut coeffs = [0.0f32; 128];
        let mut sin_omega = [0.0f32; 128];

        for note in 0..128 {
            let freq = 440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0);
            let omega = 2.0 * PI * freq / sample_rate;
            coeffs[note] = 2.0 * omega.cos();
            sin_omega[note] = omega.sin();
        }

        Self { coeffs, sin_omega }
    }

    pub fn coeff(&self, note: u8) -> f32 {
        self.coeffs[note as usize]
    }

    pub fn sin_omega(&self, note: u8) -> f32 {
        self.sin_omega[note as usize]
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
    //oscillator
    y1: f32,
    y2: f32,
    coeff: f32,
    sin_omega: f32,
    samples_since_renorm: u32,

    //envelope
    env: f32,
    attack_step: f32,
    release_step: f32,

    //metadata
    pub state: VoiceState,
    pub note_id: u8,
    age: u64,
}

impl Voice {
    pub const fn new() -> Self {
        Self {
            y1: 0.0,
            y2: 0.0,
            coeff: 0.0,
            sin_omega: 0.0,
            samples_since_renorm: 0,
            env: 0.0,
            attack_step: 0.0,
            release_step: 0.0,
            state: VoiceState::Off,
            note_id: 0,
            age: 0,
        }
    }

    pub fn start(
        &mut self,
        note: u8,
        table: &NoteTable,
        attack_step: f32,
        release_step: f32,
        current_time: u64,
    ) {
        self.coeff = table.coeff(note);
        self.sin_omega = table.sin_omega(note);
        self.y1 = self.sin_omega;
        self.y2 = 0.0;
        self.samples_since_renorm = 0;
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

        //gordon-smith recursive sine: y[n] = 2cos(w)·y[n-1] - y[n-2]
        let y = self.coeff * self.y1 - self.y2;
        self.y2 = self.y1;
        self.y1 = y;

        //correct floating-point amplitude drift periodically
        self.samples_since_renorm += 1;
        if self.samples_since_renorm >= RENORM_INTERVAL {
            self.samples_since_renorm = 0;
            self.renormalise();
        }

        match self.state {
            VoiceState::Attack => {
                self.env += self.attack_step;
                if self.env >= 1.0 {
                    self.env = 1.0;
                    self.state = VoiceState::Sustain;
                }
            }
            VoiceState::Sustain => {}
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

    //the oscillator conserves: y1² + y2² - c·y1·y2 = sin²(w)
    //floating-point error breaks this invariant; restore it here
    fn renormalise(&mut self) {
        let energy = self.y1 * self.y1 + self.y2 * self.y2 - self.coeff * self.y1 * self.y2;
        let target = self.sin_omega * self.sin_omega;
        if energy > 0.0 {
            let correction = (target / energy).sqrt();
            self.y1 *= correction;
            self.y2 *= correction;
        }
    }

    pub fn envelope(&self) -> f32 {
        self.env
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
        let attack_samples = (ATTACK_MS / 1000.0) * sample_rate;
        let release_samples = (RELEASE_MS / 1000.0) * sample_rate;

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
        let idx = self.allocate_voice();
        self.voices[idx].start(
            note,
            &self.table,
            self.attack_step,
            self.release_step,
            self.time,
        );
    }

    pub fn note_off(&mut self, note: u8) {
        for voice in &mut self.voices {
            if voice.note_id == note
                && (voice.state == VoiceState::Attack || voice.state == VoiceState::Sustain)
            {
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

    //priority: free voice > quietest releasing voice > oldest active voice
    fn allocate_voice(&self) -> usize {
        if let Some(idx) = self.voices.iter().position(|v| v.state == VoiceState::Off) {
            return idx;
        }

        //steal the releasing voice nearest to silence to minimise audible click
        let releasing = self
            .voices
            .iter()
            .enumerate()
            .filter(|(_, v)| v.state == VoiceState::Release)
            .min_by(|(_, a), (_, b)| {
                a.envelope()
                    .partial_cmp(&b.envelope())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

        if let Some((idx, _)) = releasing {
            return idx;
        }

        //no releasing voices; steal the one that has played the longest
        self.voices
            .iter()
            .enumerate()
            .min_by_key(|(_, v)| v.age())
            .map(|(i, _)| i)
            .unwrap_or(0)
    }
}
