// CPAL-based audio capture implementation

use super::{AudioBuffer, AudioCaptureDevice, AudioConfig, AudioRingBuffer};
use crate::error::AudioError;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// CPAL-based audio capture device
///
/// Captures system audio output using the CPAL library with platform-specific
/// backends (WASAPI on Windows, CoreAudio on macOS, ALSA/PulseAudio on Linux).
///
/// # Examples
///
/// ```no_run
/// use crabmusic::audio::{AudioCaptureDevice, CpalAudioDevice, AudioRingBuffer};
/// use std::sync::Arc;
///
/// let ring_buffer = Arc::new(AudioRingBuffer::new(10));
/// let mut device = CpalAudioDevice::new(ring_buffer).expect("Failed to create device");
/// device.start_capture().expect("Failed to start capture");
/// ```
pub struct CpalAudioDevice {
    /// The active audio stream (None if not started)
    stream: Option<cpal::Stream>,
    /// Audio configuration
    config: AudioConfig,
    /// Ring buffer for passing samples to DSP thread
    ring_buffer: Arc<AudioRingBuffer>,
    /// Flag indicating if currently capturing
    is_capturing: Arc<AtomicBool>,
    /// Optional device name to use (None = default)
    device_name: Option<String>,
}

impl CpalAudioDevice {
    /// Create a new CPAL audio capture device
    ///
    /// # Arguments
    /// * `ring_buffer` - Shared ring buffer for audio samples
    ///
    /// # Errors
    /// Returns `AudioError::DeviceNotAvailable` if no audio device found
    /// Returns `AudioError::ConfigError` if device configuration fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use crabmusic::audio::{CpalAudioDevice, AudioRingBuffer};
    /// use std::sync::Arc;
    ///
    /// let ring_buffer = Arc::new(AudioRingBuffer::new(10));
    /// let device = CpalAudioDevice::new(ring_buffer)?;
    /// # Ok::<(), crabmusic::error::AudioError>(())
    /// ```
    pub fn new(ring_buffer: Arc<AudioRingBuffer>) -> Result<Self, AudioError> {
        Self::new_with_device(ring_buffer, None)
    }

    /// Create a new CPAL audio capture device with a specific device name
    ///
    /// # Arguments
    /// * `ring_buffer` - Shared ring buffer for audio samples
    /// * `device_name` - Optional device name to use (None = default input device)
    ///
    /// # Errors
    /// Returns `AudioError::DeviceNotAvailable` if no audio device found
    /// Returns `AudioError::ConfigError` if device configuration fails
    pub fn new_with_device(
        ring_buffer: Arc<AudioRingBuffer>,
        device_name: Option<String>,
    ) -> Result<Self, AudioError> {
        info!("Initializing CPAL audio device");

        let host = cpal::default_host();
        debug!("Using audio host: {:?}", host.id());

        // Get the specified device or default input device
        let device = if let Some(ref name) = device_name {
            info!("Looking for device: {}", name);
            Self::find_device_by_name(&host, name)?
        } else {
            host.default_input_device()
                .ok_or(AudioError::DeviceNotAvailable)?
        };

        info!(
            "Using audio device: {}",
            device.name().unwrap_or_else(|_| "Unknown".to_string())
        );

        // Get default input config
        let supported_config = device.default_input_config().map_err(|e| {
            error!("Failed to get default input config: {}", e);
            AudioError::ConfigError(format!("Failed to get device config: {}", e))
        })?;

        debug!("Supported config: {:?}", supported_config);

        let config = AudioConfig {
            sample_rate: supported_config.sample_rate().0,
            channels: supported_config.channels(),
            buffer_size: 1024, // Default buffer size
        };

        info!(
            "Audio config: {} Hz, {} channels, {} buffer size",
            config.sample_rate, config.channels, config.buffer_size
        );

        Ok(Self {
            stream: None,
            config,
            ring_buffer,
            is_capturing: Arc::new(AtomicBool::new(false)),
            device_name,
        })
    }

    /// Find a device by name
    fn find_device_by_name(
        host: &cpal::Host,
        name: &str,
    ) -> Result<cpal::Device, AudioError> {
        // Try input devices first
        if let Ok(devices) = host.input_devices() {
            for device in devices {
                if let Ok(device_name) = device.name() {
                    if device_name.to_lowercase().contains(&name.to_lowercase()) {
                        info!("Found matching input device: {}", device_name);
                        return Ok(device);
                    }
                }
            }
        }

        // If not found, try output devices (for loopback)
        if let Ok(devices) = host.output_devices() {
            for device in devices {
                if let Ok(device_name) = device.name() {
                    if device_name.to_lowercase().contains(&name.to_lowercase()) {
                        info!("Found matching output device: {}", device_name);
                        return Ok(device);
                    }
                }
            }
        }

        Err(AudioError::DeviceNotAvailable)
    }

