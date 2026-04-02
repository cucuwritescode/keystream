//recursive sine oscillator, envelope, voice pool
use std::f32::consts::PI;

const MAX_VOICES: usize = 32;
const ATTACK_MS: f32 = 5.0;
const RELEASE_MS: f32 = 200.0;

//the gordon-smith oscillator drifts in f32 precision;
//correct every 512 samples to maintain amplitude invariant
const RENORM_INTERVAL: u32 = 512;

//precomputed coefficients for all 128 midi notes
struct NoteTable {
    coeffs: [f32; 128],
    sin_omega: [f32; 128],
}

impl NoteTable {
    fn new(sample_rate: f32) -> Self {
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

    fn coeff(&self, note: u8) -> f32 {
        self.coeffs[note as usize]
    }

    fn sin_omega(&self, note: u8) -> f32 {
        self.sin_omega[note as usize]
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum VoiceState {
    Off,
    Attack,
    Sustain,
    Release,
}

#[derive(Clone, Copy)]
struct Voice {
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
    state: VoiceState,
    note_id: u8,
    age: u64,
}

impl Voice {
    const fn new() -> Self {
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

    fn start(
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

    fn release(&mut self) {
        if self.state == VoiceState::Attack || self.state == VoiceState::Sustain {
            self.state = VoiceState::Release;
        }
    }

    fn process(&mut self) -> f32 {
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

    fn envelope(&self) -> f32 {
        self.env
    }

    fn age(&self) -> u64 {
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
        if note > 127 {
            return;
        }
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
        if note > 127 {
            return;
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SAMPLE_RATE: f32 = 44100.0;

    #[test]
    fn oscillator_amplitude_stays_stable() {
        //a69 (440hz) sustained for ~2.3 seconds must not drift
        let table = NoteTable::new(TEST_SAMPLE_RATE);
        let mut voice = Voice::new();
        voice.start(69, &table, 1.0, 0.0001, 1);

        //skip past the attack phase
        for _ in 0..10 {
            voice.process();
        }

        //measure peak amplitude over a long run
        let mut max_amp: f32 = 0.0;
        let mut min_amp: f32 = f32::MAX;
        for _ in 0..100_000 {
            let sample = voice.process();
            let abs = sample.abs();
            if abs > max_amp {
                max_amp = abs;
            }
            if abs < min_amp && abs > 0.001 {
                min_amp = abs;
            }
        }

        //amplitude should stay within 1% of unity
        assert!(max_amp < 1.01, "amplitude grew to {}", max_amp);
        assert!(min_amp > 0.0, "amplitude collapsed");
    }

    #[test]
    fn envelope_attack_sustain_release() {
        let table = NoteTable::new(TEST_SAMPLE_RATE);
        let attack_step = 1.0 / 220.0; // ~5ms at 44100
        let release_step = 1.0 / 8820.0; // ~200ms at 44100
        let mut voice = Voice::new();

        assert_eq!(voice.state, VoiceState::Off);

        voice.start(60, &table, attack_step, release_step, 1);
        assert_eq!(voice.state, VoiceState::Attack);

        //run through attack
        for _ in 0..250 {
            voice.process();
        }
        assert_eq!(voice.state, VoiceState::Sustain);
        assert!((voice.envelope() - 1.0).abs() < f32::EPSILON);

        //sustain holds
        for _ in 0..1000 {
            voice.process();
        }
        assert_eq!(voice.state, VoiceState::Sustain);

        //release
        voice.release();
        assert_eq!(voice.state, VoiceState::Release);

        for _ in 0..9000 {
            voice.process();
        }
        assert_eq!(voice.state, VoiceState::Off);
        assert!(voice.envelope() == 0.0);
    }

    #[test]
    fn voice_allocation_uses_free_voices_first() {
        let mut vm = VoiceManager::new(TEST_SAMPLE_RATE);

        vm.note_on(60);
        vm.note_on(64);
        vm.note_on(67);

        //first three voices should be used, rest should be off
        for i in 0..3 {
            assert_ne!(vm.voices[i].state, VoiceState::Off);
        }
        for i in 3..MAX_VOICES {
            assert_eq!(vm.voices[i].state, VoiceState::Off);
        }
    }

    #[test]
    fn voice_stealing_prefers_releasing_voices() {
        let mut vm = VoiceManager::new(TEST_SAMPLE_RATE);

        //fill all voices
        for i in 0..MAX_VOICES {
            vm.note_on(40 + i as u8);
        }

        //release voice 0
        vm.note_off(40);
        assert_eq!(vm.voices[0].state, VoiceState::Release);

        //run a few samples so the releasing voice's envelope drops
        for _ in 0..100 {
            vm.process();
        }

        //next note should steal the releasing voice (index 0)
        vm.note_on(80);
        assert_eq!(vm.voices[0].note_id, 80);
    }

    #[test]
    fn voice_stealing_takes_oldest_when_none_releasing() {
        let mut vm = VoiceManager::new(TEST_SAMPLE_RATE);

        //fill all voices; voice 0 gets the oldest timestamp
        for i in 0..MAX_VOICES {
            vm.note_on(40 + i as u8);
        }

        //all sustaining, none releasing — should steal voice 0 (oldest)
        //run samples so all voices reach sustain
        for _ in 0..300 {
            vm.process();
        }

        vm.note_on(80);
        assert_eq!(vm.voices[0].note_id, 80);
    }

    #[test]
    fn note_off_releases_correct_voice() {
        let mut vm = VoiceManager::new(TEST_SAMPLE_RATE);

        vm.note_on(60);
        vm.note_on(64);
        vm.note_on(67);

        //run through attack
        for _ in 0..300 {
            vm.process();
        }

        vm.note_off(64);

        //voice 0 (note 60) should still sustain
        assert_eq!(vm.voices[0].state, VoiceState::Sustain);
        //voice 1 (note 64) should be releasing
        assert_eq!(vm.voices[1].state, VoiceState::Release);
        //voice 2 (note 67) should still sustain
        assert_eq!(vm.voices[2].state, VoiceState::Sustain);
    }

    #[test]
    fn invalid_note_ignored() {
        let mut vm = VoiceManager::new(TEST_SAMPLE_RATE);

        vm.note_on(128);
        vm.note_on(255);

        //no voices should have been allocated
        for voice in &vm.voices {
            assert_eq!(voice.state, VoiceState::Off);
        }
    }

    #[test]
    fn off_voice_produces_silence() {
        let mut voice = Voice::new();

        for _ in 0..100 {
            assert_eq!(voice.process(), 0.0);
        }
    }

    #[test]
    fn note_table_covers_full_midi_range() {
        let table = NoteTable::new(TEST_SAMPLE_RATE);

        for note in 0..128u8 {
            let coeff = table.coeff(note);
            let sin_w = table.sin_omega(note);

            //coefficient must be in [-2, 2] for stable oscillation
            assert!(
                (-2.0..=2.0).contains(&coeff),
                "note {} has unstable coefficient {}",
                note,
                coeff
            );
            //sin(omega) must be in [-1, 1]
            assert!(
                (-1.0..=1.0).contains(&sin_w),
                "note {} has invalid sin_omega {}",
                note,
                sin_w
            );
        }
    }
}
