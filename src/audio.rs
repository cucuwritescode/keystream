use crate::keyboard::KeyEvent;
use crate::voice::VoiceManager;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, Stream, StreamConfig};
use crossbeam_channel::Receiver;

const MASTER_GAIN: f32 = 0.1;

pub struct AudioEngine {
    _stream: Stream,
}

impl AudioEngine {
    pub fn new(event_receiver: Receiver<KeyEvent>) -> Result<Self, AudioError> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or(AudioError::NoOutputDevice)?;

        let config = device.default_output_config()?;
        let sample_rate = config.sample_rate().0 as f32;

        let stream = match config.sample_format() {
            SampleFormat::F32 => build_stream::<f32>(&device, &config.into(), event_receiver, sample_rate),
            SampleFormat::I16 => build_stream::<i16>(&device, &config.into(), event_receiver, sample_rate),
            SampleFormat::U16 => build_stream::<u16>(&device, &config.into(), event_receiver, sample_rate),
            _ => return Err(AudioError::UnsupportedFormat),
        }?;

        stream.play()?;

        Ok(Self { _stream: stream })
    }
}

fn build_stream<T>(
    device: &Device,
    config: &StreamConfig,
    event_receiver: Receiver<KeyEvent>,
    sample_rate: f32,
) -> Result<Stream, AudioError>
where
    T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
{
    let channels = config.channels as usize;
    let mut voice_manager = VoiceManager::new(sample_rate);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            // Process events (non-blocking)
            while let Ok(event) = event_receiver.try_recv() {
                match event {
                    KeyEvent::Down(note) => voice_manager.note_on(note),
                    KeyEvent::Up(note) => voice_manager.note_off(note),
                }
            }

            // Generate audio
            for frame in data.chunks_mut(channels) {
                let sample = voice_manager.process() * MASTER_GAIN;
                let sample_t = T::from_sample(sample);
                for channel in frame.iter_mut() {
                    *channel = sample_t;
                }
            }
        },
        |err| eprintln!("Audio stream error: {}", err),
        None,
    )?;

    Ok(stream)
}

#[derive(Debug)]
pub enum AudioError {
    NoOutputDevice,
    UnsupportedFormat,
    StreamError(cpal::BuildStreamError),
    PlayError(cpal::PlayStreamError),
    DeviceError(cpal::DefaultStreamConfigError),
}

impl From<cpal::BuildStreamError> for AudioError {
    fn from(e: cpal::BuildStreamError) -> Self {
        AudioError::StreamError(e)
    }
}

impl From<cpal::PlayStreamError> for AudioError {
    fn from(e: cpal::PlayStreamError) -> Self {
        AudioError::PlayError(e)
    }
}

impl From<cpal::DefaultStreamConfigError> for AudioError {
    fn from(e: cpal::DefaultStreamConfigError) -> Self {
        AudioError::DeviceError(e)
    }
}

impl std::fmt::Display for AudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioError::NoOutputDevice => write!(f, "No audio output device found"),
            AudioError::UnsupportedFormat => write!(f, "Unsupported sample format"),
            AudioError::StreamError(e) => write!(f, "Stream error: {}", e),
            AudioError::PlayError(e) => write!(f, "Play error: {}", e),
            AudioError::DeviceError(e) => write!(f, "Device error: {}", e),
        }
    }
}

impl std::error::Error for AudioError {}
