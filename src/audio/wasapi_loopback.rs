// Windows WASAPI loopback capture for system audio
// This module provides native Windows loopback capture without virtual cables

#![cfg(windows)]

use crate::audio::{AudioBuffer, AudioCaptureDevice, AudioConfig, AudioRingBuffer};
use crate::error::AudioError;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use tracing::{debug, error, info, warn};
use wasapi::*;

/// WASAPI loopback audio capture device
///
/// Captures system audio output (what's playing through speakers) using
/// Windows WASAPI loopback mode. This works without any virtual audio cables.
pub struct WasapiLoopbackDevice {
    /// Ring buffer for captured audio samples
    ring_buffer: Arc<AudioRingBuffer>,
    /// Audio configuration
    config: AudioConfig,
    /// Flag indicating if capture is active
    is_capturing: Arc<AtomicBool>,
    /// Capture thread handle
    capture_thread: Option<thread::JoinHandle<()>>,
    /// Flag to signal thread shutdown
    should_stop: Arc<AtomicBool>,
}

impl WasapiLoopbackDevice {
    /// Create a new WASAPI loopback device
    ///
    /// # Arguments
    /// * `ring_buffer` - Shared ring buffer for audio samples
    ///
    /// # Returns
    /// A new WasapiLoopbackDevice instance
    ///
    /// # Errors
    /// Returns `AudioError::DeviceNotAvailable` if no default output device exists
    pub fn new(ring_buffer: Arc<AudioRingBuffer>) -> Result<Self, AudioError> {
        info!("Initializing WASAPI loopback device");

        // Initialize COM for this thread
        let hr = initialize_mta();
        if hr.is_err() {
            error!("Failed to initialize COM: {:?}", hr);
            return Err(AudioError::DeviceNotAvailable);
        }

        // Get the default output device (what we'll capture from)
        let device = get_default_device(&Direction::Render).map_err(|e| {
            error!("Failed to get default output device: {:?}", e);
            AudioError::DeviceNotAvailable
        })?;

        // Get the device's native format
        let audio_client = device.get_iaudioclient().map_err(|e| {
            error!("Failed to get audio client: {:?}", e);
            AudioError::DeviceNotAvailable
        })?;

        let wave_format = audio_client.get_mixformat().map_err(|e| {
            error!("Failed to get mix format: {:?}", e);
            AudioError::DeviceNotAvailable
        })?;

        info!(
            "WASAPI device format: {} Hz, {} channels, {} bits",
            wave_format.get_samplespersec(),
            wave_format.get_nchannels(),
            wave_format.get_bitspersample()
        );

        let config = AudioConfig {
            sample_rate: wave_format.get_samplespersec(),
            channels: wave_format.get_nchannels(),
            buffer_size: 1024, // Will be adjusted by WASAPI
        };

        Ok(Self {
            ring_buffer,
            config,
            is_capturing: Arc::new(AtomicBool::new(false)),
            capture_thread: None,
            should_stop: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Capture thread function
    fn capture_loop(
        ring_buffer: Arc<AudioRingBuffer>,
        is_capturing: Arc<AtomicBool>,
        should_stop: Arc<AtomicBool>,
        sample_rate: u32,
        _channels: u16,
    ) {
        // Initialize COM for this thread
        let hr = initialize_mta();
        if hr.is_err() {
            error!("Failed to initialize COM in capture thread: {:?}", hr);
            return;
        }

        // Get the default output device (for loopback)
        let device = match get_default_device(&Direction::Render) {
            Ok(d) => d,
            Err(e) => {
                error!("Failed to get default output device: {:?}", e);
                return;
            }
        };

        // Get audio client
        let mut audio_client = match device.get_iaudioclient() {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to get audio client: {:?}", e);
                return;
            }
        };

        // Get the mix format
        let wave_format = match audio_client.get_mixformat() {
            Ok(f) => f,
            Err(e) => {
                error!("Failed to get mix format: {:?}", e);
                return;
            }
        };

        let blockalign = wave_format.get_blockalign();
        debug!("Wave format: {:?}", wave_format);

        // Get device period
        let (_def_time, min_time) = match audio_client.get_device_period() {
            Ok(periods) => periods,
            Err(e) => {
                error!("Failed to get device period: {:?}", e);
                return;
            }
        };

        // Initialize audio client in loopback mode (shared mode with autoconvert)
        let mode = StreamMode::EventsShared {
            autoconvert: true,
            buffer_duration_hns: min_time,
        };

        if let Err(e) = audio_client.initialize_client(&wave_format, &Direction::Capture, &mode) {
            error!("Failed to initialize audio client: {:?}", e);
            return;
        }

        // Get the capture client
        let capture_client = match audio_client.get_audiocaptureclient() {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to get capture client: {:?}", e);
                return;
            }
        };

        // Get event handle for notifications
        let event_handle = match audio_client.set_get_eventhandle() {
            Ok(h) => h,
            Err(e) => {
                error!("Failed to get event handle: {:?}", e);
                return;
            }
        };

        // Start the audio client
        if let Err(e) = audio_client.start_stream() {
            error!("Failed to start audio stream: {:?}", e);
            return;
        }

        info!("WASAPI loopback capture started");
        is_capturing.store(true, Ordering::Relaxed);

        // Buffer for reading audio data
        let mut sample_queue: VecDeque<u8> = VecDeque::new();

        // Capture loop
        while !should_stop.load(Ordering::Relaxed) {
            // Wait for audio data (with timeout in milliseconds)
            if let Err(e) = event_handle.wait_for_event(1000) {
                warn!("Event wait error: {:?}", e);
                continue;
            }

            // Read the audio data into our queue
            if let Err(e) = capture_client.read_from_device_to_deque(&mut sample_queue) {
                warn!("Failed to read audio data: {:?}", e);
                continue;
            }

            // Convert bytes to f32 samples and push to ring buffer
            // Collect all available samples into a single buffer
            let mut all_samples = Vec::new();
            while sample_queue.len() >= blockalign as usize {
                let mut frame_bytes = vec![0u8; blockalign as usize];
                for byte in frame_bytes.iter_mut() {
                    *byte = sample_queue.pop_front().unwrap();
                }

                // Convert to f32 samples
                let samples = Self::convert_to_f32(&frame_bytes, &wave_format);

                // Convert stereo to mono if needed
                if samples.len() == 2 {
                    // Average left and right channels
                    all_samples.push((samples[0] + samples[1]) / 2.0);
                } else {
                    // Already mono or use first channel
                    all_samples.push(samples[0]);
                }
            }

            // Push all samples as a single AudioBuffer (like CPAL does)
            if !all_samples.is_empty() {
                let buffer = AudioBuffer::with_samples(all_samples.clone(), sample_rate, 1);
                if !ring_buffer.push(buffer) {
                    debug!("Ring buffer full, dropped {} samples", all_samples.len());
                }
            }
        }

        // Stop the stream
        if let Err(e) = audio_client.stop_stream() {
            error!("Failed to stop audio stream: {:?}", e);
        }

        is_capturing.store(false, Ordering::Relaxed);
        info!("WASAPI loopback capture stopped");
    }

    /// Convert raw audio data to f32 samples
    fn convert_to_f32(data: &[u8], wave_format: &WaveFormat) -> Vec<f32> {
        let bits_per_sample = wave_format.get_bitspersample();

        // Check if it's float format
        let is_float = matches!(wave_format.get_subformat(), Ok(SampleType::Float));

        if bits_per_sample == 32 && is_float {
            // 32-bit float (most common for WASAPI)
            let mut samples = Vec::with_capacity(data.len() / 4);
            for chunk in data.chunks_exact(4) {
                let sample = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                samples.push(sample);
            }
            samples
        } else if bits_per_sample == 16 {
            // 16-bit PCM
            let mut samples = Vec::with_capacity(data.len() / 2);
            for chunk in data.chunks_exact(2) {
                let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
                samples.push(sample as f32 / 32768.0);
            }
            samples
        } else {
            // Fallback: treat as 32-bit float
            warn!("Unsupported audio format: {} bits, assuming 32-bit float", bits_per_sample);
            let mut samples = Vec::with_capacity(data.len() / 4);
            for chunk in data.chunks_exact(4) {
                let sample = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                samples.push(sample);
            }
            samples
        }
    }
}

impl AudioCaptureDevice for WasapiLoopbackDevice {
    fn start_capture(&mut self) -> Result<(), AudioError> {
        if self.is_capturing.load(Ordering::Relaxed) {
            warn!("WASAPI loopback capture already started");
            return Ok(());
        }

        info!("Starting WASAPI loopback capture thread");

        self.should_stop.store(false, Ordering::Relaxed);

        let ring_buffer = self.ring_buffer.clone();
        let is_capturing = self.is_capturing.clone();
        let should_stop = self.should_stop.clone();
        let sample_rate = self.config.sample_rate;
        let channels = self.config.channels;

        let handle = thread::spawn(move || {
            Self::capture_loop(ring_buffer, is_capturing, should_stop, sample_rate, channels);
        });

        self.capture_thread = Some(handle);

        Ok(())
    }

    fn stop_capture(&mut self) -> Result<(), AudioError> {
        if !self.is_capturing.load(Ordering::Relaxed) {
            return Ok(());
        }

        info!("Stopping WASAPI loopback capture");

        self.should_stop.store(true, Ordering::Relaxed);

        if let Some(handle) = self.capture_thread.take() {
            if let Err(e) = handle.join() {
                error!("Failed to join capture thread: {:?}", e);
            }
        }

        Ok(())
    }

    fn is_capturing(&self) -> bool {
        self.is_capturing.load(Ordering::Relaxed)
    }

    fn read_samples(&mut self) -> Option<AudioBuffer> {
        // Read from ring buffer
        self.ring_buffer.pop()
    }

    fn get_config(&self) -> AudioConfig {
        self.config
    }
}

impl Drop for WasapiLoopbackDevice {
    fn drop(&mut self) {
        let _ = self.stop_capture();
    }
}

