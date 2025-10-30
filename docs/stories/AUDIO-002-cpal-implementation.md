# [AUDIO-002] Implement CPAL Audio Capture

**Epic**: Audio Capture System
**Priority**: P0 (Blocking)
**Estimated Effort**: 1-2 days
**Status**: Not Started

---

## Description

Implement the AudioCaptureDevice trait using CPAL (Cross-Platform Audio Library) to capture system audio output. This is the entry point for all audio data into the application.

**Agent Instructions**: Implement a working CPAL-based audio capture that:
- Detects and opens the default system audio output device
- Starts capturing audio samples in a callback
- Handles stereo-to-mono conversion if needed
- Writes samples to ring buffer for DSP consumption

---

## Acceptance Criteria

- [ ] CpalAudioDevice struct implements AudioCaptureDevice trait
- [ ] Successfully opens default audio output device
- [ ] Audio callback receives samples in real-time
- [ ] Stereo audio converted to mono (average channels)
- [ ] Sample rate configuration respected (default 44100 Hz)
- [ ] Samples written to ring buffer without blocking
- [ ] Graceful error handling for device unavailable
- [ ] Platform-specific backends work (ALSA/PulseAudio on Linux, CoreAudio on macOS)
- [ ] Unit tests with mock audio device
- [ ] Integration test logs captured audio statistics

---

## Technical Approach

### CPAL Device Initialization

Reference: **docs/architecture.md - Audio Capture Component**

```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

pub struct CpalAudioDevice {
    stream: Option<cpal::Stream>,
    config: cpal::StreamConfig,
    sample_rate: u32,
    ring_buffer: Arc<RingBuffer>, // Shared with DSP
}

impl CpalAudioDevice {
    pub fn new(ring_buffer: Arc<RingBuffer>) -> Result<Self> {
        let host = cpal::default_host();

        // Get default output device (system audio)
        let device = host.default_output_device()
            .ok_or(AudioError::DeviceNotAvailable)?;

        let config = device.default_output_config()?;
        // ... setup stream
    }
}
```

### Audio Callback

The callback runs on high-priority audio thread:
```rust
let stream = device.build_input_stream(
    &config.into(),
    move |data: &[f32], _: &cpal::InputCallbackInfo| {
        // Convert stereo to mono if needed
        let mono_samples = if channels == 2 {
            data.chunks(2).map(|ch| (ch[0] + ch[1]) / 2.0).collect()
        } else {
            data.to_vec()
        };

        // Write to ring buffer (lock-free)
        ring_buffer.write(mono_samples);
    },
    |err| {
        // Error callback
        eprintln!("Audio stream error: {}", err);
    },
)?;
```

### Platform Considerations

- **Linux**: Prioritize PipeWire, fall back to PulseAudio/ALSA
- **macOS**: CoreAudio backend (CPAL handles this)
- **Windows**: WASAPI backend (post-MVP testing)

---

## Dependencies

- **Depends on**:
  - FOUND-001 (project structure exists)
  - AUDIO-001 (AudioCaptureDevice trait defined)
  - AUDIO-003 (ring buffer implemented)
- **Blocks**: DSP-001 (need audio samples)

---

## Architecture References

- **Component Spec**: docs/architecture.md - "Audio Capture Component"
- **Tech Stack**: docs/architecture.md - CPAL 0.15+ entry
- **Error Handling**: docs/architecture.md - "Audio Capture Errors"

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_stereo_to_mono_conversion() {
        // Test stereo averaging logic
    }

    #[test]
    fn test_device_initialization_fails_gracefully() {
        // Mock CPAL to test error handling
    }
}
```

### Integration Tests

Create `tests/audio_capture_test.rs`:
- Initialize audio device
- Capture 1 second of audio
- Validate sample rate, buffer sizes
- Log statistics (min/max amplitude, etc.)

### Manual Testing

Run application and verify:
- Audio captured while music plays
- No audio dropouts or glitches
- CPU usage reasonable (<5% for audio thread)

---

## Notes for AI Agent

**CRITICAL**: The audio callback runs on a high-priority thread - it MUST NOT:
- ❌ Allocate memory
- ❌ Acquire locks (use lock-free ring buffer)
- ❌ Do I/O operations
- ❌ Perform expensive computations

**Performance Target**: Audio callback should complete in < 1ms

**Error Handling**: If audio device fails to open:
1. Try 3 times with exponential backoff
2. Print helpful error message (see architecture docs)
3. Exit gracefully

**Testing Note**: CPAL testing is challenging without real audio hardware. Focus on:
- Logic testing with synthetic data
- Error path testing with mocks
- Manual validation with real audio

**Success Indicator**: Can run app, play music, and see audio samples flowing through ring buffer (add debug logging)
