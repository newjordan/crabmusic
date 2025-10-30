# [AUDIO-001] Define Audio Capture Interface

**Epic**: Audio Capture System
**Priority**: P0 (Blocking)
**Estimated Effort**: 0.5-1 day
**Status**: Not Started

---

## Description

Define the core audio capture interface and data structures that will be used throughout the audio pipeline. This story establishes the contract that all audio capture implementations must follow.

**Agent Instructions**: Create the AudioCaptureDevice trait and AudioBuffer struct that:
- Define clear interface for starting/stopping audio capture
- Provide non-blocking sample reading
- Include configuration access for sample rate and channels
- Create AudioBuffer struct for passing audio data between components
- Add comprehensive documentation and examples

---

## Acceptance Criteria

- [ ] AudioCaptureDevice trait defined with all required methods
- [ ] AudioBuffer struct with fields: samples, sample_rate, channels, timestamp
- [ ] AudioConfig struct for device configuration
- [ ] All public APIs have doc comments with examples
- [ ] AudioBuffer implements Clone and Debug traits
- [ ] Unit tests for AudioBuffer creation and basic operations
- [ ] Integration with error types from src/error.rs
- [ ] Code follows docs/architecture/coding-standards.md
- [ ] All tests pass with `cargo test`
- [ ] No clippy warnings with `cargo clippy`

---

## Technical Approach

### AudioCaptureDevice Trait

Reference: **docs/architecture.md - Audio Capture Component**

```rust
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
```

### AudioBuffer Struct

```rust
/// Audio buffer containing captured samples
///
/// Stores raw audio samples with metadata about sample rate and channel count.
/// Used to pass audio data between capture and DSP processing stages.
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
```

### File Organization

Update `src/audio/mod.rs` to include the trait and struct definitions above, replacing the TODO placeholders from FOUND-001.

---

## Dependencies

- **Depends on**:
  - FOUND-001 (project structure exists)
- **Blocks**: 
  - AUDIO-002 (CPAL implementation needs trait)
  - AUDIO-003 (ring buffer needs AudioBuffer)

---

## Architecture References

- **Component Spec**: docs/architecture.md - "Audio Capture Component"
- **Source Tree**: docs/architecture/source-tree.md - "src/audio/" section
- **Coding Standards**: docs/architecture/coding-standards.md

---

## Testing Requirements

### Unit Tests

```rust
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
        let samples = vec![0.0; 88200]; // 1 second of stereo audio at 44100 Hz
        let buffer = AudioBuffer::with_samples(samples, 44100, 2);
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
}
```

---

## Dev Notes

- AudioBuffer uses `Vec<f32>` for samples (32-bit float is standard for DSP)
- Timestamp field enables latency tracking and synchronization
- Trait design allows for mock implementations in tests
- Non-blocking `read_samples()` prevents audio thread from blocking
- AudioConfig is Copy to avoid unnecessary allocations

---

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Code reviewed against coding standards
- [ ] All unit tests passing
- [ ] Documentation complete with examples
- [ ] No compiler warnings
- [ ] Clippy passes with no warnings
- [ ] Code formatted with rustfmt

