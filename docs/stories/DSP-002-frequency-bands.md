# [DSP-002] Frequency Band Extraction

**Epic**: DSP Processing
**Priority**: P0 (Blocking)
**Estimated Effort**: 1-1.5 days
**Status**: Not Started

---

## Description

Implement frequency band extraction from FFT spectrum data to populate the AudioParameters struct. This extracts bass, mid, and treble frequency bands that drive visualization behavior.

**Agent Instructions**: Extend DspProcessor to:
- Extract bass (20-250 Hz), mid (250-4000 Hz), and treble (4000-20000 Hz) bands from spectrum
- Calculate overall amplitude (RMS)
- Populate AudioParameters struct
- Implement the process() method that combines FFT + band extraction

---

## Acceptance Criteria

- [ ] process() method implemented that calls process_buffer() and extracts bands
- [ ] Bass band extraction (20-250 Hz) working correctly
- [ ] Mid band extraction (250-4000 Hz) working correctly
- [ ] Treble band extraction (4000-20000 Hz) working correctly
- [ ] Overall amplitude calculated as RMS of all samples
- [ ] Band values normalized to 0.0-1.0 range
- [ ] Helper method to find bin range for frequency range
- [ ] Unit tests validate correct band extraction with synthetic audio
- [ ] Test with bass-heavy, mid-heavy, and treble-heavy signals

---

## Technical Approach

### Frequency Band Extraction

Reference: **docs/architecture.md - DSP Processing Component**

```rust
impl DspProcessor {
    /// Process audio buffer and extract audio parameters
    ///
    /// Performs FFT analysis and extracts frequency bands.
    ///
    /// # Arguments
    /// * `buffer` - Audio buffer to process
    ///
    /// # Returns
    /// AudioParameters with extracted frequency bands and amplitude
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
        let bin_min = (freq_min * self.window_size as f32 / self.sample_rate as f32).ceil() as usize;
        let bin_max = (freq_max * self.window_size as f32 / self.sample_rate as f32).floor() as usize;
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
```

---

## Dependencies

- **Depends on**:
  - DSP-001 (FFT processor and spectrum extraction)
- **Blocks**: VIZ-005 (sine wave visualizer needs audio parameters)

---

## Architecture References

- **Component Spec**: docs/architecture.md - "DSP Processing Component"
- **Frequency Bands**: config/default.yaml - bass_range, mid_range, treble_range
- **AudioParameters**: src/dsp/mod.rs - AudioParameters struct

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_bass_band() {
        // Generate 100 Hz sine wave (bass frequency)
        let buffer = generate_sine_wave(100.0, 1.0, 44100, 2048);
        let mut processor = DspProcessor::new(44100, 2048).unwrap();

        let params = processor.process(&buffer);

        // Bass should be dominant
        assert!(params.bass > 0.5, "Bass should be > 0.5, got {}", params.bass);
        assert!(params.bass > params.mid, "Bass should be > mid");
        assert!(params.bass > params.treble, "Bass should be > treble");
    }

    #[test]
    fn test_extract_mid_band() {
        // Generate 1000 Hz sine wave (mid frequency)
        let buffer = generate_sine_wave(1000.0, 1.0, 44100, 2048);
        let mut processor = DspProcessor::new(44100, 2048).unwrap();

        let params = processor.process(&buffer);

        // Mid should be dominant
        assert!(params.mid > 0.5, "Mid should be > 0.5, got {}", params.mid);
        assert!(params.mid > params.bass, "Mid should be > bass");
        assert!(params.mid > params.treble, "Mid should be > treble");
    }

    #[test]
    fn test_extract_treble_band() {
        // Generate 8000 Hz sine wave (treble frequency)
        let buffer = generate_sine_wave(8000.0, 1.0, 44100, 2048);
        let mut processor = DspProcessor::new(44100, 2048).unwrap();

        let params = processor.process(&buffer);

        // Treble should be dominant
        assert!(params.treble > 0.5, "Treble should be > 0.5, got {}", params.treble);
        assert!(params.treble > params.bass, "Treble should be > bass");
        assert!(params.treble > params.mid, "Treble should be > mid");
    }

    #[test]
    fn test_calculate_rms() {
        let processor = DspProcessor::new(44100, 2048).unwrap();

        // Test with known values
        let samples = vec![0.0, 1.0, 0.0, -1.0]; // RMS = sqrt((0 + 1 + 0 + 1) / 4) = sqrt(0.5) ≈ 0.707
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
```

---

## Notes for AI Agent

**Frequency Band Ranges**:
- **Bass**: 20-250 Hz (sub-bass and bass)
- **Mid**: 250-4000 Hz (midrange and presence)
- **Treble**: 4000-20000 Hz (brilliance and air)

These ranges are standard for music visualization and match the AudioParameters struct definition.

**Band Extraction Strategy**:
- Use average energy in frequency range (not sum) for normalization
- Spectrum is already normalized 0.0-1.0 from process_buffer()
- Band values will also be 0.0-1.0 range

**RMS Calculation**:
- RMS = sqrt(mean(samples^2))
- Provides overall loudness measure
- Used for amplitude-based visualization scaling

**Bin Calculation**:
- bin = frequency * window_size / sample_rate
- Example: 440 Hz @ 44100 Hz, 2048 window = bin 20.4 ≈ bin 20
- Use ceil() for min, floor() for max to stay within range

**Success Indicator**: Unit tests pass showing correct band isolation for bass/mid/treble frequencies

