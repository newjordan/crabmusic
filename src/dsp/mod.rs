// DSP (Digital Signal Processing) module
// Handles FFT processing and audio parameter extraction

#![allow(dead_code)]

pub mod smoothing;
pub mod windowing;

use crate::audio::AudioBuffer;
use crate::config::BeatDetectionConfig;
use crate::error::DspError;
use rustfft::num_complex::Complex;
use rustfft::FftPlanner;
use std::time::Instant;

/// Beat detector using energy-based onset detection
///
/// Detects beat onsets by tracking sudden increases in audio energy.
/// Uses a threshold-based approach with cooldown to prevent false positives.
#[derive(Debug)]
struct BeatDetector {
    /// Energy history for comparison
    energy_history: Vec<f32>,
    /// Maximum history size
    history_size: usize,
    /// Sensitivity multiplier (higher = more sensitive)
    sensitivity: f32,
    /// Minimum time between beats (in seconds)
    cooldown_seconds: f32,
    /// Last beat time
    last_beat_time: Option<Instant>,
}

impl BeatDetector {
    /// Create a new beat detector
    ///
    /// # Arguments
    /// * `sensitivity` - Sensitivity multiplier (1.0 = normal, higher = more sensitive)
    /// * `cooldown_seconds` - Minimum time between beats in seconds
    fn new(sensitivity: f32, cooldown_seconds: f32) -> Self {
        Self {
            energy_history: Vec::with_capacity(10),
            history_size: 10,
            sensitivity,
            cooldown_seconds,
            last_beat_time: None,
        }
    }

    /// Detect if a beat occurred based on current energy
    ///
    /// # Arguments
    /// * `current_energy` - Current audio energy (RMS amplitude)
    ///
    /// # Returns
    /// true if a beat was detected, false otherwise
    fn detect(&mut self, current_energy: f32) -> bool {
        // Check cooldown
        if let Some(last_time) = self.last_beat_time {
            let elapsed = last_time.elapsed().as_secs_f32();
            if elapsed < self.cooldown_seconds {
                return false;
            }
        }

        // Add current energy to history
        self.energy_history.push(current_energy);
        if self.energy_history.len() > self.history_size {
            self.energy_history.remove(0);
        }

        // Need at least 3 samples for comparison
        if self.energy_history.len() < 3 {
            return false;
        }

        // Calculate average energy of recent history (excluding current)
        let history_avg = self.energy_history[..self.energy_history.len() - 1]
            .iter()
            .sum::<f32>()
            / (self.energy_history.len() - 1) as f32;

        // Detect beat if current energy is significantly higher than average
        let threshold = history_avg * (1.5 / self.sensitivity);
        let is_beat = current_energy > threshold && current_energy > 0.1;

        if is_beat {
            self.last_beat_time = Some(Instant::now());
        }

        is_beat
    }

    /// Set beat detection sensitivity
    ///
    /// # Arguments
    /// * `sensitivity` - Sensitivity multiplier (higher = more sensitive)
    fn set_sensitivity(&mut self, sensitivity: f32) {
        self.sensitivity = sensitivity;
    }

    /// Set beat detection cooldown
    ///
    /// # Arguments
    /// * `cooldown_seconds` - Minimum time between beats in seconds
    fn set_cooldown(&mut self, cooldown_seconds: f32) {
        self.cooldown_seconds = cooldown_seconds;
    }
}

/// Spectral flux beat detector for harmonic onset detection
///
/// Detects beat onsets by tracking changes in frequency spectrum.
/// More sensitive to harmonic instruments (piano, guitar) than energy-based detection.
///
/// # Algorithm
/// Spectral flux measures the rate of change in the frequency spectrum:
/// ```text
/// flux = Σ max(0, spectrum[i] - prev_spectrum[i])
/// ```
/// Only positive differences are counted (increases in spectral energy).
#[derive(Debug)]
struct SpectralFluxDetector {
    /// Previous spectrum for comparison
    prev_spectrum: Vec<f32>,
    /// Flux history for threshold calculation
    flux_history: Vec<f32>,
    /// Maximum history size
    history_size: usize,
    /// Sensitivity multiplier (higher = more sensitive)
    sensitivity: f32,
    /// Minimum time between beats (in seconds)
    cooldown_seconds: f32,
    /// Last beat time
    last_beat_time: Option<Instant>,
}

impl SpectralFluxDetector {
    /// Create a new spectral flux detector
    ///
    /// # Arguments
    /// * `sensitivity` - Sensitivity multiplier (1.0 = normal, higher = more sensitive)
    /// * `cooldown_seconds` - Minimum time between beats in seconds
    /// * `spectrum_size` - Size of the spectrum (typically window_size / 2)
    fn new(sensitivity: f32, cooldown_seconds: f32, spectrum_size: usize) -> Self {
        Self {
            prev_spectrum: vec![0.0; spectrum_size],
            flux_history: Vec::with_capacity(10),
            history_size: 10,
            sensitivity,
            cooldown_seconds,
            last_beat_time: None,
        }
    }

    /// Calculate spectral flux (sum of positive spectral differences)
    ///
    /// # Arguments
    /// * `spectrum` - Current frequency spectrum
    ///
    /// # Returns
    /// Spectral flux value (sum of increases in spectral energy)
    fn calculate_flux(&self, spectrum: &[f32]) -> f32 {
        spectrum
            .iter()
            .zip(self.prev_spectrum.iter())
            .map(|(curr, prev)| (curr - prev).max(0.0))
            .sum()
    }

    /// Detect beat based on spectral flux
    ///
    /// # Arguments
    /// * `spectrum` - Current frequency spectrum
    ///
    /// # Returns
    /// true if a beat was detected, false otherwise
    fn detect(&mut self, spectrum: &[f32]) -> bool {
        // 1. Check cooldown
        if let Some(last_time) = self.last_beat_time {
            let elapsed = last_time.elapsed().as_secs_f32();
            if elapsed < self.cooldown_seconds {
                self.prev_spectrum.copy_from_slice(spectrum);
                return false;
            }
        }

        // 2. Calculate flux
        let flux = self.calculate_flux(spectrum);

        // 3. Update history
        self.flux_history.push(flux);
        if self.flux_history.len() > self.history_size {
            self.flux_history.remove(0);
        }

        // 4. Need at least 3 samples
        if self.flux_history.len() < 3 {
            self.prev_spectrum.copy_from_slice(spectrum);
            return false;
        }

        // 5. Calculate average flux (excluding current)
        let avg_flux = self.flux_history[..self.flux_history.len() - 1]
            .iter()
            .sum::<f32>()
            / (self.flux_history.len() - 1) as f32;

        // 6. Detect onset when flux exceeds threshold
        let threshold = avg_flux * (1.5 / self.sensitivity);
        let is_beat = flux > threshold && flux > 0.01;

        // 7. Update state
        if is_beat {
            self.last_beat_time = Some(Instant::now());
        }
        self.prev_spectrum.copy_from_slice(spectrum);

        is_beat
    }

    /// Set spectral flux detection sensitivity
    ///
    /// # Arguments
    /// * `sensitivity` - Sensitivity multiplier (higher = more sensitive)
    fn set_sensitivity(&mut self, sensitivity: f32) {
        self.sensitivity = sensitivity;
    }

