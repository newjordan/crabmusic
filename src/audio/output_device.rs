// Audio output/playback module
// Handles audio playback for monitoring what's being visualized

use crate::audio::{AudioBuffer, AudioConfig};
use crate::error::AudioError;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info, warn};

/// Audio output device for playback
///
/// Plays back audio from a shared buffer, allowing users to hear
/// what they're visualizing in real-time.
pub struct AudioOutputDevice {
    /// The active audio output stream
    stream: Option<cpal::Stream>,
    /// Audio configuration
    config: AudioConfig,
    /// Flag indicating if playback is active
    is_playing: Arc<AtomicBool>,
    /// Shared buffer for audio samples to play
    playback_buffer: Arc<Mutex<Vec<f32>>>,
    /// Optional device name to use (None = default)
    device_name: Option<String>,
}

impl AudioOutputDevice {
    /// Create a new audio output device
    ///
    /// # Errors
    /// Returns `AudioError::DeviceNotAvailable` if no output device found
    /// Returns `AudioError::ConfigError` if device configuration fails
    pub fn new() -> Result<Self, AudioError> {
        Self::new_with_device(None)
    }

    /// Create a new audio output device with a specific device name
    ///
    /// # Arguments
    /// * `device_name` - Optional device name to use (None = default output device)
    ///
    /// # Errors
    /// Returns `AudioError::DeviceNotAvailable` if no audio device found
    /// Returns `AudioError::ConfigError` if device configuration fails
    pub fn new_with_device(device_name: Option<String>) -> Result<Self, AudioError> {
        info!("Initializing audio output device");

        let host = cpal::default_host();
        debug!("Using audio host: {:?}", host.id());

        // Get the specified device or default output device
        let device = if let Some(ref name) = device_name {
            info!("Looking for output device: {}", name);
            Self::find_device_by_name(&host, name)?
        } else {
            host.default_output_device()
                .ok_or(AudioError::DeviceNotAvailable)?
        };

        info!(
            "Using output device: {}",
            device.name().unwrap_or_else(|_| "Unknown".to_string())
        );

        // Get default output config
        let supported_config = device.default_output_config().map_err(|e| {
            error!("Failed to get default output config: {}", e);
            AudioError::ConfigError(format!("Failed to get device config: {}", e))
        })?;

        let sample_rate = supported_config.sample_rate().0;
        let channels = supported_config.channels();

        info!("Output config: {} Hz, {} channels", sample_rate, channels);

        let config = AudioConfig {
            sample_rate,
            channels,
            buffer_size: 1024,
        };

        Ok(Self {
            stream: None,
            config,
            is_playing: Arc::new(AtomicBool::new(false)),
            playback_buffer: Arc::new(Mutex::new(Vec::new())),
            device_name,
        })
    }

    /// Find an output device by name
    fn find_device_by_name(host: &cpal::Host, name: &str) -> Result<cpal::Device, AudioError> {
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

        error!("Output device not found: {}", name);
        Err(AudioError::DeviceNotAvailable)
    }

    /// Start audio playback
    ///
    /// # Errors
    /// Returns `AudioError::DeviceNotAvailable` if device cannot be opened
    /// Returns `AudioError::StreamError` if stream cannot be started
    pub fn start_playback(&mut self) -> Result<(), AudioError> {
        if self.is_playing.load(Ordering::Relaxed) {
            warn!("Audio playback already started");
            return Ok(());
        }

        info!("Starting audio playback");

        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or(AudioError::DeviceNotAvailable)?;

        let config: cpal::StreamConfig = cpal::StreamConfig {
            channels: self.config.channels,
            sample_rate: cpal::SampleRate(self.config.sample_rate),
            buffer_size: cpal::BufferSize::Fixed(self.config.buffer_size as u32),
        };

        let playback_buffer = Arc::clone(&self.playback_buffer);
        let is_playing = Arc::clone(&self.is_playing);

        // Build output stream
        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let buffer = playback_buffer.lock().unwrap();

                    // Fill output buffer with samples from playback buffer
                    for (i, sample) in data.iter_mut().enumerate() {
                        *sample = if i < buffer.len() {
                            buffer[i]
                        } else {
                            0.0 // Silence if no data available
                        };
                    }
                },
                move |err| {
                    error!("Audio output stream error: {}", err);
                    is_playing.store(false, Ordering::Relaxed);
                },
                None, // No timeout
            )
            .map_err(|e| {
                error!("Failed to build output stream: {}", e);
                AudioError::StreamError(format!("Failed to build stream: {}", e))
            })?;

        // Start the stream
        stream.play().map_err(|e| {
            error!("Failed to start output stream: {}", e);
            AudioError::StreamError(format!("Failed to start stream: {}", e))
        })?;

        self.is_playing.store(true, Ordering::Relaxed);
        self.stream = Some(stream);

        info!("Audio playback started successfully");
        Ok(())
    }

    /// Stop audio playback
    ///
    /// # Errors
    /// Returns `AudioError::StreamError` if stream cannot be stopped
    pub fn stop_playback(&mut self) -> Result<(), AudioError> {
        if !self.is_playing.load(Ordering::Relaxed) {
            return Ok(());
        }

        info!("Stopping audio playback");

        if let Some(stream) = self.stream.take() {
            stream.pause().map_err(|e| {
                error!("Failed to stop output stream: {}", e);
                AudioError::StreamError(format!("Failed to stop stream: {}", e))
            })?;
        }

        self.is_playing.store(false, Ordering::Relaxed);
        info!("Audio playback stopped");
        Ok(())
    }

    /// Check if currently playing audio
    pub fn is_playing(&self) -> bool {
        self.is_playing.load(Ordering::Relaxed)
    }

    /// Write audio samples to the playback buffer
    ///
    /// This allows passing captured audio through to the output
    pub fn write_samples(&self, buffer: &AudioBuffer) {
        let mut playback = self.playback_buffer.lock().unwrap();
        playback.clear();
        playback.extend_from_slice(&buffer.samples);
    }

    /// Get the current audio configuration
    pub fn get_config(&self) -> AudioConfig {
        self.config
    }
}

impl Drop for AudioOutputDevice {
    fn drop(&mut self) {
        let _ = self.stop_playback();
    }
}
