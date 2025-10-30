// Audio capture module
// Handles system audio capture and buffering

#![allow(dead_code)]
#![allow(unused_imports)]

mod cpal_device;
mod ring_buffer;

pub use cpal_device::CpalAudioDevice;
pub use ring_buffer::AudioRingBuffer;

use crate::error::AudioError;
use std::time::Instant;

/// Configuration for audio capture
#[derive(Debug, Clone, Copy)]
pub struct AudioConfig {
    /// Sample rate in Hz (e.g., 44100, 48000)
    pub sample_rate: u32,
    /// Number of audio channels (1 = mono, 2 = stereo)
    pub channels: u16,
    /// Buffer size in samples
    pub buffer_size: usize,
}

/// Trait for audio capture devices
///
/// Implementations provide platform-specific audio capture functionality.
/// The trait abstracts over different audio backends (CPAL, etc.)
///
/// # Examples
///
/// ```no_run
/// use crabmusic::audio::{AudioCaptureDevice, AudioBuffer};
///
/// fn capture_audio(device: &mut dyn AudioCaptureDevice) {
///     device.start_capture().expect("Failed to start capture");
///
///     while device.is_capturing() {
///         if let Some(buffer) = device.read_samples() {
///             println!("Captured {} samples", buffer.len());
///         }
///     }
///
///     device.stop_capture().expect("Failed to stop capture");
/// }
/// ```
pub trait AudioCaptureDevice {
    /// Start capturing audio from the device
    ///
    /// # Errors
    /// Returns `AudioError::DeviceNotAvailable` if device cannot be opened
    /// Returns `AudioError::StreamError` if stream cannot be started
    fn start_capture(&mut self) -> Result<(), AudioError>;

    /// Stop capturing audio
    ///
    /// # Errors
    /// Returns `AudioError::StreamError` if stream cannot be stopped
    fn stop_capture(&mut self) -> Result<(), AudioError>;

    /// Check if currently capturing audio
    fn is_capturing(&self) -> bool;

    /// Read captured audio samples (non-blocking)
    ///
    /// Returns None if no samples are available
    fn read_samples(&mut self) -> Option<AudioBuffer>;

    /// Get the current audio configuration
    fn get_config(&self) -> AudioConfig;
}

/// Audio buffer containing captured samples
///
/// Stores raw audio samples with metadata about sample rate and channel count.
/// Used to pass audio data between capture and DSP processing stages.
///
/// # Examples
///
/// ```
/// use crabmusic::audio::AudioBuffer;
///
/// let buffer = AudioBuffer::new(1024, 44100, 2);
/// assert_eq!(buffer.sample_rate, 44100);
/// assert_eq!(buffer.channels, 2);
/// assert!(buffer.is_empty());
/// ```
#[derive(Clone, Debug)]
pub struct AudioBuffer {
    /// Raw audio sample data (32-bit float for DSP precision)
    /// Mono audio: single channel
    /// Stereo audio: interleaved [L, R, L, R, ...]
    pub samples: Vec<f32>,

    /// Sampling rate in Hz (typically 44100 or 48000)
    pub sample_rate: u32,

    /// Number of audio channels (mono=1, stereo=2)
    pub channels: u16,

    /// Capture timestamp for latency tracking
    pub timestamp: Instant,
}

impl AudioBuffer {
    /// Create a new audio buffer
    ///
    /// # Arguments
    /// * `capacity` - Initial capacity for sample storage
    /// * `sample_rate` - Sampling rate in Hz
    /// * `channels` - Number of audio channels
    ///
    /// # Returns
    /// A new AudioBuffer instance with empty samples
    ///
    /// # Examples
    ///
    /// ```
    /// use crabmusic::audio::AudioBuffer;
    ///
    /// let buffer = AudioBuffer::new(1024, 44100, 2);
    /// assert_eq!(buffer.len(), 0);
    /// assert!(buffer.is_empty());
    /// ```
    pub fn new(capacity: usize, sample_rate: u32, channels: u16) -> Self {
        Self {
            samples: Vec::with_capacity(capacity),
            sample_rate,
            channels,
            timestamp: Instant::now(),
        }
    }

    /// Create an audio buffer with pre-filled samples
    ///
    /// # Arguments
    /// * `samples` - Audio sample data
    /// * `sample_rate` - Sampling rate in Hz
    /// * `channels` - Number of audio channels
    ///
    /// # Examples
    ///
    /// ```
    /// use crabmusic::audio::AudioBuffer;
    ///
    /// let samples = vec![0.1, 0.2, 0.3, 0.4];
    /// let buffer = AudioBuffer::with_samples(samples, 44100, 2);
    /// assert_eq!(buffer.len(), 4);
    /// ```
    pub fn with_samples(samples: Vec<f32>, sample_rate: u32, channels: u16) -> Self {
        Self {
            samples,
            sample_rate,
            channels,
            timestamp: Instant::now(),
        }
    }

    /// Get the number of samples in the buffer
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Get the duration of audio in seconds
    ///
    /// # Examples
    ///
    /// ```
    /// use crabmusic::audio::AudioBuffer;
    ///
    /// // 1 second of stereo audio at 44100 Hz = 88200 samples
    /// let samples = vec![0.0; 88200];
    /// let buffer = AudioBuffer::with_samples(samples, 44100, 2);
    /// assert!((buffer.duration_secs() - 1.0).abs() < 0.001);
    /// ```
    pub fn duration_secs(&self) -> f64 {
        let frames = self.samples.len() / self.channels as usize;
        frames as f64 / self.sample_rate as f64
    }

    /// Clear all samples from the buffer
    pub fn clear(&mut self) {
        self.samples.clear();
        self.timestamp = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_buffer_creation() {
        let buffer = AudioBuffer::new(1024, 44100, 2);
        assert_eq!(buffer.sample_rate, 44100);
        assert_eq!(buffer.channels, 2);
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_audio_buffer_with_samples() {
        let samples = vec![0.1, 0.2, 0.3, 0.4];
        let buffer = AudioBuffer::with_samples(samples.clone(), 44100, 2);
        assert_eq!(buffer.len(), 4);
        assert!(!buffer.is_empty());
        assert_eq!(buffer.samples, samples);
    }

    #[test]
    fn test_audio_buffer_duration() {
        // 1 second of stereo audio at 44100 Hz = 88200 samples
        let samples = vec![0.0; 88200];
        let buffer = AudioBuffer::with_samples(samples, 44100, 2);
        assert!((buffer.duration_secs() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_audio_buffer_duration_mono() {
        // 1 second of mono audio at 44100 Hz = 44100 samples
        let samples = vec![0.0; 44100];
        let buffer = AudioBuffer::with_samples(samples, 44100, 1);
        assert!((buffer.duration_secs() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_audio_buffer_clear() {
        let samples = vec![0.1, 0.2, 0.3, 0.4];
        let mut buffer = AudioBuffer::with_samples(samples, 44100, 2);
        buffer.clear();
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn test_audio_config_creation() {
        let config = AudioConfig {
            sample_rate: 48000,
            channels: 1,
            buffer_size: 512,
        };
        assert_eq!(config.sample_rate, 48000);
        assert_eq!(config.channels, 1);
        assert_eq!(config.buffer_size, 512);
    }

    #[test]
    fn test_audio_buffer_capacity() {
        let buffer = AudioBuffer::new(2048, 44100, 2);
        assert!(buffer.samples.capacity() >= 2048);
    }
}