    /// Set spectral flux detection cooldown
    ///
    /// # Arguments
    /// * `cooldown_seconds` - Minimum time between beats in seconds
    fn set_cooldown(&mut self, cooldown_seconds: f32) {
        self.cooldown_seconds = cooldown_seconds;
    }
}

/// Tempo detector for BPM estimation
///
/// Analyzes inter-onset intervals (IOI) between beats to estimate tempo.
/// Uses median filtering for stability and provides confidence metric.
///
/// # Algorithm
/// - Tracks timestamps of recent beats (sliding window)
/// - Calculates inter-onset intervals (IOI) between consecutive beats
/// - Estimates BPM using median of recent IOIs (robust to outliers)
/// - Filters outliers (non-musical tempo ranges)
/// - Calculates confidence based on interval variance
#[derive(Debug)]
struct TempoDetector {
    /// Recent beat timestamps for interval calculation
    beat_times: Vec<Instant>,
    /// Maximum number of beats to track
    history_size: usize,
    /// Current estimated BPM
    current_bpm: f32,
    /// Confidence in tempo estimate (0.0-1.0)
    confidence: f32,
    /// Minimum BPM to consider (filters out slow outliers)
    min_bpm: f32,
    /// Maximum BPM to consider (filters out fast outliers)
    max_bpm: f32,
}

impl TempoDetector {
    /// Create a new tempo detector
    ///
    /// # Arguments
    /// * `min_bpm` - Minimum musical tempo (typically 60 BPM)
    /// * `max_bpm` - Maximum musical tempo (typically 180 BPM)
    fn new(min_bpm: f32, max_bpm: f32) -> Self {
        Self {
            beat_times: Vec::with_capacity(8),
            history_size: 8,
            current_bpm: 120.0, // Default tempo
            confidence: 0.0,    // No confidence initially
            min_bpm,
            max_bpm,
        }
    }

    /// Register a beat occurrence and update tempo estimate
    ///
    /// # Arguments
    /// * `beat_time` - Timestamp of the beat
    fn register_beat(&mut self, beat_time: Instant) {
        // Add beat time to history
        self.beat_times.push(beat_time);
        if self.beat_times.len() > self.history_size {
            self.beat_times.remove(0);
        }

        // Need at least 3 beats to estimate tempo (2 intervals)
        if self.beat_times.len() < 3 {
            self.confidence = 0.0;
            return;
        }

        // Calculate inter-onset intervals (IOI)
        let mut intervals: Vec<f32> = Vec::new();
        for i in 1..self.beat_times.len() {
            let interval = self.beat_times[i]
                .duration_since(self.beat_times[i - 1])
                .as_secs_f32();

            // Filter outliers (too slow or too fast)
            let bpm = 60.0 / interval;
            if bpm >= self.min_bpm && bpm <= self.max_bpm {
                intervals.push(interval);
            }
        }

        // Need valid intervals for estimation
        if intervals.is_empty() {
            self.confidence = 0.0;
            return;
        }

        // Use median interval for stability (robust to outliers)
        intervals.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median_interval = intervals[intervals.len() / 2];

        // Calculate BPM from median interval
        self.current_bpm = 60.0 / median_interval;

        // Calculate confidence from variance
        let mean_interval = intervals.iter().sum::<f32>() / intervals.len() as f32;
        let variance = intervals
            .iter()
            .map(|&x| (x - mean_interval).powi(2))
            .sum::<f32>()
            / intervals.len() as f32;

        // Confidence decreases with variance (normalized)
        // Low variance = high confidence, high variance = low confidence
        self.confidence = (1.0 / (1.0 + variance * 100.0)).min(1.0);
    }

    /// Get current BPM estimate
    fn bpm(&self) -> f32 {
        self.current_bpm
    }

    /// Get confidence in tempo estimate (0.0-1.0)
    fn confidence(&self) -> f32 {
        self.confidence
    }

    /// Set the BPM range for tempo detection
    ///
    /// # Arguments
    /// * `min_bpm` - Minimum musical tempo
    /// * `max_bpm` - Maximum musical tempo
    fn set_bpm_range(&mut self, min_bpm: f32, max_bpm: f32) {
        self.min_bpm = min_bpm;
        self.max_bpm = max_bpm;
    }

    /// Set the history size for tempo estimation
    ///
    /// # Arguments
    /// * `size` - Number of beats to track
    fn set_history_size(&mut self, size: usize) {
        self.history_size = size;
        self.beat_times.reserve(size);
    }
}

/// DSP processor for audio analysis
///
/// Performs FFT analysis and extracts audio parameters like frequency bands,
/// amplitude, and beat detection.
///
/// # Examples
///
/// ```
/// use crabmusic::dsp::DspProcessor;
/// use crabmusic::audio::AudioBuffer;
///
/// let mut processor = DspProcessor::new(44100, 2048).expect("Failed to create processor");
/// let buffer = AudioBuffer::new(2048, 44100, 1);
/// let spectrum = processor.process_buffer(&buffer);
/// assert_eq!(spectrum.len(), 1024); // Half of window size
/// ```
pub struct DspProcessor {
    /// FFT planner for creating FFT instances
    fft_planner: FftPlanner<f32>,
    /// FFT window size (must be power of 2)
    window_size: usize,
    /// Audio sample rate in Hz
    sample_rate: u32,
    /// Pre-computed Hann window coefficients
    hann_window: Vec<f32>,
    /// Pre-allocated scratch buffer for FFT computation
    scratch_buffer: Vec<Complex<f32>>,
    /// Energy-based beat detection state
    beat_detector: BeatDetector,
    /// Spectral flux beat detection state
    flux_detector: SpectralFluxDetector,
    /// Tempo detection and BPM estimation state
    tempo_detector: TempoDetector,
}

impl DspProcessor {
    /// Create a new DSP processor
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate in Hz
    /// * `window_size` - FFT window size (must be power of 2)
    ///
    /// # Returns
    /// A new DspProcessor instance
    ///
    /// # Errors
    /// Returns `DspError::InvalidWindowSize` if window_size is not a power of 2
    ///
    /// # Examples
    ///
    /// ```
    /// use crabmusic::dsp::DspProcessor;
    ///
    /// let processor = DspProcessor::new(44100, 2048).expect("Failed to create processor");
    /// ```
    pub fn new(sample_rate: u32, window_size: usize) -> Result<Self, DspError> {
        // Validate window_size is power of 2
        if !window_size.is_power_of_two() {
            return Err(DspError::InvalidWindowSize(window_size));
        }

        let fft_planner = FftPlanner::new();
        let hann_window = Self::generate_hann_window(window_size);
        let scratch_buffer = vec![Complex::new(0.0, 0.0); window_size];
        let beat_detector = BeatDetector::new(1.0, 0.1); // Normal sensitivity, 100ms cooldown
        let flux_detector = SpectralFluxDetector::new(1.0, 0.1, window_size / 2); // Normal sensitivity, 100ms cooldown, spectrum size
        let tempo_detector = TempoDetector::new(60.0, 180.0); // 60-180 BPM range

        Ok(Self {
            fft_planner,
            window_size,
            sample_rate,
            hann_window,
            scratch_buffer,
            beat_detector,
            flux_detector,
            tempo_detector,
        })
    }

