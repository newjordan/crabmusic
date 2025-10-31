# [VIZ-007] Enhanced Oscilloscope Visualizer

**Epic**: Visualization Engine
**Priority**: P1 (Post-MVP Enhancement)
**Estimated Effort**: 2-3 days
**Status**: ✅ Completed

---

## Description

Upgrade the oscilloscope visualizer from synthetic waveform generation to **true waveform visualization** using real audio samples. The current implementation synthesizes waveforms from bass/mid/treble parameters, resulting in an artificial representation that doesn't show the actual audio waveform shape. A proper oscilloscope should display the real audio signal captured from the system.

**Current Limitation** (src/visualization/oscilloscope.rs:107-108):
> "This is a simplified approach - ideally we'd have access to raw audio samples."

**The Problem**:
- DspProcessor receives raw audio samples in AudioBuffer
- AudioParameters only exposes derived values (bass, mid, treble, amplitude)
- Raw waveform data is discarded after processing
- OscilloscopeVisualizer generates fake waveforms using sine synthesis
- Visualization doesn't represent actual audio signal

**The Solution**:
- Extend AudioParameters to include waveform samples
- Update DspProcessor::process() to include downsampled waveform
- Rewrite OscilloscopeVisualizer to use real audio samples
- Add trigger detection for stable waveform display
- Implement intelligent downsampling for efficient visualization

**Visual Quality Goal**: Oscilloscope should display the actual waveform shape - showing kick drums as sharp transients, vocals as complex patterns, and sine waves as smooth curves - just like a real oscilloscope.

---

## Acceptance Criteria

- [ ] AudioParameters includes waveform samples
  - [ ] Add `waveform: Vec<f32>` field to AudioParameters struct
  - [ ] Field contains downsampled audio samples suitable for visualization
  - [ ] Waveform length is configurable (default: 512 samples)
  - [ ] Values are normalized to -1.0 to 1.0 range
- [ ] DspProcessor populates waveform field
  - [ ] DspProcessor::process() includes waveform in returned AudioParameters
  - [ ] Waveform extracted from mono-mixed audio buffer
  - [ ] Intelligent downsampling applied if buffer > target length
  - [ ] Maintains waveform shape fidelity (no artificial smoothing)
- [ ] OscilloscopeVisualizer uses real waveform data
  - [ ] Remove generate_waveform_sample() synthetic generation
  - [ ] Use AudioParameters::waveform directly for display
  - [ ] Handle variable waveform lengths gracefully
  - [ ] Support scrolling/updating display as new samples arrive
- [ ] Trigger detection for stable display
  - [ ] Implement zero-crossing trigger detection
  - [ ] Configurable trigger level (-1.0 to 1.0)
  - [ ] Configurable trigger slope (positive/negative/both)
  - [ ] Prevents waveform from "scrolling" when stable signal present
- [ ] Visual enhancements
  - [ ] Accurate waveform rendering with proper interpolation
  - [ ] Configurable time scale (zoom in/out)
  - [ ] Grid lines for reference (optional)
  - [ ] Proper handling of silence (flat line at center)
- [ ] Configuration options
  - [ ] OscilloscopeConfig includes: waveform_length, trigger_enabled, trigger_level, trigger_slope, time_scale, show_grid
  - [ ] Reasonable defaults: 512 samples, trigger enabled, 0.0 level, positive slope
- [ ] Performance requirements
  - [ ] No perceptible performance impact from waveform data
  - [ ] Rendering stays at 60 FPS
  - [ ] Memory usage remains reasonable (waveform ~2KB per frame)
- [ ] Testing
  - [ ] Unit tests for downsampling algorithm
  - [ ] Unit tests for trigger detection
  - [ ] Integration test with real audio shows actual waveform shape
  - [ ] Visual test: sine wave shows smooth curve, not synthetic pattern
- [ ] Documentation
  - [ ] Code comments explain downsampling and trigger algorithms
  - [ ] Examples showing how to configure oscilloscope
  - [ ] AudioParameters waveform field documented

---

## Technical Approach

### Phase 1: Extend AudioParameters

**File**: `src/dsp/mod.rs`

