# [VIZ-006] Enhanced Spectrum Analyzer

**Epic**: Visualization Engine
**Priority**: P1 (Post-MVP Enhancement)
**Estimated Effort**: 3-4 days
**Status**: Not Started

---

## Description

Upgrade the spectrum analyzer visualizer from a simplified 3-band display to a **true frequency spectrum analyzer** using real FFT data. The current implementation only uses bass/mid/treble parameters, resulting in a visually limited representation with just 3 distinct regions. A proper spectrum analyzer should display dozens of frequency bars across the entire audible spectrum.

**Current Limitation** (src/visualization/spectrum.rs:97):
> "This is a simplified approach - ideally we'd have access to the full FFT spectrum."

**The Problem**:
- DspProcessor already computes full FFT spectrum (1024 bins for 2048 window)
- AudioParameters only exposes 3 bands (bass, mid, treble)
- Full spectrum data is discarded after band extraction
- SpectrumVisualizer has no access to real frequency data

**The Solution**:
- Extend AudioParameters to include full frequency spectrum
- Update DspProcessor::process() to return spectrum data
- Rewrite SpectrumVisualizer to use real FFT bins
- Add logarithmic frequency scaling for perceptually accurate display
- Add visual enhancements (peak hold, smoother bars, better spacing)

**Visual Quality Goal**: Spectrum should look like a professional audio analyzer with smooth, responsive bars that accurately represent the music's frequency content.

---

## Acceptance Criteria

- [ ] AudioParameters includes frequency spectrum data
  - [ ] Add `spectrum: Vec<f32>` field to AudioParameters struct
  - [ ] Field contains normalized magnitude values (0.0-1.0) for each FFT bin
  - [ ] Spectrum length is window_size / 2 (e.g., 1024 for 2048 window)
- [ ] DspProcessor populates spectrum field
  - [ ] DspProcessor::process() includes spectrum in returned AudioParameters
  - [ ] Spectrum data comes from existing process_buffer() output
  - [ ] Spectrum is properly normalized to 0.0-1.0 range
- [ ] SpectrumVisualizer uses real FFT data
  - [ ] Remove synthetic bass/mid/treble bar generation
  - [ ] Map frequency bins to visual bars using logarithmic scaling
  - [ ] Support configurable bar count (16-128 bars)
  - [ ] Support configurable frequency range (default: 20 Hz - 20 kHz)
- [ ] Logarithmic frequency scaling implemented
  - [ ] Bars represent perceptually equal frequency ranges
  - [ ] More bars in bass/mid range (where humans are sensitive)
  - [ ] Fewer bars in treble range
  - [ ] Use log scale: `f(i) = f_min * (f_max/f_min)^(i/N)`
- [ ] Visual enhancements
  - [ ] Smooth bar animations (no jitter)
  - [ ] Peak hold indicators (optional bright character at peak)
  - [ ] Configurable bar spacing (0-2 character gaps)
  - [ ] Proper amplitude scaling with configurable sensitivity
- [ ] Configuration options
  - [ ] SpectrumConfig includes: bar_count, freq_min, freq_max, smoothing_factor, amplitude_sensitivity, bar_spacing, peak_hold_enabled, peak_decay_rate
  - [ ] Reasonable defaults: 32 bars, 20-20000 Hz, 0.7 smoothing
- [ ] Performance requirements
  - [ ] No perceptible performance impact from spectrum data
  - [ ] Rendering stays at 60 FPS
  - [ ] Memory usage remains reasonable (spectrum ~4KB per frame)
- [ ] Testing
  - [ ] Unit tests for logarithmic frequency mapping
  - [ ] Unit tests for bin-to-bar aggregation
  - [ ] Integration test with synthetic audio (sine sweep)
  - [ ] Visual test with real music shows expected frequency peaks
- [ ] Documentation
  - [ ] Code comments explain logarithmic scaling algorithm
  - [ ] Examples showing how to configure spectrum analyzer
  - [ ] AudioParameters spectrum field documented

---

## Technical Approach

### Phase 1: Extend AudioParameters

**File**: `src/dsp/mod.rs`

Add spectrum field to AudioParameters:

```rust
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
}
```

Update DspProcessor::process() to include spectrum:

