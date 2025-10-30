// DSP (Digital Signal Processing) module
// Handles FFT processing and audio parameter extraction

#![allow(dead_code)]

pub mod smoothing;
pub mod windowing;

use crate::audio::AudioBuffer;
use crate::error::DspError;
use rustfft::num_complex::Complex;
use rustfft::FftPlanner;

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

        Ok(Self {
            fft_planner,
            window_size,
            sample_rate,
            hann_window,
            scratch_buffer,
        })
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

        AudioParameters {
            bass,
            mid,
            treble,
            amplitude,
            beat: false, // TODO: Implement in DSP-004
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
}

/// Audio parameters extracted from DSP processing
///
/// Contains frequency band amplitudes, overall amplitude, and other
/// audio features used for visualization.
#[derive(Debug, Clone, Default)]
pub struct AudioParameters {
    /// Bass frequency band amplitude (20-250 Hz)
    pub bass: f32,

    /// Mid frequency band amplitude (250-4000 Hz)
    pub mid: f32,

    /// Treble frequency band amplitude (4000-20000 Hz)
    pub treble: f32,

    /// Overall amplitude (RMS)
    pub amplitude: f32,

    /// Beat detected (true if beat onset detected)
    pub beat: bool,
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
}