Add waveform field to AudioParameters:

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
    /// Example: For a 1kHz sine wave, this would show one complete cycle
    /// with smooth positive and negative swings.
    pub waveform: Vec<f32>,
}
```

Update DspProcessor to include waveform:

```rust
impl DspProcessor {
    /// Downsample audio buffer to target length for oscilloscope display
    ///
    /// Uses intelligent downsampling that preserves waveform shape.
    /// For small buffers: returns as-is (upsampling not needed for oscilloscope)
    /// For large buffers: downsamples using linear interpolation
    ///
    /// # Arguments
    /// * `buffer` - Audio buffer (may be stereo, will be mono-mixed)
    /// * `target_length` - Desired output length (typically 512)
    ///
    /// # Returns
    /// Downsampled waveform normalized to -1.0 to 1.0
    fn downsample_for_waveform(&self, buffer: &AudioBuffer, target_length: usize) -> Vec<f32> {
        // Convert stereo to mono by averaging channels
        let mono_samples = if buffer.channels == 2 {
            buffer.samples
                .chunks_exact(2)
                .map(|chunk| (chunk[0] + chunk[1]) / 2.0)
                .collect::<Vec<f32>>()
        } else {
            buffer.samples.clone()
        };

        let input_len = mono_samples.len();

        if input_len == 0 {
            return vec![0.0; target_length];
        }

        // If input is already small enough, return as-is
        if input_len <= target_length {
            return mono_samples;
        }

        // Downsample using decimation with averaging
        // Take every Nth sample where N = input_len / target_length
        let step = input_len as f32 / target_length as f32;
        let mut output = Vec::with_capacity(target_length);

        for i in 0..target_length {
            let idx = (i as f32 * step) as usize;
            // Average a few samples around this point for smoother result
            let start = idx.saturating_sub(1);
            let end = (idx + 2).min(input_len);
            let avg = mono_samples[start..end].iter().sum::<f32>() / (end - start) as f32;
            output.push(avg.max(-1.0).min(1.0)); // Normalize
        }

        output
    }

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

        // 5. Extract waveform for oscilloscope (NEW)
        let waveform = self.downsample_for_waveform(buffer, 512);

        AudioParameters {
            bass,
            mid,
            treble,
            amplitude,
            beat,
            spectrum,
            waveform, // NEW: Include waveform
        }
    }
}
```

**Memory Impact**: ~2KB per frame (512 samples * 4 bytes/f32). Negligible for modern systems.

---

### Phase 2: Trigger Detection

**File**: `src/visualization/oscilloscope.rs`

Implement trigger detection for stable waveform display:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TriggerSlope {
    Positive,  // Trigger on rising edge
    Negative,  // Trigger on falling edge
    Both,      // Trigger on either edge
}

impl OscilloscopeVisualizer {
    /// Find trigger point in waveform for stable display
    ///
    /// Searches for a point where the waveform crosses the trigger level
    /// in the specified direction. This makes periodic signals appear stable
    /// instead of scrolling.
    ///
    /// # Arguments
    /// * `waveform` - Audio waveform samples
    /// * `trigger_level` - Level to trigger on (-1.0 to 1.0)
    /// * `trigger_slope` - Edge direction to trigger on
    ///
    /// # Returns
    /// Index in waveform where trigger occurs, or 0 if no trigger found
    fn find_trigger_point(
        &self,
        waveform: &[f32],
        trigger_level: f32,
        trigger_slope: TriggerSlope,
    ) -> usize {
        if waveform.len() < 2 {
            return 0;
        }

        // Search for trigger crossing in first half of waveform
        // (keep second half for display after trigger)
        let search_len = waveform.len() / 2;

        for i in 1..search_len {
            let prev = waveform[i - 1];
            let curr = waveform[i];

            let is_rising = prev < trigger_level && curr >= trigger_level;
            let is_falling = prev > trigger_level && curr <= trigger_level;

            let triggered = match trigger_slope {
                TriggerSlope::Positive => is_rising,
                TriggerSlope::Negative => is_falling,
                TriggerSlope::Both => is_rising || is_falling,
            };

            if triggered {
                return i;
            }
        }

        // No trigger found - return 0 (freerun mode)
        0
    }
}
```

---

### Phase 3: Enhanced OscilloscopeVisualizer

**File**: `src/visualization/oscilloscope.rs`

Update configuration:

```rust
#[derive(Debug, Clone)]
pub struct OscilloscopeConfig {
    /// Number of samples to display (time window)
    pub sample_count: usize,
    /// Amplitude sensitivity multiplier
    pub amplitude_sensitivity: f32,
    /// Smoothing factor (0.0-1.0, higher = smoother)
    pub smoothing_factor: f32,
    /// Line thickness in rows
    pub line_thickness: f32,
    /// Enable trigger for stable display
    pub trigger_enabled: bool,
    /// Trigger level (-1.0 to 1.0)
    pub trigger_level: f32,
    /// Trigger slope
    pub trigger_slope: TriggerSlope,
    /// Show reference grid
    pub show_grid: bool,
}

impl Default for OscilloscopeConfig {
    fn default() -> Self {
        Self {
            sample_count: 512,  // Display all samples from waveform
            amplitude_sensitivity: 1.5,
            smoothing_factor: 0.1,  // Less smoothing for accurate waveform
            line_thickness: 2.0,
            trigger_enabled: true,
            trigger_level: 0.0,  // Zero-crossing trigger
            trigger_slope: TriggerSlope::Positive,
            show_grid: true,
        }
    }
}
```

Update visualizer implementation:

```rust
pub struct OscilloscopeVisualizer {
    /// Current waveform (real audio samples, not synthetic)
    waveform: Vec<f32>,
    /// Configuration
    config: OscilloscopeConfig,
    /// Beat flash effect (0.0-1.0, decays over time)
    beat_flash: f32,
}

impl OscilloscopeVisualizer {
    pub fn new(config: OscilloscopeConfig) -> Self {
        let waveform = vec![0.0; config.sample_count];
        Self {
            waveform,
            config,
            beat_flash: 0.0,
        }
    }
}

impl Visualizer for OscilloscopeVisualizer {
    fn update(&mut self, params: &AudioParameters) {
        // Use real waveform from audio parameters
        if !params.waveform.is_empty() {
            // Find trigger point for stable display
            let trigger_offset = if self.config.trigger_enabled {
                self.find_trigger_point(
                    &params.waveform,
                    self.config.trigger_level,
                    self.config.trigger_slope,
                )
            } else {
                0
            };

            // Extract waveform starting from trigger point
            let waveform_len = params.waveform.len();
            let display_len = self.config.sample_count.min(waveform_len - trigger_offset);

            // Apply gentle smoothing to reduce noise (but preserve shape)
            for i in 0..display_len {
                let source_idx = trigger_offset + i;
                let new_value = params.waveform[source_idx];

                // Smooth with previous value to reduce jitter
                let smoothing = self.config.smoothing_factor;
                if i < self.waveform.len() {
                    self.waveform[i] = lerp(self.waveform[i], new_value, 1.0 - smoothing);
                }
            }

            // Fill remaining with zeros if waveform is shorter than display
            for i in display_len..self.waveform.len() {
                self.waveform[i] = lerp(self.waveform[i], 0.0, 1.0 - self.config.smoothing_factor);
            }
        } else {
            // No waveform data - fade to zero
            for sample in &mut self.waveform {
                *sample *= 0.9;
            }
        }

        // Handle beat flash effect
        if params.beat {
            self.beat_flash = 1.0;
        } else {
            self.beat_flash *= 0.85;
        }
    }

    fn render(&self, grid: &mut GridBuffer) {
        grid.clear();

        let width = grid.width();
        let height = grid.height();
        let center_y = height / 2;

        // Draw reference grid if enabled
        if self.config.show_grid {
            // Center line
            for x in 0..width {
                if x % 8 == 0 {
                    grid.set_cell(x, center_y, '┼');
                } else if x % 4 == 0 {
                    grid.set_cell(x, center_y, '┬');
                } else {
                    grid.set_cell(x, center_y, '·');
                }
            }

            // Top and bottom reference lines
            let quarter_y_top = height / 4;
            let quarter_y_bottom = (height * 3) / 4;
            for x in (0..width).step_by(8) {
                grid.set_cell(x, quarter_y_top, '·');
                grid.set_cell(x, quarter_y_bottom, '·');
            }
        }

        // Render waveform
        for y in 0..height {
            for x in 0..width {
                let coverage = self.calculate_coverage(x, y, width, height);
                if coverage > 0.1 {
                    let character = select_character_for_coverage(coverage);
                    grid.set_cell(x, y, character);
                }
            }
        }
    }

    fn name(&self) -> &str {
        "Oscilloscope"
    }
}
```