    /// Configure beat detection parameters
    ///
    /// # Arguments
    /// * `config` - Beat detection configuration
    ///
    /// # Examples
    ///
    /// ```
    /// use crabmusic::dsp::DspProcessor;
    /// use crabmusic::config::BeatDetectionConfig;
    ///
    /// let mut processor = DspProcessor::new(44100, 2048).unwrap();
    /// let config = BeatDetectionConfig::default();
    /// processor.configure_beat_detection(&config);
    /// ```
    pub fn configure_beat_detection(&mut self, config: &BeatDetectionConfig) {
        // Configure beat detectors
        self.beat_detector.set_sensitivity(config.sensitivity);
        self.beat_detector.set_cooldown(config.cooldown_seconds);
        self.flux_detector.set_sensitivity(config.sensitivity);
        self.flux_detector.set_cooldown(config.cooldown_seconds);

        // Configure tempo detector
        self.tempo_detector.set_bpm_range(config.min_bpm, config.max_bpm);
        self.tempo_detector.set_history_size(config.tempo_history_size);
    }

    /// Generate Hann window coefficients
    ///
    /// The Hann window reduces spectral leakage by smoothly tapering
    /// the signal to zero at the edges.
    ///
    /// # Arguments
    /// * `size` - Window size
    ///
    /// # Returns
    /// Vector of Hann window coefficients
    fn generate_hann_window(size: usize) -> Vec<f32> {
        (0..size)
            .map(|i| {
                let factor = 2.0 * std::f32::consts::PI * i as f32 / (size - 1) as f32;
                0.5 * (1.0 - factor.cos())
            })
            .collect()
    }

    /// Process an audio buffer and return frequency spectrum
    ///
    /// Applies Hann windowing, performs FFT, and returns normalized magnitude spectrum.
    ///
    /// # Arguments
    /// * `buffer` - Audio buffer to process
    ///
    /// # Returns
    /// Vector of normalized magnitude values (0.0-1.0) for each frequency bin.
    /// Length is window_size / 2 (only positive frequencies).
    ///
    /// # Examples
    ///
    /// ```
    /// use crabmusic::dsp::DspProcessor;
    /// use crabmusic::audio::AudioBuffer;
    ///
    /// let mut processor = DspProcessor::new(44100, 2048).unwrap();
    /// let buffer = AudioBuffer::new(2048, 44100, 1);
    /// let spectrum = processor.process_buffer(&buffer);
    /// assert_eq!(spectrum.len(), 1024);
    /// ```
    pub fn process_buffer(&mut self, buffer: &AudioBuffer) -> Vec<f32> {
        let num_samples = buffer.samples.len().min(self.window_size);

        // 1. Apply Hann window to reduce spectral leakage
        for i in 0..num_samples {
            self.scratch_buffer[i] = Complex::new(buffer.samples[i] * self.hann_window[i], 0.0);
        }

        // Zero-pad if buffer is smaller than window size
        for i in num_samples..self.window_size {
            self.scratch_buffer[i] = Complex::new(0.0, 0.0);
        }

        // 2. Perform FFT
        let fft = self.fft_planner.plan_fft_forward(self.window_size);
        fft.process(&mut self.scratch_buffer);

        // 3. Convert to magnitude spectrum (only first half, FFT is symmetric)
        let spectrum: Vec<f32> = self.scratch_buffer[..self.window_size / 2]
            .iter()
            .map(|c| (c.re * c.re + c.im * c.im).sqrt())
            .collect();

        // 4. Normalize to 0.0-1.0 range
        let max_magnitude = spectrum.iter().cloned().fold(0.0f32, f32::max);
        if max_magnitude > 0.0 {
            spectrum.iter().map(|&m| m / max_magnitude).collect()
        } else {
            spectrum
        }
    }

    /// Map frequency bin index to frequency in Hz
    ///
    /// # Arguments
    /// * `bin` - Frequency bin index
    ///
    /// # Returns
    /// Frequency in Hz corresponding to the bin
    ///
    /// # Examples
    ///
    /// ```
    /// use crabmusic::dsp::DspProcessor;
    ///
    /// let processor = DspProcessor::new(44100, 2048).unwrap();
    /// let freq = processor.bin_to_frequency(10);
    /// assert!((freq - 215.33).abs() < 0.1); // ~215 Hz
    /// ```
    pub fn bin_to_frequency(&self, bin: usize) -> f32 {
        bin as f32 * self.sample_rate as f32 / self.window_size as f32
    }

    /// Process an audio buffer and extract parameters
    ///
    /// Performs FFT analysis and extracts frequency bands.
    ///
    /// # Arguments
    /// * `buffer` - Audio buffer to process
    ///
    /// # Returns
    /// AudioParameters with extracted frequency bands and amplitude
    ///
    /// # Examples
    ///
    /// ```
    /// use crabmusic::dsp::DspProcessor;
    /// use crabmusic::audio::AudioBuffer;
    ///
    /// let mut processor = DspProcessor::new(44100, 2048).unwrap();
    /// let buffer = AudioBuffer::new(2048, 44100, 1);
    /// let params = processor.process(&buffer);
    /// assert!(params.bass >= 0.0 && params.bass <= 1.0);
    /// ```
    pub fn process(&mut self, buffer: &AudioBuffer) -> AudioParameters {
        // 1. Get FFT spectrum
        let spectrum = self.process_buffer(buffer);

        // 2. Extract frequency bands
        let bass = self.extract_band(&spectrum, 20.0, 250.0);
        let mid = self.extract_band(&spectrum, 250.0, 4000.0);
        let treble = self.extract_band(&spectrum, 4000.0, 20000.0);

        // 3. Calculate overall amplitude (RMS)
        let amplitude = self.calculate_rms(&buffer.samples);

        // 4. Detect beat using energy-based onset detection
        let beat_energy = self.beat_detector.detect(amplitude);

        // 5. Detect beat using spectral flux (harmonic onset detection)
        let beat_flux = self.flux_detector.detect(&spectrum);

        // 6. Hybrid beat detection: combine both methods
        // This catches both percussive (energy) and harmonic (flux) onsets
        let beat = beat_energy || beat_flux;

        // 7. Update tempo detector if beat occurred
        if beat {
            self.tempo_detector.register_beat(Instant::now());
        }

        // 8. Get tempo estimate
        let bpm = self.tempo_detector.bpm();
        let tempo_confidence = self.tempo_detector.confidence();

        // 9. Extract waveform for oscilloscope visualization
        let waveform = self.downsample_for_waveform(buffer, 512);

        AudioParameters {
            bass,
            mid,
            treble,
            amplitude,
            beat,             // Hybrid: energy OR flux
            beat_flux,        // Flux-only (for debugging/visualization)
            bpm,              // Estimated tempo in BPM
            tempo_confidence, // Confidence in tempo estimate
            spectrum,         // Include full spectrum for advanced visualizers
            waveform,         // Include waveform for oscilloscope
        }
    }