```rust
pub fn process(&mut self, buffer: &AudioBuffer) -> AudioParameters {
    // 1. Get FFT spectrum
    let spectrum = self.process_buffer(buffer);

    // 2. Extract frequency bands (existing code)
    let bass = self.extract_band(&spectrum, 20.0, 250.0);
    let mid = self.extract_band(&spectrum, 250.0, 4000.0);
    let treble = self.extract_band(&spectrum, 4000.0, 20000.0);

    // 3. Calculate overall amplitude (RMS)
    let amplitude = self.calculate_rms(&buffer.samples);

    // 4. Detect beat using energy-based onset detection
    let beat = self.beat_detector.detect(amplitude);

    AudioParameters {
        bass,
        mid,
        treble,
        amplitude,
        beat,
        spectrum,  // NEW: Include full spectrum
    }
}
```

**Memory Impact**: ~4KB per frame (1024 bins * 4 bytes/f32). Negligible for modern systems.

---

### Phase 2: Logarithmic Frequency Mapping

**File**: `src/visualization/spectrum.rs`

Key insight: Human hearing is logarithmic. An octave from 100-200 Hz sounds the same "distance" as 1000-2000 Hz, even though the latter is 10x more Hz. Use logarithmic scaling to map bars to frequency ranges.

```rust
impl SpectrumVisualizer {
    /// Map bar index to frequency range using logarithmic scaling
    ///
    /// # Arguments
    /// * `bar_index` - Visual bar index (0 to bar_count-1)
    ///
    /// # Returns
    /// Tuple of (freq_min, freq_max) in Hz for this bar
    ///
    /// # Examples
    /// For 32 bars spanning 20-20000 Hz:
    /// - Bar 0: 20-25 Hz (bass)
    /// - Bar 16: ~400-600 Hz (mid)
    /// - Bar 31: ~15000-20000 Hz (treble)
    fn bar_to_frequency_range(&self, bar_index: usize) -> (f32, f32) {
        let f_min = self.config.freq_min;
        let f_max = self.config.freq_max;
        let n = self.config.bar_count as f32;
        let i = bar_index as f32;

        // Logarithmic scaling: f(i) = f_min * (f_max/f_min)^(i/n)
        let ratio = f_max / f_min;
        let freq_start = f_min * ratio.powf(i / n);
        let freq_end = f_min * ratio.powf((i + 1.0) / n);

        (freq_start, freq_end)
    }

    /// Extract bar height from FFT spectrum
    ///
    /// Aggregates FFT bins within the bar's frequency range.
    ///
    /// # Arguments
    /// * `spectrum` - FFT magnitude spectrum from AudioParameters
    /// * `sample_rate` - Audio sample rate in Hz
    /// * `bar_index` - Visual bar index
    ///
    /// # Returns
    /// Normalized bar height (0.0-1.0)
    fn extract_bar_from_spectrum(
        &self,
        spectrum: &[f32],
        sample_rate: u32,
        bar_index: usize,
    ) -> f32 {
        let (freq_min, freq_max) = self.bar_to_frequency_range(bar_index);

        // Convert frequency range to FFT bin range
        let window_size = spectrum.len() * 2; // Spectrum is half of window
        let bin_min = (freq_min * window_size as f32 / sample_rate as f32).ceil() as usize;
        let bin_max = (freq_max * window_size as f32 / sample_rate as f32).floor() as usize;

        // Clamp to valid range
        let bin_min = bin_min.min(spectrum.len());
        let bin_max = bin_max.min(spectrum.len());

        if bin_min >= bin_max {
            return 0.0;
        }

        // Average magnitude in this frequency range
        let sum: f32 = spectrum[bin_min..bin_max].iter().sum();
        let count = (bin_max - bin_min) as f32;

        if count > 0.0 {
            (sum / count) * self.config.amplitude_sensitivity
        } else {
            0.0
        }
    }
}
```

---

### Phase 3: Enhanced SpectrumVisualizer

**File**: `src/visualization/spectrum.rs`

Update configuration:

```rust
#[derive(Debug, Clone)]
pub struct SpectrumConfig {
    /// Number of frequency bars to display
    pub bar_count: usize,
    /// Minimum frequency to display (Hz)
    pub freq_min: f32,
    /// Maximum frequency to display (Hz)
    pub freq_max: f32,
    /// Smoothing factor (0.0-1.0, higher = smoother)
    pub smoothing_factor: f32,
    /// Amplitude sensitivity multiplier
    pub amplitude_sensitivity: f32,
    /// Bar spacing (0 = no gap, 1 = one char gap, 2 = two char gap)
    pub bar_spacing: usize,
    /// Enable peak hold indicators
    pub peak_hold_enabled: bool,
    /// Peak hold decay rate (units per frame)
    pub peak_decay_rate: f32,
}

impl Default for SpectrumConfig {
    fn default() -> Self {
        Self {
            bar_count: 32,
            freq_min: 20.0,
            freq_max: 20000.0,
            smoothing_factor: 0.7,
            amplitude_sensitivity: 2.0,  // Boosted for visibility
            bar_spacing: 0,
            peak_hold_enabled: true,
            peak_decay_rate: 0.02,  // Slow decay = peaks stay visible
        }
    }
}
```

Update visualizer state:

```rust
pub struct SpectrumVisualizer {
    /// Current bar heights (smoothed, 0.0-1.0)
    bar_heights: Vec<f32>,
    /// Peak hold values for each bar
    peak_heights: Vec<f32>,
    /// Configuration
    config: SpectrumConfig,
    /// Sample rate (needed for frequency mapping)
    sample_rate: u32,
}

impl SpectrumVisualizer {
    pub fn new(config: SpectrumConfig, sample_rate: u32) -> Self {
        let bar_heights = vec![0.0; config.bar_count];
        let peak_heights = vec![0.0; config.bar_count];
        Self {
            bar_heights,
            peak_heights,
            config,
            sample_rate,
        }
    }
}
```

Update Visualizer trait implementation:

```rust
impl Visualizer for SpectrumVisualizer {
    fn update(&mut self, params: &AudioParameters) {
        // Extract bar heights from real FFT spectrum
        for i in 0..self.config.bar_count {
            let target_height = self.extract_bar_from_spectrum(
                &params.spectrum,
                self.sample_rate,
                i,
            );

            // Apply smoothing
            self.bar_heights[i] = lerp(
                self.bar_heights[i],
                target_height.min(1.0),
                self.config.smoothing_factor,
            );

            // Update peak hold
            if self.config.peak_hold_enabled {
                if self.bar_heights[i] > self.peak_heights[i] {
                    self.peak_heights[i] = self.bar_heights[i];
                } else {
                    // Decay peak slowly
                    self.peak_heights[i] = (self.peak_heights[i] - self.config.peak_decay_rate).max(0.0);
                }
            }
        }
    }

    fn render(&self, grid: &mut GridBuffer) {
        let bar_width = grid.width / self.config.bar_count;
        let spacing = self.config.bar_spacing;

        for i in 0..self.config.bar_count {
            let x_start = i * bar_width;
            let x_end = (x_start + bar_width).saturating_sub(spacing);

            // Calculate bar height in rows
            let bar_rows = (self.bar_heights[i] * grid.height as f32) as usize;
            let peak_row = (self.peak_heights[i] * grid.height as f32) as usize;

            // Render vertical bar (bottom to top)
            for y in 0..grid.height {
                let rows_from_bottom = grid.height - 1 - y;

                for x in x_start..x_end {
                    if rows_from_bottom < bar_rows {
                        // Filled portion of bar
                        grid.set_cell(x, y, '█');
                    } else if rows_from_bottom == peak_row && self.config.peak_hold_enabled {
                        // Peak indicator
                        grid.set_cell(x, y, '▬');
                    } else {
                        // Empty space
                        grid.set_cell(x, y, ' ');
                    }
                }
            }
        }
    }

    fn name(&self) -> &str {
        "Spectrum Analyzer"
    }
}
```

---

## Dependencies

- **Depends on**:
  - DSP-001 (DspProcessor exists and computes FFT)
  - VIZ-001 (GridBuffer exists)
  - VIZ-004 (Visualizer trait defined)
  - Current implementation has basic structure in place
- **Blocks**: None (this is an enhancement, not blocking)
- **Enables**: Better user experience, more professional visualizations

---

## Architecture References

- **DSP Component**: docs/architecture/README.md - DSP Processing section
- **Visualization Engine**: docs/architecture/README.md - Visualization component
- **Source Tree**: docs/architecture/source-tree.md - visualization/ module
- **Coding Standards**: docs/architecture/coding-standards.md - Rust style guide
- **Tech Stack**: docs/architecture/tech-stack.md - rustfft, spectrum-analyzer crates

---

## Testing Requirements

### Unit Tests