Update calculate_coverage to use real waveform:

```rust
fn calculate_coverage(&self, x: usize, y: usize, width: usize, height: usize) -> f32 {
    // Map x to waveform sample index
    let sample_idx = (x as f32 / width as f32 * self.waveform.len() as f32) as usize;
    if sample_idx >= self.waveform.len() {
        return 0.0;
    }

    // Get REAL waveform value at this x position (-1.0 to 1.0)
    let waveform_value = self.waveform[sample_idx];

    // Convert waveform value to y position (normalized 0.0 to 1.0)
    let waveform_y = 0.5 - (waveform_value * self.config.amplitude_sensitivity * 0.45);

    // Normalize y coordinate
    let norm_y = y as f32 / height as f32;

    // Calculate distance from waveform line
    let distance = (norm_y - waveform_y).abs();

    // Convert distance to coverage based on thickness
    let half_thickness = self.config.line_thickness / height as f32 / 2.0;

    let base_coverage = if distance < half_thickness {
        1.0 // Inside the line
    } else if distance < half_thickness * 2.0 {
        // Edge anti-aliasing
        1.0 - (distance - half_thickness) / half_thickness
    } else {
        0.0 // Outside
    };

    // Apply beat flash
    let flash_boost = self.beat_flash * 0.3;
    (base_coverage + flash_boost).min(1.0)
}
```

---

## Dependencies

- **Depends on**:
  - DSP-001 (DspProcessor exists)
  - VIZ-001 (GridBuffer exists)
  - VIZ-004 (Visualizer trait defined)
  - VIZ-006 (Sets pattern for extending AudioParameters)
  - Current oscilloscope implementation provides foundation
- **Blocks**: None (this is an enhancement)
- **Enables**: Real-time waveform monitoring, better debugging of audio issues

---

## Architecture References

- **DSP Component**: docs/architecture/README.md - DSP Processing section
- **Visualization Engine**: docs/architecture/README.md - Visualization component
- **Source Tree**: docs/architecture/source-tree.md - visualization/ module
- **Coding Standards**: docs/architecture/coding-standards.md - Rust style guide
- **Tech Stack**: docs/architecture/tech-stack.md - rustfft crate

---

## Testing Requirements

### Unit Tests

**File**: `src/visualization/oscilloscope.rs` (tests module)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trigger_detection_positive_slope() {
        let config = OscilloscopeConfig {
            trigger_enabled: true,
            trigger_level: 0.0,
            trigger_slope: TriggerSlope::Positive,
            ..Default::default()
        };
        let viz = OscilloscopeVisualizer::new(config);

        // Waveform that crosses zero upward at index 10
        let mut waveform = vec![-0.5; 512];
        for i in 10..512 {
            waveform[i] = 0.5;
        }

        let trigger = viz.find_trigger_point(&waveform, 0.0, TriggerSlope::Positive);
        assert_eq!(trigger, 10);
    }

    #[test]
    fn test_trigger_detection_negative_slope() {
        let config = OscilloscopeConfig {
            trigger_enabled: true,
            trigger_level: 0.0,
            trigger_slope: TriggerSlope::Negative,
            ..Default::default()
        };
        let viz = OscilloscopeVisualizer::new(config);

        // Waveform that crosses zero downward at index 20
        let mut waveform = vec![0.5; 512];
        for i in 20..512 {
            waveform[i] = -0.5;
        }

        let trigger = viz.find_trigger_point(&waveform, 0.0, TriggerSlope::Negative);
        assert_eq!(trigger, 20);
    }

    #[test]
    fn test_no_trigger_freerun() {
        let config = OscilloscopeConfig::default();
        let viz = OscilloscopeVisualizer::new(config);

        // Waveform that never crosses trigger level
        let waveform = vec![0.8; 512];

        let trigger = viz.find_trigger_point(&waveform, 0.0, TriggerSlope::Positive);
        assert_eq!(trigger, 0); // Freerun mode
    }

    #[test]
    fn test_real_waveform_update() {
        let mut viz = OscilloscopeVisualizer::new(OscilloscopeConfig::default());

        // Create sine wave as real waveform
        let mut waveform = Vec::new();
        for i in 0..512 {
            let t = i as f32 / 512.0;
            waveform.push((t * 2.0 * std::f32::consts::PI * 4.0).sin());
        }

        let params = AudioParameters {
            waveform,
            amplitude: 0.5,
            beat: false,
            ..Default::default()
        };

        viz.update(&params);

        // Waveform should now contain real sine wave data
        assert!(viz.waveform.iter().any(|&s| s > 0.5));
        assert!(viz.waveform.iter().any(|&s| s < -0.5));
    }
}
```

### Integration Tests

**File**: `tests/oscilloscope_integration_test.rs`

```rust
use crabmusic::dsp::DspProcessor;
use crabmusic::audio::AudioBuffer;
use crabmusic::visualization::{OscilloscopeVisualizer, OscilloscopeConfig, Visualizer, GridBuffer};

