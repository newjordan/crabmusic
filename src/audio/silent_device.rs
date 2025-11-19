// Silent audio capture implementation (fallback)

use super::{AudioBuffer, AudioCaptureDevice, AudioConfig, AudioRingBuffer};
use crate::error::AudioError;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tracing::{info, warn};

/// Silent audio capture device
///
/// A fallback device that generates silent (zero) audio samples when no physical
/// audio device is available. This allows the application to run in "visualizer only"
/// mode or test mode on systems without audio hardware (e.g., WSL, headless servers).
pub struct SilentAudioDevice {
    /// Audio configuration
    config: AudioConfig,
    /// Ring buffer for passing samples to DSP thread
    ring_buffer: Arc<AudioRingBuffer>,
    /// Flag indicating if currently capturing
    is_capturing: Arc<AtomicBool>,
    /// Thread handle for the silence generator
    generator_thread: Option<thread::JoinHandle<()>>,
}

impl SilentAudioDevice {
    /// Create a new silent audio device
    ///
    /// # Arguments
    /// * `ring_buffer` - Shared ring buffer for audio samples
    /// * `sample_rate` - Target sample rate (default: 44100)
    /// * `channels` - Target channel count (default: 2)
    pub fn new(
        ring_buffer: Arc<AudioRingBuffer>,
        sample_rate: u32,
        channels: u16,
    ) -> Result<Self, AudioError> {
        let config = AudioConfig {
            sample_rate,
            channels,
            buffer_size: 1024,
        };

        Ok(Self {
            config,
            ring_buffer,
            is_capturing: Arc::new(AtomicBool::new(false)),
            generator_thread: None,
        })
    }
}

impl AudioCaptureDevice for SilentAudioDevice {
    fn start_capture(&mut self) -> Result<(), AudioError> {
        if self.is_capturing.load(Ordering::Relaxed) {
            return Ok(());
        }

        info!("Starting silent audio generator (fallback mode)");

        let is_capturing = Arc::clone(&self.is_capturing);
        let ring_buffer = Arc::clone(&self.ring_buffer);
        let sample_rate = self.config.sample_rate;
        let channels = self.config.channels;
        let buffer_size = self.config.buffer_size;

        is_capturing.store(true, Ordering::Relaxed);

        // Spawn a thread to generate silence at the appropriate rate
        let thread_handle = thread::spawn(move || {
            let interval = Duration::from_secs_f64(buffer_size as f64 / sample_rate as f64);
            let mut next_frame_time = Instant::now();

            while is_capturing.load(Ordering::Relaxed) {
                // Create a buffer of silence
                // We add a tiny bit of random noise so it's not absolute zero,
                // which can sometimes cause issues with some DSP algorithms expecting non-zero input
                // or just to make the visualizer look "alive" but idle.
                let samples: Vec<f32> = (0..buffer_size * channels as usize)
                    .map(|_| (rand::random::<f32>() - 0.5) * 0.001) // Very low noise floor
                    .collect();

                let buffer = AudioBuffer::with_samples(samples, sample_rate, channels);
                ring_buffer.push(buffer);

                // Sleep to simulate real-time audio timing
                next_frame_time += interval;
                let now = Instant::now();
                if next_frame_time > now {
                    thread::sleep(next_frame_time - now);
                } else {
                    // If we're falling behind, reset the clock to avoid burst catch-up
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
        
        info!("Silent audio generator stopped");
        Ok(())
    }

    fn is_capturing(&self) -> bool {
        self.is_capturing.load(Ordering::Relaxed)
    }

    fn read_samples(&mut self) -> Option<AudioBuffer> {
        self.ring_buffer.pop()
    }

    fn get_config(&self) -> AudioConfig {
        self.config
    }
}

impl Drop for SilentAudioDevice {
    fn drop(&mut self) {
        if self.is_capturing() {
            let _ = self.stop_capture();
        }
    }
}