**File**: `src/visualization/spectrum.rs` (tests module)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logarithmic_frequency_mapping() {
        let config = SpectrumConfig::default();
        let viz = SpectrumVisualizer::new(config, 44100);

        // First bar should start at freq_min
        let (f_min, f_max) = viz.bar_to_frequency_range(0);
        assert!((f_min - 20.0).abs() < 1.0);

        // Last bar should end at freq_max
        let (f_min, f_max) = viz.bar_to_frequency_range(31);
        assert!((f_max - 20000.0).abs() < 100.0);

        // Middle bars should be logarithmically spaced
        // (each octave should have similar number of bars)
        let (f1_min, f1_max) = viz.bar_to_frequency_range(10);
        let (f2_min, f2_max) = viz.bar_to_frequency_range(20);

        // Ratio should be consistent (logarithmic property)
        let ratio1 = f1_max / f1_min;
        let ratio2 = f2_max / f2_min;
        assert!((ratio1 - ratio2).abs() < 0.1);
    }

    #[test]
    fn test_extract_bar_from_spectrum() {
        let config = SpectrumConfig::default();
        let viz = SpectrumVisualizer::new(config, 44100);

        // Create synthetic spectrum with peak at 1000 Hz
        let mut spectrum = vec![0.0; 1024];
        let peak_bin = (1000.0 * 2048.0 / 44100.0) as usize;
        spectrum[peak_bin] = 1.0;

        // Bar containing 1000 Hz should have highest value
        let mut max_bar = 0;
        let mut max_height = 0.0;

        for i in 0..32 {
            let height = viz.extract_bar_from_spectrum(&spectrum, 44100, i);
            if height > max_height {
                max_height = height;
                max_bar = i;
            }
        }

        // Verify the correct bar has the peak
        let (f_min, f_max) = viz.bar_to_frequency_range(max_bar);
        assert!(f_min <= 1000.0 && 1000.0 <= f_max);
    }

    #[test]
    fn test_peak_hold_behavior() {
        let config = SpectrumConfig {
            peak_hold_enabled: true,
            peak_decay_rate: 0.1,
            ..Default::default()
        };
        let mut viz = SpectrumVisualizer::new(config, 44100);

        // Create params with high energy
        let mut params = AudioParameters::default();
        params.spectrum = vec![1.0; 1024];

        viz.update(&params);
        let peak_after_high = viz.peak_heights[0];

        // Update with low energy
        params.spectrum = vec![0.1; 1024];
        viz.update(&params);
        let peak_after_low = viz.peak_heights[0];

        // Peak should decay but stay higher than current
        assert!(peak_after_low < peak_after_high);
        assert!(peak_after_low > viz.bar_heights[0]);
    }

    #[test]
    fn test_smoothing_prevents_jitter() {
        let config = SpectrumConfig {
            smoothing_factor: 0.3,
            ..Default::default()
        };
        let mut viz = SpectrumVisualizer::new(config, 44100);

        // Sudden change in spectrum
        let mut params = AudioParameters::default();
        params.spectrum = vec![0.0; 1024];
        viz.update(&params);

        params.spectrum = vec![1.0; 1024];
        viz.update(&params);

        // Height should move toward target but not instantly
        assert!(viz.bar_heights[0] > 0.0);
        assert!(viz.bar_heights[0] < 1.0);
    }
}
```

### Integration Tests

**File**: `tests/spectrum_integration_test.rs`

```rust
use crabmusic::dsp::DspProcessor;
use crabmusic::audio::AudioBuffer;
use crabmusic::visualization::{SpectrumVisualizer, SpectrumConfig, Visualizer, GridBuffer};

#[test]
fn test_spectrum_with_sine_sweep() {
    // Create DSP processor
    let mut dsp = DspProcessor::new(44100, 2048).unwrap();

    // Create spectrum visualizer
    let config = SpectrumConfig::default();
    let mut viz = SpectrumVisualizer::new(config, 44100);

    // Generate sine wave at 1000 Hz
    let mut buffer = AudioBuffer::new(2048, 44100, 1);
    for i in 0..2048 {
        let t = i as f32 / 44100.0;
        buffer.samples[i] = (2.0 * std::f32::consts::PI * 1000.0 * t).sin();
    }

    // Process and visualize
    let params = dsp.process(&buffer);
    viz.update(&params);

    // The bar containing 1000 Hz should be prominent
    // Find which bar that is
    let mut found_peak = false;
    for i in 0..viz.bar_heights.len() {
        let (f_min, f_max) = viz.bar_to_frequency_range(i);
        if f_min <= 1000.0 && 1000.0 <= f_max {
            assert!(viz.bar_heights[i] > 0.5, "Expected peak at 1000 Hz");
            found_peak = true;
        }
    }
    assert!(found_peak, "Should find bar containing 1000 Hz");
}
```

### Manual Testing

```bash
# Build and run with spectrum analyzer mode
cargo run --release -- --loopback