#[test]
fn test_oscilloscope_with_real_audio() {
    // Create DSP processor
    let mut dsp = DspProcessor::new(44100, 2048).unwrap();

    // Create oscilloscope visualizer
    let config = OscilloscopeConfig::default();
    let mut viz = OscilloscopeVisualizer::new(config);

    // Generate 440 Hz sine wave (A note)
    let mut buffer = AudioBuffer::new(2048, 44100, 1);
    for i in 0..2048 {
        let t = i as f32 / 44100.0;
        buffer.samples.push((2.0 * std::f32::consts::PI * 440.0 * t).sin());
    }

    // Process and visualize
    let params = dsp.process(&buffer);
    viz.update(&params);

    // Waveform should show sine wave pattern
    // Check that we have positive and negative values (not flat)
    assert!(viz.waveform.iter().any(|&s| s > 0.5));
    assert!(viz.waveform.iter().any(|&s| s < -0.5));

    // Render and verify output
    let mut grid = GridBuffer::new(80, 24);
    viz.render(&mut grid);

    // Should have visualization content
    let mut has_content = false;
    for y in 0..grid.height() {
        for x in 0..grid.width() {
            if grid.get_cell(x, y).character != ' ' && grid.get_cell(x, y).character != '·' {
                has_content = true;
                break;
            }
        }
    }
    assert!(has_content);
}

#[test]
fn test_oscilloscope_shows_different_waveforms() {
    let mut dsp = DspProcessor::new(44100, 2048).unwrap();
    let config = OscilloscopeConfig::default();
    let mut viz = OscilloscopeVisualizer::new(config);

    // Test 1: Sine wave (smooth)
    let mut sine_buffer = AudioBuffer::new(2048, 44100, 1);
    for i in 0..2048 {
        let t = i as f32 / 44100.0;
        sine_buffer.samples.push((2.0 * std::f32::consts::PI * 440.0 * t).sin());
    }
    let sine_params = dsp.process(&sine_buffer);

    // Test 2: Square wave (sharp transitions)
    let mut square_buffer = AudioBuffer::new(2048, 44100, 1);
    for i in 0..2048 {
        let t = i as f32 / 44100.0;
        let phase = (440.0 * t) % 1.0;
        square_buffer.samples.push(if phase < 0.5 { 1.0 } else { -1.0 });
    }
    let square_params = dsp.process(&square_buffer);

    // Waveforms should be different
    assert_ne!(sine_params.waveform, square_params.waveform);
}
```

### Manual Testing

```bash
# Build and run with oscilloscope mode
cargo run --release -- --loopback