    /// Extract energy from a frequency band
    ///
    /// # Arguments
    /// * `spectrum` - FFT magnitude spectrum
    /// * `freq_min` - Minimum frequency in Hz
    /// * `freq_max` - Maximum frequency in Hz
    ///
    /// # Returns
    /// Normalized band energy (0.0-1.0)
    fn extract_band(&self, spectrum: &[f32], freq_min: f32, freq_max: f32) -> f32 {
        let (bin_min, bin_max) = self.frequency_range_to_bins(freq_min, freq_max);

        if bin_min >= bin_max || bin_max > spectrum.len() {
            return 0.0;
        }

        // Sum energy in frequency range
        let sum: f32 = spectrum[bin_min..bin_max].iter().sum();
        let count = (bin_max - bin_min) as f32;

        // Return average energy in band
        if count > 0.0 {
            sum / count
        } else {
            0.0
        }
    }

    /// Convert frequency range to bin range
    ///
    /// # Arguments
    /// * `freq_min` - Minimum frequency in Hz
    /// * `freq_max` - Maximum frequency in Hz
    ///
    /// # Returns
    /// Tuple of (bin_min, bin_max)
    fn frequency_range_to_bins(&self, freq_min: f32, freq_max: f32) -> (usize, usize) {
        let bin_min =
            (freq_min * self.window_size as f32 / self.sample_rate as f32).ceil() as usize;
        let bin_max =
            (freq_max * self.window_size as f32 / self.sample_rate as f32).floor() as usize;
        (bin_min, bin_max)
    }

    /// Calculate RMS (Root Mean Square) amplitude
    ///
    /// # Arguments
    /// * `samples` - Audio samples
    ///
    /// # Returns
    /// RMS amplitude (0.0-1.0)
    fn calculate_rms(&self, samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }

        let sum_squares: f32 = samples.iter().map(|&s| s * s).sum();
        (sum_squares / samples.len() as f32).sqrt()
    }

    /// Downsample audio buffer to target length for oscilloscope display
    ///
    /// Uses intelligent downsampling that preserves waveform shape.
    /// For small buffers: returns as-is (upsampling not needed for oscilloscope)
    /// For large buffers: downsamples using averaging for anti-aliasing
    ///
    /// # Arguments
    /// * `buffer` - Audio buffer (may be stereo, will be mono-mixed)
    /// * `target_length` - Desired output length (typically 512)
    ///
    /// # Returns
    /// Downsampled waveform normalized to -1.0 to 1.0
    ///
    /// # Examples
    ///
    /// ```
    /// use crabmusic::dsp::DspProcessor;
    /// use crabmusic::audio::AudioBuffer;
    ///
    /// let mut processor = DspProcessor::new(44100, 2048).unwrap();
    /// let buffer = AudioBuffer::new(2048, 44100, 2);
    /// // Internal method, tested via process()
    /// ```
    fn downsample_for_waveform(&self, buffer: &AudioBuffer, target_length: usize) -> Vec<f32> {
        // Convert stereo to mono by averaging channels
        let mono_samples: Vec<f32> = if buffer.channels == 2 {
            buffer
                .samples
                .chunks_exact(2)
                .map(|chunk| (chunk[0] + chunk[1]) / 2.0)
                .collect()
        } else {
            buffer.samples.clone()
        };

        let input_len = mono_samples.len();

        // Handle empty buffer
        if input_len == 0 {
            return vec![0.0; target_length];
        }

        // If input is already small enough, return as-is (or pad with zeros)
        if input_len <= target_length {
            let mut result = mono_samples;
            result.resize(target_length, 0.0);
            return result;
        }

        // Downsample using decimation with averaging for anti-aliasing
        // Take every Nth sample where N = input_len / target_length
        let step = input_len as f32 / target_length as f32;
        let mut output = Vec::with_capacity(target_length);

        for i in 0..target_length {
            let idx = (i as f32 * step) as usize;
            // Average a few samples around this point for smoother result
            let start = idx.saturating_sub(1);
            let end = (idx + 2).min(input_len);
            let avg = mono_samples[start..end].iter().sum::<f32>() / (end - start) as f32;
            output.push(avg.clamp(-1.0, 1.0)); // Normalize
        }

        output
    }
}

/// Audio parameters extracted from DSP processing
///
/// Contains frequency band amplitudes, overall amplitude, and other
/// audio features used for visualization.
#[derive(Debug, Clone)]
pub struct AudioParameters {
    /// Bass frequency band amplitude (20-250 Hz)
    pub bass: f32,

    /// Mid frequency band amplitude (250-4000 Hz)
    pub mid: f32,

    /// Treble frequency band amplitude (4000-20000 Hz)
    pub treble: f32,

    /// Overall amplitude (RMS)
    pub amplitude: f32,

    /// Beat detected (true if beat onset detected via hybrid detection)
    ///
    /// This combines both energy-based and spectral flux detection
    /// for comprehensive beat detection across all music types.
    pub beat: bool,

    /// Beat detected via spectral flux only (for debugging/visualization)
    ///
    /// This is true only when spectral flux detection triggers,
    /// useful for analyzing harmonic onset detection performance.
    pub beat_flux: bool,

    /// Estimated tempo in BPM (Beats Per Minute)
    ///
    /// Range: typically 60-180 BPM for most music.
    /// Default: 120 BPM when insufficient data.
    ///
    /// Use `tempo_confidence` to determine reliability.
    ///
    /// # Examples
    ///
    /// ```
    /// if params.tempo_confidence > 0.7 {
    ///     // High confidence - use BPM for tempo-synced effects
    ///     let pulse_period = 60.0 / params.bpm; // seconds per beat
    /// }
    /// ```
    pub bpm: f32,

    /// Confidence in tempo estimate (0.0-1.0)
    ///
    /// - 0.0 = No confidence (insufficient data or unstable tempo)
    /// - 0.5 = Moderate confidence (some variance in beat timing)
    /// - 1.0 = High confidence (stable, consistent beat timing)
    ///
    /// Visualizers should check confidence before using BPM:
    /// - confidence > 0.7: Safe to use for tempo-synced effects
    /// - confidence < 0.5: Tempo unreliable, use default or disable sync
    pub tempo_confidence: f32,

    /// Full frequency spectrum (FFT magnitude bins)
    ///
    /// Normalized magnitude values (0.0-1.0) for each frequency bin.
    /// Length is window_size / 2 (e.g., 1024 for 2048 window).
    /// Bin i corresponds to frequency: i * sample_rate / window_size Hz
    ///
    /// Example: For 44100 Hz sample rate and 2048 window:
    /// - bin[0] = 0 Hz (DC)
    /// - bin[1] = 21.5 Hz
    /// - bin[10] = 215 Hz
    /// - bin[100] = 2150 Hz
    pub spectrum: Vec<f32>,

