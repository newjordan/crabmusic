//! File-backed audio capture.
//!
//! Plays a WAV file into the same ring buffer live devices use, looping so a
//! visualizer can run with no microphone or system-audio device.

use super::{read_wav, AudioBuffer, AudioCaptureDevice, AudioConfig, AudioError, AudioRingBuffer};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use tracing::info;

/// Audio capture device that streams a WAV file in real time.
pub struct FileAudioDevice {
    config: AudioConfig,
    path: PathBuf,
    samples: Arc<Vec<f32>>,
    ring_buffer: Arc<AudioRingBuffer>,
    is_capturing: Arc<AtomicBool>,
    generator_thread: Option<JoinHandle<()>>,
}

impl FileAudioDevice {
    /// Open a WAV file and prepare to stream it.
    ///
    /// The file is looped for as long as capture is running.
    pub fn open(
        path: impl AsRef<Path>,
        ring_buffer: Arc<AudioRingBuffer>,
    ) -> Result<Self, AudioError> {
        let path = path.as_ref();
        let wav = read_wav(path)?;
        if wav.samples.is_empty() {
            return Err(AudioError::FileError(format!(
                "{} contains no audio samples",
                path.display()
            )));
        }

        let buffer_size = 1024.min(wav.samples.len() / wav.channels as usize).max(1);
        Ok(Self {
            config: AudioConfig {
                sample_rate: wav.sample_rate,
                channels: wav.channels,
                buffer_size,
            },
            path: path.to_path_buf(),
            samples: Arc::new(wav.samples),
            ring_buffer,
            is_capturing: Arc::new(AtomicBool::new(false)),
            generator_thread: None,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Copy `frame_count` frames from `samples`, wrapping to the start of the file.
pub fn looping_frames(
    samples: &[f32],
    channels: u16,
    frame_cursor: &mut usize,
    frame_count: usize,
) -> Vec<f32> {
    let channels = channels.max(1) as usize;
    let total_frames = (samples.len() / channels).max(1);
    let mut out = Vec::with_capacity(frame_count * channels);
    for _ in 0..frame_count {
        if *frame_cursor >= total_frames {
            *frame_cursor = 0;
        }
        let start = *frame_cursor * channels;
        let end = (start + channels).min(samples.len());
        out.extend_from_slice(&samples[start..end]);
        if end - start < channels {
            out.resize(out.len() + (channels - (end - start)), 0.0);
        }
        *frame_cursor += 1;
    }
    out
}

impl AudioCaptureDevice for FileAudioDevice {
    fn start_capture(&mut self) -> Result<(), AudioError> {
        if self.is_capturing.load(Ordering::Relaxed) {
            return Ok(());
        }

        info!(
            "Playing {} ({} Hz, {} ch) instead of a live audio device",
            self.path.display(),
            self.config.sample_rate,
            self.config.channels
        );

        let is_capturing = Arc::clone(&self.is_capturing);
        let ring_buffer = Arc::clone(&self.ring_buffer);
        let samples = Arc::clone(&self.samples);
        let sample_rate = self.config.sample_rate;
        let channels = self.config.channels;
        let buffer_size = self.config.buffer_size;
        is_capturing.store(true, Ordering::Relaxed);

        let thread_handle = thread::spawn(move || {
            let interval = Duration::from_secs_f64(buffer_size as f64 / sample_rate as f64);
            let mut next_frame_time = Instant::now();
            let mut cursor = 0usize;

            while is_capturing.load(Ordering::Relaxed) {
                let chunk = looping_frames(&samples, channels, &mut cursor, buffer_size);
                let buffer = AudioBuffer::with_samples(chunk, sample_rate, channels);
                ring_buffer.push(buffer);

                next_frame_time += interval;
                let now = Instant::now();
                if next_frame_time > now {
                    thread::sleep(next_frame_time - now);
                } else {
                    next_frame_time = now;
                }
            }
        });

        self.generator_thread = Some(thread_handle);
        Ok(())
    }

    fn stop_capture(&mut self) -> Result<(), AudioError> {
        self.is_capturing.store(false, Ordering::Relaxed);
        if let Some(handle) = self.generator_thread.take() {
            let _ = handle.join();
        }
        info!("Stopped file playback for {}", self.path.display());
        Ok(())
    }

    fn is_capturing(&self) -> bool {
        self.is_capturing.load(Ordering::Relaxed)
    }

    fn read_samples(&mut self) -> Option<AudioBuffer> {
        self.ring_buffer.pop()
    }

    fn read_latest_samples(&mut self) -> (Option<AudioBuffer>, usize) {
        self.ring_buffer.pop_latest()
    }

    fn get_config(&self) -> AudioConfig {
        self.config
    }
}

impl Drop for FileAudioDevice {
    fn drop(&mut self) {
        if self.is_capturing() {
            let _ = self.stop_capture();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::{write_wav, WavData};

    #[test]
    fn looping_frames_wraps_to_the_start() {
        let samples = vec![0.0, 1.0, 2.0, 3.0];
        let mut cursor = 3;
        let chunk = looping_frames(&samples, 1, &mut cursor, 3);
        assert_eq!(chunk, vec![3.0, 0.0, 1.0]);
        assert_eq!(cursor, 2);
    }

    #[test]
    fn file_device_streams_a_wav_without_a_live_input() {
        let path =
            std::env::temp_dir().join(format!("crabmusic-file-input-{}.wav", std::process::id()));
        write_wav(
            &path,
            &WavData {
                sample_rate: 8_000,
                channels: 1,
                samples: vec![0.5; 8_000],
            },
        )
        .unwrap();

        let ring = Arc::new(AudioRingBuffer::new(8));
        let mut device = FileAudioDevice::open(&path, ring).unwrap();
        device.start_capture().unwrap();

        let started = Instant::now();
        let mut received = None;
        while started.elapsed() < Duration::from_secs(2) {
            if let Some(buffer) = device.read_samples() {
                received = Some(buffer);
                break;
            }
            thread::sleep(Duration::from_millis(5));
        }
        device.stop_capture().unwrap();
        let _ = std::fs::remove_file(&path);

        let buffer = received.expect("file capture should produce samples");
        assert_eq!(buffer.sample_rate, 8_000);
        assert_eq!(buffer.channels, 1);
        assert!(buffer.samples.iter().any(|sample| *sample > 0.4));
    }
}
