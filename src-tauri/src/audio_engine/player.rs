use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use crate::audio_engine::decoder::AudioBuffer;

struct PlayerState {
    buffer: AudioBuffer,
    position_frames: usize,
    playing: bool,
}

pub struct AudioPlayer {
    state: Arc<Mutex<PlayerState>>,
    _stream: Option<cpal::Stream>,
}

impl AudioPlayer {
    pub fn new(buffer: AudioBuffer) -> Self {
        let state = Arc::new(Mutex::new(PlayerState {
            buffer,
            position_frames: 0,
            playing: false,
        }));

        let state_clone = Arc::clone(&state);
        let stream = Self::build_stream(state_clone).ok();

        AudioPlayer { state, _stream: stream }
    }

    fn build_stream(state: Arc<Mutex<PlayerState>>) -> Result<cpal::Stream, cpal::BuildStreamError> {
        let host = cpal::default_host();
        let device = host.default_output_device()
            .ok_or(cpal::BuildStreamError::DeviceNotAvailable)?;

        let (sample_rate, channels) = {
            let s = state.lock().unwrap();
            (s.buffer.sample_rate, s.buffer.channels as usize)
        };

        let config = cpal::StreamConfig {
            channels: channels as u16,
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let stream = device.build_output_stream(
            &config,
            move |output: &mut [f32], _| {
                let mut s = state.lock().unwrap();
                if !s.playing {
                    output.fill(0.0);
                    return;
                }
                for sample in output.iter_mut() {
                    if s.position_frames < s.buffer.samples.len() {
                        *sample = s.buffer.samples[s.position_frames];
                        s.position_frames += 1;
                    } else {
                        *sample = 0.0;
                        s.playing = false;
                    }
                }
            },
            |_err| {},
            None,
        )?;

        stream.play().ok();
        Ok(stream)
    }

    pub fn play(&self) {
        let mut s = self.state.lock().unwrap();
        s.playing = true;
    }

    pub fn pause(&self) {
        let mut s = self.state.lock().unwrap();
        s.playing = false;
    }

    pub fn seek(&self, position_ms: u64) {
        let mut s = self.state.lock().unwrap();
        let target_frame = ((position_ms as f64 / 1000.0) * s.buffer.sample_rate as f64) as usize;
        s.position_frames = (target_frame * s.buffer.channels as usize).min(s.buffer.samples.len());
    }

    pub fn position_ms(&self) -> u64 {
        let s = self.state.lock().unwrap();
        let frames = s.position_frames / s.buffer.channels.max(1) as usize;
        (frames as f64 / s.buffer.sample_rate as f64 * 1000.0) as u64
    }

    pub fn is_playing(&self) -> bool {
        self.state.lock().unwrap().playing
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_engine::decoder::AudioBuffer;

    fn silent_buffer(duration_ms: u64) -> AudioBuffer {
        let sample_rate = 44100u32;
        let channels = 2u16;
        let frames = (sample_rate as u64 * duration_ms / 1000) as usize;
        AudioBuffer {
            samples: vec![0.0f32; frames * channels as usize],
            sample_rate,
            channels,
            duration_ms,
        }
    }

    #[test]
    fn test_player_starts_paused() {
        let player = AudioPlayer::new(silent_buffer(5000));
        assert!(!player.is_playing());
        assert_eq!(player.position_ms(), 0);
    }

    #[test]
    fn test_seek_clamps_to_duration() {
        let player = AudioPlayer::new(silent_buffer(5000));
        player.seek(99999);
        assert!(player.position_ms() <= 5000);
    }

    #[test]
    fn test_seek_updates_position() {
        let player = AudioPlayer::new(silent_buffer(10000));
        player.seek(3000);
        let pos = player.position_ms();
        assert!(pos >= 2900 && pos <= 3100);
    }
}