    /// Convert stereo samples to mono by averaging channels
    fn stereo_to_mono(samples: &[f32]) -> Vec<f32> {
        samples
            .chunks_exact(2)
            .map(|chunk| (chunk[0] + chunk[1]) / 2.0)
            .collect()
    }
}

impl AudioCaptureDevice for CpalAudioDevice {
    fn start_capture(&mut self) -> Result<(), AudioError> {
        if self.is_capturing.load(Ordering::Relaxed) {
            warn!("Audio capture already started");
            return Ok(());
        }

        info!("Starting audio capture");

        let host = cpal::default_host();

        // Get the device (either by name or default)
        let device = if let Some(ref name) = self.device_name {
            Self::find_device_by_name(&host, name)?
        } else {
            host.default_input_device()
                .ok_or(AudioError::DeviceNotAvailable)?
        };

        let config: cpal::StreamConfig = cpal::StreamConfig {
            channels: self.config.channels,
            sample_rate: cpal::SampleRate(self.config.sample_rate),
            buffer_size: cpal::BufferSize::Fixed(self.config.buffer_size as u32),
        };

        let ring_buffer = Arc::clone(&self.ring_buffer);
        let is_capturing = Arc::clone(&self.is_capturing);
        let sample_rate = self.config.sample_rate;
        let channels = self.config.channels;

        // Build input stream with f32 samples
        let stream = device
            .build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    // Convert stereo to mono if needed
                    let mono_samples = if channels == 2 {
                        Self::stereo_to_mono(data)
                    } else {
                        data.to_vec()
                    };

                    // Create audio buffer
                    let buffer = AudioBuffer::with_samples(mono_samples, sample_rate, 1);

                    // Push to ring buffer (non-blocking)
                    ring_buffer.push(buffer);
                },
                move |err| {
                    error!("Audio stream error: {}", err);
                    is_capturing.store(false, Ordering::Relaxed);
                },
                None, // No timeout
            )
            .map_err(|e| {
                error!("Failed to build input stream: {}", e);
                AudioError::StreamError(format!("Failed to build stream: {}", e))
            })?;

        // Start the stream
        stream.play().map_err(|e| {
            error!("Failed to start stream: {}", e);
            AudioError::StreamError(format!("Failed to start stream: {}", e))
        })?;

        self.stream = Some(stream);
        self.is_capturing.store(true, Ordering::Relaxed);

        info!("Audio capture started successfully");
        Ok(())
    }

    fn stop_capture(&mut self) -> Result<(), AudioError> {
        if !self.is_capturing.load(Ordering::Relaxed) {
            warn!("Audio capture not started");
            return Ok(());
        }

        info!("Stopping audio capture");

        if let Some(stream) = self.stream.take() {
            stream.pause().map_err(|e| {
                error!("Failed to pause stream: {}", e);
                AudioError::StreamError(format!("Failed to pause stream: {}", e))
            })?;
        }

        self.is_capturing.store(false, Ordering::Relaxed);

        info!("Audio capture stopped");
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

impl Drop for CpalAudioDevice {
    fn drop(&mut self) {
        if self.is_capturing() {
            let _ = self.stop_capture();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stereo_to_mono_conversion() {
        let stereo = vec![0.5, 0.3, 0.7, 0.1, 0.2, 0.8];
        let mono = CpalAudioDevice::stereo_to_mono(&stereo);

        assert_eq!(mono.len(), 3);
        assert!((mono[0] - 0.4).abs() < 0.001); // (0.5 + 0.3) / 2
        assert!((mono[1] - 0.4).abs() < 0.001); // (0.7 + 0.1) / 2
        assert!((mono[2] - 0.5).abs() < 0.001); // (0.2 + 0.8) / 2
    }

    #[test]
    fn test_stereo_to_mono_empty() {
        let stereo: Vec<f32> = vec![];
        let mono = CpalAudioDevice::stereo_to_mono(&stereo);
        assert_eq!(mono.len(), 0);
    }

    #[test]
    fn test_stereo_to_mono_single_frame() {
        let stereo = vec![1.0, -1.0];
        let mono = CpalAudioDevice::stereo_to_mono(&stereo);
        assert_eq!(mono.len(), 1);
        assert!((mono[0] - 0.0).abs() < 0.001); // (1.0 + -1.0) / 2
    }

    // Note: Testing actual device initialization requires audio hardware
    // and is better suited for integration tests
}