# Once running:
# 1. Press 'V' to switch to spectrum analyzer mode
# 2. Play music with distinct frequency content:
#    - Bass-heavy track: Should see activity in left bars
#    - Vocals/mids: Should see activity in middle bars
#    - Hi-hats/cymbals: Should see activity in right bars
# 3. Verify peak hold indicators appear and decay
# 4. Verify no jitter (smooth bar movements)
# 5. Test different sensitivity levels with +/- keys
```

---

## Notes for AI Agent

**This is a significant enhancement that transforms the spectrum analyzer from "demo quality" to "production quality".**

### Implementation Order

1. **Phase 1**: Extend AudioParameters (easy, low risk)
   - Add spectrum field to struct
   - Update DspProcessor::process() to populate it
   - Run tests to ensure no regressions

2. **Phase 2**: Implement logarithmic mapping (core algorithm)
   - Write `bar_to_frequency_range()` method
   - Write unit tests for frequency mapping
   - Verify logarithmic distribution is correct

3. **Phase 3**: Rewrite SpectrumVisualizer (main work)
   - Replace synthetic bar generation with real FFT extraction
   - Add peak hold feature
   - Update rendering for better visual quality
   - Tune configuration defaults

4. **Phase 4**: Testing and tuning
   - Run unit and integration tests
   - Manual testing with real music
   - Adjust sensitivity, smoothing, and decay rates
   - Ensure 60 FPS performance maintained

### Key Implementation Details

**Logarithmic Scaling Math**:
- Formula: `f(i) = f_min * (f_max/f_min)^(i/N)`
- This ensures each bar represents an octave-proportional range
- Example: 20-20000 Hz with 32 bars
  - Ratio = 20000/20 = 1000
  - Bar 0: 20 * 1000^(0/32) = 20 Hz
  - Bar 16: 20 * 1000^(16/32) ≈ 632 Hz
  - Bar 32: 20 * 1000^(32/32) = 20000 Hz
- More bars in bass/mid where humans are sensitive!

**Performance Considerations**:
- Spectrum field adds ~4KB per frame (negligible)
- Logarithmic calculations done once per bar per frame (32 bars * 60 fps = trivial)
- FFT computation already happening, we're just exposing the data
- Should have ZERO performance impact

**Visual Quality Tuning**:
- **Smoothing** (0.7 default): Higher = smoother but slightly laggy
- **Amplitude sensitivity** (2.0 default): Higher = more responsive but may clip
- **Peak decay** (0.02 default): Lower = peaks stay longer
- Tune these based on visual feel, not math!

**Bar Rendering Tips**:
- Use '█' for filled portions (solid block)
- Use '▬' for peak indicators (horizontal line)
- Consider character set integration (use current charset for bars)
- Bar spacing helps visual clarity but reduces resolution

**Common Pitfalls**:
- Don't forget to clamp bar heights to 0.0-1.0
- Handle empty frequency ranges gracefully (bin_min >= bin_max)
- Spectrum length is window_size / 2, not window_size
- Peak hold needs to check if enabled before rendering

**Success Indicators**:
1. Playing bass-heavy music shows tall bars on the left
2. Hi-hat sounds show activity on the right
3. Vocals show activity in middle bars
4. Bars move smoothly without jitter
5. Peak indicators help see transient sounds
6. Looks and feels like a professional spectrum analyzer

**Time Estimate**: 3-4 days
- Day 1: Extend AudioParameters, update DspProcessor, unit tests
- Day 2: Implement logarithmic mapping, extract_bar_from_spectrum
- Day 3: Rewrite SpectrumVisualizer, integrate peak hold, rendering
- Day 4: Testing, tuning, polish, documentation

**Reference Implementation**: If stuck, look at professional audio software like:
- Audacity's spectrum analyzer
- VLC media player's visualizer
- Any DAW spectrum analyzer

This is a high-impact story that will make the application feel much more professional!