# Once running:
# 1. Press 'V' until you reach oscilloscope mode
# 2. Play different types of audio:
#    - Pure tone (sine wave): Should show smooth sinusoidal curve
#    - Drum hit: Should show sharp transient spike
#    - Vocals: Should show complex waveform pattern
#    - Silence: Should show flat line at center
# 3. Verify trigger keeps waveform stable (not scrolling)
# 4. Test that grid lines appear correctly
# 5. Test different sensitivity levels
```

---

## Notes for AI Agent

**This enhancement transforms the oscilloscope from "synthetic demo" to "real-time signal monitor".**

### Implementation Order

1. **Phase 1**: Extend AudioParameters (easy, low risk)
   - Add waveform field to struct
   - Implement downsample_for_waveform() method
   - Update DspProcessor::process() to populate it
   - Run tests to ensure no regressions

2. **Phase 2**: Implement trigger detection (core algorithm)
   - Write find_trigger_point() method
   - Write unit tests for trigger detection
   - Verify trigger works for sine waves and complex signals

3. **Phase 3**: Update OscilloscopeVisualizer (main work)
   - Remove generate_waveform_sample() synthetic generation
   - Update update() to use real waveform from AudioParameters
   - Add trigger logic to find stable display point
   - Update rendering for grid lines
   - Tune configuration defaults

4. **Phase 4**: Testing and validation
   - Run unit and integration tests
   - Manual testing with real music
   - Verify different waveforms look different
   - Ensure trigger provides stable display
   - Confirm 60 FPS performance maintained

### Key Implementation Details

**Downsampling Approach**:
- AudioBuffer may contain 2048 samples (46ms at 44.1kHz)
- Oscilloscope display only needs ~512 samples (visual resolution)
- Use decimation with averaging to prevent aliasing
- Formula: `output[i] = avg(input[i*step..i*step+window])`
- Preserves waveform shape without artificial smoothing

**Trigger Detection**:
- Essential for stable display of periodic signals
- Zero-crossing trigger (level = 0.0) works for most music
- Search first half of waveform for trigger point
- Display from trigger point forward
- If no trigger found, use "freerun" mode (start at 0)
- Prevents waveform from appearing to "scroll"

**Mono Mixing**:
- Stereo audio has interleaved samples [L, R, L, R, ...]
- Oscilloscope shows single waveform, not stereo
- Mix to mono: `mono[i] = (left[i] + right[i]) / 2.0`
- Simpler than trying to display two channels

**Performance Considerations**:
- Waveform field adds ~2KB per frame (512 samples * 4 bytes)
- Downsampling is O(n) where n = buffer length (~2048)
- Trigger detection is O(n/2) search through half the waveform
- All operations are fast (<1ms total)
- Should have ZERO performance impact

**Visual Quality Tuning**:
- **Smoothing** (0.1 default): Lower than spectrum analyzer - we want accurate waveform
- **Amplitude sensitivity** (1.5 default): Similar to before, adjust for display scale
- **Line thickness** (2.0 default): Makes thin waveforms visible
- **Trigger level** (0.0 default): Zero-crossing is most stable for music
- Tune based on visual appearance with real audio

**Trigger Slope Options**:
- **Positive**: Trigger when waveform crosses level going up
- **Negative**: Trigger when waveform crosses level going down
- **Both**: Trigger on either direction (fastest lock)
- Positive is most common for oscilloscopes

**Common Pitfalls**:
- Don't forget to normalize waveform values to -1.0 to 1.0
- Handle empty waveform gracefully (fade to zero)
- Stereo-to-mono conversion must happen before downsampling
- Trigger search should only look in first half (need second half for display)
- Grid rendering should happen before waveform (background layer)

**Success Indicators**:
1. Playing 440 Hz sine wave shows smooth sinusoidal pattern (not synthetic steps)
2. Drum hits show sharp transient spikes (not smooth curves)
3. Vocals show complex, non-repeating patterns
4. Waveform stays stable with trigger enabled (doesn't scroll)
5. Different audio sources produce visibly different waveforms
6. Silence shows flat line at center (not noise)
7. Performance remains at 60 FPS

**Differences from VIZ-006 (Spectrum Analyzer)**:
- Spectrum used FFT frequency data → Oscilloscope uses time-domain samples
- Spectrum needed logarithmic mapping → Oscilloscope uses linear time mapping
- Spectrum used magnitude values → Oscilloscope uses signed amplitude values
- Both extend AudioParameters in similar way

**Time Estimate**: 2-3 days
- Day 1: Extend AudioParameters, implement downsampling, unit tests
- Day 2: Implement trigger detection, update OscilloscopeVisualizer
- Day 3: Testing, tuning, validation with real audio

**Validation Checklist**:
- [ ] Sine wave shows smooth curve (not stepped/synthetic)
- [ ] Square wave shows sharp transitions
- [ ] Kick drum shows transient spike
- [ ] Waveform is stable (not scrolling) with trigger enabled
- [ ] Grid lines visible and properly positioned
- [ ] Different audio produces different waveforms
- [ ] Performance at 60 FPS maintained

This enhancement brings the oscilloscope visualization to professional quality, showing actual audio waveforms in real-time!