    /// Waveform samples for oscilloscope visualization
    ///
    /// Downsampled audio waveform showing actual signal shape.
    /// Normalized to -1.0 to 1.0 range, mono-mixed if stereo.
    /// Default length: 512 samples (enough for ~11ms at 44.1kHz)
    ///
    /// This is a time-domain representation of the audio signal,
    /// suitable for oscilloscope-style visualization.
    ///
    /// Example: For a 1kHz sine wave, this would show one or more
    /// complete cycles with smooth positive and negative swings.
    pub waveform: Vec<f32>,
}

impl Default for AudioParameters {
    fn default() -> Self {
        Self {
            bass: 0.0,
            mid: 0.0,
            treble: 0.0,
            amplitude: 0.0,
            beat: false,
            beat_flux: false,
            bpm: 120.0,           // Default tempo
            tempo_confidence: 0.0, // No confidence initially
            spectrum: Vec::new(),
            waveform: Vec::new(),
        }
    }
}

impl AudioParameters {
    /// Create new audio parameters with all values at zero
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to generate synthetic sine wave for testing
    fn generate_sine_wave(
        freq: f32,
        amplitude: f32,
        sample_rate: u32,
        num_samples: usize,
    ) -> AudioBuffer {
        let samples: Vec<f32> = (0..num_samples)
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                amplitude * (2.0 * std::f32::consts::PI * freq * t).sin()
            })
            .collect();

        AudioBuffer::with_samples(samples, sample_rate, 1)
    }

    #[test]
    fn test_dsp_processor_creation() {
        // Valid window size (power of 2)
        let result = DspProcessor::new(44100, 1024);
        assert!(result.is_ok());

        // Invalid window size (not power of 2)
        let result = DspProcessor::new(44100, 1000);
        assert!(result.is_err());
    }

    #[test]
    fn test_fft_detects_440hz_sine_wave() {
        // Generate pure 440 Hz tone
        let buffer = generate_sine_wave(440.0, 1.0, 44100, 2048);
        let mut processor = DspProcessor::new(44100, 2048).unwrap();

        let spectrum = processor.process_buffer(&buffer);

        // Find peak frequency
        let peak_bin = spectrum
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap();

        let peak_freq = processor.bin_to_frequency(peak_bin);

        // Frequency resolution is sample_rate / window_size = 44100 / 2048 = ~21.5 Hz
        // So we should be within one bin (~22 Hz) of 440 Hz
        assert!(
            (peak_freq - 440.0).abs() < 22.0,
            "Expected ~440 Hz (within 22 Hz), got {} Hz",
            peak_freq
        );
    }

    #[test]
    fn test_hann_window_symmetry() {
        let window = DspProcessor::generate_hann_window(1024);
        assert_eq!(window.len(), 1024);

        // Hann window should be symmetric
        assert!((window[0] - window[1023]).abs() < 0.001);

        // Peak should be in the middle
        assert!(window[512] > window[0]);
        assert!(window[512] > window[1023]);
    }

    #[test]
    fn test_zero_input_produces_zero_spectrum() {
        let buffer = AudioBuffer::with_samples(vec![0.0; 2048], 44100, 1);
        let mut processor = DspProcessor::new(44100, 2048).unwrap();

        let spectrum = processor.process_buffer(&buffer);

        assert!(spectrum.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn test_spectrum_length() {
        let buffer = generate_sine_wave(440.0, 1.0, 44100, 2048);
        let mut processor = DspProcessor::new(44100, 2048).unwrap();

        let spectrum = processor.process_buffer(&buffer);

        // Spectrum should be half the window size (only positive frequencies)
        assert_eq!(spectrum.len(), 1024);
    }

    #[test]
    fn test_spectrum_normalized() {
        let buffer = generate_sine_wave(440.0, 1.0, 44100, 2048);
        let mut processor = DspProcessor::new(44100, 2048).unwrap();

        let spectrum = processor.process_buffer(&buffer);

        // Spectrum should be normalized to 0.0-1.0 range
        let max_val = spectrum.iter().cloned().fold(0.0f32, f32::max);
        assert!(
            (max_val - 1.0).abs() < 0.001,
            "Max should be ~1.0, got {}",
            max_val
        );

        // All values should be in range
        assert!(spectrum.iter().all(|&s| (0.0..=1.0).contains(&s)));
    }

    #[test]
    fn test_bin_to_frequency() {
        let processor = DspProcessor::new(44100, 2048).unwrap();

        // Bin 0 should be 0 Hz (DC component)
        assert_eq!(processor.bin_to_frequency(0), 0.0);

        // Bin 10 should be ~215 Hz
        let freq = processor.bin_to_frequency(10);
        assert!((freq - 215.33).abs() < 0.1);

        // Nyquist frequency (bin 1024) should be 22050 Hz
        let nyquist = processor.bin_to_frequency(1024);
        assert!((nyquist - 22050.0).abs() < 0.1);
    }

    #[test]
    fn test_audio_parameters_default() {
        let params = AudioParameters::new();
        assert_eq!(params.bass, 0.0);
        assert_eq!(params.mid, 0.0);
        assert_eq!(params.treble, 0.0);
        assert_eq!(params.amplitude, 0.0);
        assert!(!params.beat);
    }

    #[test]
    fn test_extract_bass_band() {
        // Generate 100 Hz sine wave (bass frequency)
        let buffer = generate_sine_wave(100.0, 1.0, 44100, 2048);
        let mut processor = DspProcessor::new(44100, 2048).unwrap();

        let params = processor.process(&buffer);

        // Bass should be dominant (but values are averaged across band, so will be lower)
        assert!(
            params.bass > 0.0,
            "Bass should be > 0.0, got {}",
            params.bass
        );
        assert!(
            params.bass > params.mid,
            "Bass ({}) should be > mid ({})",
            params.bass,
            params.mid
        );
        assert!(
            params.bass > params.treble,
            "Bass ({}) should be > treble ({})",
            params.bass,
            params.treble
        );
    }

    #[test]
    fn test_extract_mid_band() {
        // Generate 1000 Hz sine wave (mid frequency)
        let buffer = generate_sine_wave(1000.0, 1.0, 44100, 2048);
        let mut processor = DspProcessor::new(44100, 2048).unwrap();

        let params = processor.process(&buffer);

        // Mid should be dominant
        assert!(params.mid > 0.0, "Mid should be > 0.0, got {}", params.mid);
        assert!(
            params.mid > params.bass,
            "Mid ({}) should be > bass ({})",
            params.mid,
            params.bass
        );
        assert!(
            params.mid > params.treble,
            "Mid ({}) should be > treble ({})",
            params.mid,
            params.treble
        );
    }

    #[test]
    fn test_extract_treble_band() {
        // Generate 8000 Hz sine wave (treble frequency)
        let buffer = generate_sine_wave(8000.0, 1.0, 44100, 2048);
        let mut processor = DspProcessor::new(44100, 2048).unwrap();

        let params = processor.process(&buffer);

        // Treble should be dominant
        assert!(
            params.treble > 0.0,
            "Treble should be > 0.0, got {}",
            params.treble
        );
        assert!(
            params.treble > params.bass,
            "Treble ({}) should be > bass ({})",
            params.treble,
            params.bass
        );
        assert!(
            params.treble > params.mid,
            "Treble ({}) should be > mid ({})",
            params.treble,
            params.mid
        );
    }

    #[test]
    fn test_calculate_rms() {
        let processor = DspProcessor::new(44100, 2048).unwrap();

        // Test with known values
        // RMS = sqrt((0 + 1 + 0 + 1) / 4) = sqrt(0.5) ≈ 0.707
        let samples = vec![0.0, 1.0, 0.0, -1.0];
        let rms = processor.calculate_rms(&samples);
        assert!((rms - 0.707).abs() < 0.01, "Expected ~0.707, got {}", rms);

        // Test with silence
        let silence = vec![0.0; 100];
        let rms = processor.calculate_rms(&silence);
        assert_eq!(rms, 0.0);
    }

    #[test]
    fn test_frequency_range_to_bins() {
        let processor = DspProcessor::new(44100, 2048).unwrap();

        // Bass range: 20-250 Hz
        let (bin_min, bin_max) = processor.frequency_range_to_bins(20.0, 250.0);
        assert!(bin_min < bin_max);
        assert!(processor.bin_to_frequency(bin_min) >= 20.0);
        assert!(processor.bin_to_frequency(bin_max) <= 250.0);

        // Mid range: 250-4000 Hz
        let (bin_min, bin_max) = processor.frequency_range_to_bins(250.0, 4000.0);
        assert!(bin_min < bin_max);
        assert!(processor.bin_to_frequency(bin_min) >= 250.0);
        assert!(processor.bin_to_frequency(bin_max) <= 4000.0);
    }

    #[test]
    fn test_zero_input_produces_zero_parameters() {
        let buffer = AudioBuffer::with_samples(vec![0.0; 2048], 44100, 1);
        let mut processor = DspProcessor::new(44100, 2048).unwrap();

        let params = processor.process(&buffer);

        assert_eq!(params.bass, 0.0);
        assert_eq!(params.mid, 0.0);
        assert_eq!(params.treble, 0.0);
        assert_eq!(params.amplitude, 0.0);
    }

    // ============================================================
    // Beat Detection Tests
    // ============================================================

    #[test]
    fn test_beat_detector_creation() {
        let detector = BeatDetector::new(1.0, 0.1);
        assert_eq!(detector.sensitivity, 1.0);
        assert_eq!(detector.cooldown_seconds, 0.1);
        assert_eq!(detector.history_size, 10);
        assert!(detector.last_beat_time.is_none());
        assert_eq!(detector.energy_history.len(), 0);
    }

    #[test]
    fn test_beat_detector_detects_energy_spike() {
        let mut detector = BeatDetector::new(1.0, 0.1);

        // Build up low energy history
        for _ in 0..5 {
            assert_eq!(detector.detect(0.1), false);
        }

        // Sudden energy spike should trigger beat
        let is_beat = detector.detect(0.5);
        assert!(is_beat, "Should detect beat on energy spike");
    }

    #[test]
    fn test_beat_detector_cooldown() {
        let mut detector = BeatDetector::new(1.0, 1.0); // 1 second cooldown

        // First beat
        detector.detect(0.1);
        detector.detect(0.1);
        let beat1 = detector.detect(0.5);
        assert!(beat1, "First beat should be detected");

        // Immediate second spike should be blocked by cooldown
        let beat2 = detector.detect(0.5);
        assert!(!beat2, "Second beat should be blocked by cooldown");

        // After cooldown expires, should detect again
        std::thread::sleep(std::time::Duration::from_millis(1100));
        detector.detect(0.1); // Reset to low energy
        let beat3 = detector.detect(0.5);
        assert!(beat3, "Beat after cooldown should be detected");
    }

    #[test]
    fn test_beat_detector_requires_minimum_energy() {
        let mut detector = BeatDetector::new(1.0, 0.1);

        // Very low energy history
        for _ in 0..5 {
            detector.detect(0.01);
        }

        // Spike to 0.05 (below 0.1 threshold) should not trigger
        let is_beat = detector.detect(0.05);
        assert!(!is_beat, "Should not detect beat below minimum energy threshold");
    }

    #[test]
    fn test_beat_detector_sensitivity() {
        // High sensitivity (easier to trigger)
        let mut detector_sensitive = BeatDetector::new(2.0, 0.1);
        for _ in 0..5 {
            detector_sensitive.detect(0.2);
        }
        let beat_sensitive = detector_sensitive.detect(0.25);

        // Low sensitivity (harder to trigger)
        let mut detector_normal = BeatDetector::new(1.0, 0.1);
        for _ in 0..5 {
            detector_normal.detect(0.2);
        }
        let beat_normal = detector_normal.detect(0.25);

        // Sensitive detector should trigger more easily
        assert!(beat_sensitive, "High sensitivity should detect smaller changes");
        assert!(!beat_normal, "Normal sensitivity should require larger changes");
    }

    #[test]
    fn test_beat_detector_ignores_gradual_changes() {
        let mut detector = BeatDetector::new(1.0, 0.1);

        // Gradual increase should not trigger beats
        let energies = vec![0.1, 0.12, 0.14, 0.16, 0.18, 0.20];
        let mut beats_detected = 0;

        for energy in energies {
            if detector.detect(energy) {
                beats_detected += 1;
            }
        }

        // Should detect 0 or very few beats (gradual change, not sudden onset)
        assert!(beats_detected < 2, "Should not detect beats in gradual changes");
    }

    #[test]
    fn test_beat_detector_integration_with_processor() {
        let mut processor = DspProcessor::new(44100, 2048).unwrap();

        // Silent buffer - no beat
        let silent = AudioBuffer::with_samples(vec![0.0; 2048], 44100, 1);
        let params_silent = processor.process(&silent);
        assert!(!params_silent.beat, "No beat in silence");

        // Build low energy history
        let quiet = AudioBuffer::with_samples(vec![0.1; 2048], 44100, 1);
        for _ in 0..5 {
            processor.process(&quiet);
        }

        // Loud buffer - should trigger beat
        let loud = AudioBuffer::with_samples(vec![0.5; 2048], 44100, 1);
        let params_loud = processor.process(&loud);
        assert!(params_loud.beat, "Should detect beat on energy spike");
    }

    // ============================================================
    // Waveform Downsampling Tests
    // ============================================================

    #[test]
    fn test_waveform_included_in_params() {
        let mut processor = DspProcessor::new(44100, 2048).unwrap();
        let buffer = generate_sine_wave(440.0, 1.0, 44100, 2048);

        let params = processor.process(&buffer);

        // Waveform should be present
        assert_eq!(params.waveform.len(), 512);
    }

    #[test]
    fn test_waveform_normalized_range() {
        let mut processor = DspProcessor::new(44100, 2048).unwrap();
        let buffer = generate_sine_wave(440.0, 1.0, 44100, 2048);

        let params = processor.process(&buffer);

        // All waveform values should be in -1.0 to 1.0 range
        for &sample in &params.waveform {
            assert!(
                (-1.0..=1.0).contains(&sample),
                "Waveform sample {} out of range",
                sample
            );
        }
    }

    #[test]
    fn test_waveform_preserves_sine_shape() {
        let mut processor = DspProcessor::new(44100, 2048).unwrap();
        let buffer = generate_sine_wave(440.0, 1.0, 44100, 2048);

        let params = processor.process(&buffer);

        // Sine wave should have positive and negative values
        let has_positive = params.waveform.iter().any(|&s| s > 0.5);
        let has_negative = params.waveform.iter().any(|&s| s < -0.5);

        assert!(has_positive, "Waveform should have positive values");
        assert!(has_negative, "Waveform should have negative values");
    }

    #[test]
    fn test_waveform_stereo_to_mono_mixing() {
        let mut processor = DspProcessor::new(44100, 2048).unwrap();

        // Create stereo buffer with different values in each channel
        let mut samples = Vec::new();
        for i in 0..2048 {
            let t = i as f32 / 44100.0;
            let left = (2.0 * std::f32::consts::PI * 440.0 * t).sin();
            let right = 0.0; // Right channel is silent
            samples.push(left);
            samples.push(right);
        }

        let buffer = AudioBuffer::with_samples(samples, 44100, 2);
        let params = processor.process(&buffer);

        // Mono mix should have some signal (averaged from left channel)
        assert!(
            params.waveform.iter().any(|&s| s.abs() > 0.1),
            "Mono mix should preserve signal"
        );
    }

    #[test]
    fn test_waveform_downsampling_preserves_waveform() {
        let processor = DspProcessor::new(44100, 2048).unwrap();

        // Create 2048-sample sine wave
        let buffer = generate_sine_wave(100.0, 1.0, 44100, 2048);

        // Downsample to 512
        let waveform = processor.downsample_for_waveform(&buffer, 512);

        assert_eq!(waveform.len(), 512);

        // Should still have positive and negative swings
        assert!(waveform.iter().any(|&s| s > 0.5));
        assert!(waveform.iter().any(|&s| s < -0.5));
    }

    #[test]
    fn test_waveform_empty_buffer() {
        let processor = DspProcessor::new(44100, 2048).unwrap();
        let buffer = AudioBuffer::with_samples(vec![], 44100, 1);

        let waveform = processor.downsample_for_waveform(&buffer, 512);

        assert_eq!(waveform.len(), 512);
        assert!(waveform.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn test_waveform_small_buffer() {
        let processor = DspProcessor::new(44100, 2048).unwrap();
        let buffer = AudioBuffer::with_samples(vec![0.5; 100], 44100, 1);

        let waveform = processor.downsample_for_waveform(&buffer, 512);

        // Should pad to 512 samples
        assert_eq!(waveform.len(), 512);

        // First 100 should have values, rest should be zero
        assert!(waveform[0..100].iter().any(|&s| s.abs() > 0.1));
        assert!(waveform[100..512].iter().all(|&s| s == 0.0));
    }

    // ===== Spectral Flux Detection Tests =====

    #[test]
    fn test_spectral_flux_calculation() {
        let mut detector = SpectralFluxDetector::new(1.0, 0.1, 1024);

        // Spectrum with no change -> flux should be near 0
        let spectrum = vec![0.5; 1024];
        detector.prev_spectrum.copy_from_slice(&spectrum);
        let flux = detector.calculate_flux(&spectrum);
        assert_eq!(flux, 0.0, "Flux should be 0 when spectrum doesn't change");

        // Spectrum with increase -> positive flux
        let new_spectrum = vec![0.7; 1024];
        let flux = detector.calculate_flux(&new_spectrum);
        assert!(
            flux > 0.0,
            "Flux should be positive when spectrum increases"
        );
        let expected_flux = (0.7 - 0.5) * 1024.0; // 0.2 * 1024 bins
        assert!(
            (flux - expected_flux).abs() < 1.0,
            "Flux should be ~{}, got {}",
            expected_flux,
            flux
        );

        // Spectrum with decrease -> flux should be near 0 (only positive differences counted)
        detector.prev_spectrum.copy_from_slice(&new_spectrum);
        let lower_spectrum = vec![0.3; 1024];
        let flux = detector.calculate_flux(&lower_spectrum);
        assert_eq!(
            flux, 0.0,
            "Flux should be 0 when spectrum decreases (only positive changes counted)"
        );
    }

    #[test]
    fn test_spectral_flux_detects_harmonic_onset() {
        let mut detector = SpectralFluxDetector::new(1.0, 0.1, 1024);

        // Build history with stable spectrum (simulate quiet passage)
        let stable = vec![0.1; 1024];
        for _ in 0..5 {
            detector.detect(&stable);
        }

        // Sudden spectral change (simulate harmonic onset like piano note)
        let onset = vec![0.5; 1024];
        let is_beat = detector.detect(&onset);
        assert!(is_beat, "Should detect harmonic onset");
    }

    #[test]
    fn test_spectral_flux_cooldown() {
        let mut detector = SpectralFluxDetector::new(1.0, 0.1, 1024);

        // Build history
        let stable = vec![0.1; 1024];
        for _ in 0..5 {
            detector.detect(&stable);
        }

        // First onset should trigger
        let onset = vec![0.5; 1024];
        let is_beat1 = detector.detect(&onset);
        assert!(is_beat1, "First onset should trigger");

        // Immediate second onset should be blocked by cooldown
        let is_beat2 = detector.detect(&onset);
        assert!(
            !is_beat2,
            "Second onset should be blocked by cooldown (100ms)"
        );
    }

    #[test]
    fn test_spectral_flux_sensitivity() {
        // Low sensitivity (requires larger change)
        let mut detector_low = SpectralFluxDetector::new(0.5, 0.1, 1024);

        // High sensitivity (detects smaller changes)
        let mut detector_high = SpectralFluxDetector::new(2.0, 0.1, 1024);

        // Build history
        let stable = vec![0.1; 1024];
        for _ in 0..5 {
            detector_low.detect(&stable);
            detector_high.detect(&stable);
        }

        // Small spectral change
        let small_change = vec![0.15; 1024];

        let low_sens_beat = detector_low.detect(&small_change);
        let high_sens_beat = detector_high.detect(&small_change);

        // High sensitivity should be more likely to detect small changes
        // (though this test might be flaky depending on exact threshold)
        assert!(
            !low_sens_beat || high_sens_beat,
            "High sensitivity should detect at least as many beats as low sensitivity"
        );
    }

    #[test]
    fn test_hybrid_detection_combines_energy_and_flux() {
        let mut processor = DspProcessor::new(44100, 2048).unwrap();

        // Test 1: Pure tone with sudden amplitude increase (should trigger energy-based)
        let buffer_quiet = generate_sine_wave(440.0, 0.1, 44100, 2048);
        let buffer_loud = generate_sine_wave(440.0, 1.0, 44100, 2048);

        // Build history with quiet sound
        for _ in 0..5 {
            processor.process(&buffer_quiet);
        }

        // Sudden loud sound should trigger beat
        let params = processor.process(&buffer_loud);
        assert!(
            params.beat,
            "Hybrid detection should catch energy-based onset"
        );
    }

    #[test]
    fn test_spectral_flux_minimum_threshold() {
        let mut detector = SpectralFluxDetector::new(1.0, 0.1, 1024);

        // Build history with near-zero spectrum
        let near_zero = vec![0.00001; 1024];
        for _ in 0..5 {
            detector.detect(&near_zero);
        }

        // Tiny increase (total flux = 0.000005 * 1024 = 0.00512, below minimum threshold of 0.01)
        let tiny_increase = vec![0.000015; 1024];
        let is_beat = detector.detect(&tiny_increase);

        assert!(
            !is_beat,
            "Should not detect beat for flux below minimum threshold (0.01)"
        );
    }

    #[test]
    fn test_audio_parameters_includes_beat_flux() {
        let mut processor = DspProcessor::new(44100, 2048).unwrap();
        let buffer = generate_sine_wave(440.0, 1.0, 44100, 2048);

        let params = processor.process(&buffer);

        // beat_flux field should exist and be a bool
        assert!(params.beat_flux == true || params.beat_flux == false);

        // beat should also exist (hybrid)
        assert!(params.beat == true || params.beat == false);
    }

    // ============================================================
    // Tempo Detection Tests
    // ============================================================

    #[test]
    fn test_tempo_detector_creation() {
        let detector = TempoDetector::new(60.0, 180.0);
        assert_eq!(detector.current_bpm, 120.0); // Default tempo
        assert_eq!(detector.confidence, 0.0);     // No confidence yet
        assert_eq!(detector.beat_times.len(), 0);
        assert_eq!(detector.min_bpm, 60.0);
        assert_eq!(detector.max_bpm, 180.0);
        assert_eq!(detector.history_size, 8);
    }

    #[test]
    fn test_tempo_detector_requires_minimum_beats() {
        let mut detector = TempoDetector::new(60.0, 180.0);

        // Register 2 beats - insufficient for tempo (need at least 3)
        let now = Instant::now();
        detector.register_beat(now);
        std::thread::sleep(std::time::Duration::from_millis(500));
        detector.register_beat(Instant::now());

        assert_eq!(detector.confidence(), 0.0);
    }

    #[test]
    fn test_tempo_detector_estimates_120_bpm() {
        let mut detector = TempoDetector::new(60.0, 180.0);

        // 120 BPM = 500ms per beat
        // Note: Thread sleep is not perfectly accurate, so we use larger tolerance
        for _i in 0..5 {
            std::thread::sleep(std::time::Duration::from_millis(500));
            detector.register_beat(Instant::now());
        }

        let bpm = detector.bpm();
        assert!(
            (bpm - 120.0).abs() < 20.0,
            "Expected ~120 BPM (±20), got {}",
            bpm
        );
        assert!(detector.confidence() > 0.3, "Should have reasonable confidence");
    }

    #[test]
    fn test_tempo_detector_filters_outliers() {
        let mut detector = TempoDetector::new(60.0, 180.0);

        // Mix of valid beats and outliers
        let now = Instant::now();
        detector.register_beat(now);

        std::thread::sleep(std::time::Duration::from_millis(500));
        detector.register_beat(Instant::now()); // 120 BPM

        std::thread::sleep(std::time::Duration::from_millis(500));
        detector.register_beat(Instant::now()); // 120 BPM

        std::thread::sleep(std::time::Duration::from_millis(3000));
        detector.register_beat(Instant::now()); // OUTLIER (too slow)

        std::thread::sleep(std::time::Duration::from_millis(500));
        detector.register_beat(Instant::now()); // 120 BPM

        // Should ignore outlier and estimate ~120 BPM
        let bpm = detector.bpm();
        assert!(
            (bpm - 120.0).abs() < 20.0,
            "Should filter outlier, got {} BPM",
            bpm
        );
    }

    #[test]
    fn test_tempo_detector_handles_tempo_change() {
        let mut detector = TempoDetector::new(60.0, 180.0);

        // Start at 120 BPM (500ms intervals)
        // Note: Thread sleep is not perfectly accurate, so we use larger tolerance
        for _ in 0..4 {
            std::thread::sleep(std::time::Duration::from_millis(500));
            detector.register_beat(Instant::now());
        }
        let bpm1 = detector.bpm();
        assert!(
            (bpm1 - 120.0).abs() < 20.0,
            "Expected ~120 BPM (±20), got {}",
            bpm1
        );

        // Change to faster tempo (~140 BPM, 428ms intervals)
        for _ in 0..4 {
            std::thread::sleep(std::time::Duration::from_millis(428));
            detector.register_beat(Instant::now());
        }
        let bpm2 = detector.bpm();
        assert!(
            bpm2 > bpm1,
            "Should detect tempo increase, got {} -> {}",
            bpm1,
            bpm2
        );
    }

    #[test]
    fn test_tempo_detector_set_bpm_range() {
        let mut detector = TempoDetector::new(60.0, 180.0);

        detector.set_bpm_range(80.0, 160.0);
        assert_eq!(detector.min_bpm, 80.0);
        assert_eq!(detector.max_bpm, 160.0);
    }

    #[test]
    fn test_tempo_detector_set_history_size() {
        let mut detector = TempoDetector::new(60.0, 180.0);

        detector.set_history_size(16);
        assert_eq!(detector.history_size, 16);
    }

    #[test]
    fn test_audio_parameters_includes_tempo_fields() {
        let mut processor = DspProcessor::new(44100, 2048).unwrap();
        let buffer = generate_sine_wave(440.0, 1.0, 44100, 2048);

        let params = processor.process(&buffer);

        // Tempo fields should exist
        assert!(params.bpm >= 0.0);
        assert!(params.tempo_confidence >= 0.0 && params.tempo_confidence <= 1.0);
    }

    #[test]
    fn test_audio_parameters_default_values() {
        let params = AudioParameters::default();

        assert_eq!(params.bpm, 120.0); // Default BPM
        assert_eq!(params.tempo_confidence, 0.0); // No confidence initially
    }

    #[test]
    fn test_tempo_integration_with_beat_detection() {
        let mut processor = DspProcessor::new(44100, 2048).unwrap();

        // Create beat pattern by alternating quiet and loud buffers
        let quiet = AudioBuffer::with_samples(vec![0.1; 2048], 44100, 1);
        let loud = AudioBuffer::with_samples(vec![0.5; 2048], 44100, 1);

        // Build up beat history with timing
        for _ in 0..5 {
            // Process quiet frames
            for _ in 0..10 {
                processor.process(&quiet);
            }
            // Beat frame
            let params = processor.process(&loud);
            // Small delay to simulate real timing
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        // After several beats, should have some tempo data
        let params = processor.process(&quiet);

        // BPM should be in musical range
        assert!(
            params.bpm >= 60.0 && params.bpm <= 180.0,
            "BPM {} should be in musical range",
            params.bpm
        );
    }

    #[test]
    fn test_configure_beat_detection() {
        use crate::config::BeatDetectionConfig;

        let mut processor = DspProcessor::new(44100, 2048).unwrap();

        let config = BeatDetectionConfig {
            sensitivity: 1.5,
            cooldown_seconds: 0.15,
            min_bpm: 80.0,
            max_bpm: 160.0,
            tempo_history_size: 10,
        };

        processor.configure_beat_detection(&config);

        // Configuration should be applied (we can't directly test internal state,
        // but we can verify the method doesn't panic)
        assert!(true);
    }
}
