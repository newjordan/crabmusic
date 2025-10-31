# [DSP-005] Spectral Flux Beat Detection - Brownfield Enhancement

**Epic**: DSP Processing
**Priority**: P2 (Enhancement to Existing Beat Detection)
**Estimated Effort**: 1-2 days
**Status**: Draft

---

## Description

Enhance the existing energy-based beat detection (DSP-004) with **spectral flux onset detection** for more accurate beat detection across diverse music styles. Spectral flux provides better detection of harmonic onsets (piano, guitar, vocals) where energy-based detection may miss subtle transients.

**Current System**: Energy-based beat detection (DSP-004) tracks RMS amplitude changes and works well for percussive content (drums, bass) but can miss harmonic onsets in complex music.

**The Enhancement**: Add spectral flux beat detection as a complementary approach that:
- Analyzes changes in the frequency spectrum over time
- Detects onsets in harmonic instruments (piano, guitar, strings)
- Works alongside energy-based detection for hybrid approach
- Maintains the same simple boolean beat output for visualizers
- Integrates seamlessly with existing DspProcessor pipeline

**Key Benefit**: More accurate beat detection for complex music with harmonic instruments, classical music, jazz, and acoustic genres where energy-based detection alone is insufficient.

---

## User Story

**As a** CrabMusic user,
**I want** more accurate beat detection for complex music with harmonic instruments,
**So that** visual effects sync correctly with piano onsets, guitar strums, and other harmonic transients, not just drum hits.

---

## Story Context

**Existing System Integration:**
- Integrates with: DspProcessor in `src/dsp/mod.rs`
- Technology: Rust with rustfft (already calculating FFT spectrum)
- Follows pattern: BeatDetector struct with detect() method (DSP-004)
- Touch points: DspProcessor::process(), AudioParameters

**Existing Beat Detection (DSP-004)**:
- Energy-based onset detection using RMS amplitude
- Works well for: drums, bass, percussive transients
- Limitations: Misses harmonic onsets, piano notes, guitar strums
- Current implementation: BeatDetector struct (lines 15-92 in src/dsp/mod.rs)

**Spectral Flux Approach**:
- Analyzes changes in frequency spectrum between frames
- Formula: `flux = Σ max(0, spectrum[i] - prev_spectrum[i])`
- Detects increases in spectral energy across frequency bins
- More sensitive to harmonic content than raw amplitude

---

## Acceptance Criteria

**Functional Requirements:**

1. SpectralFluxDetector struct created with spectrum history tracking
2. Spectral flux calculation implemented (sum of positive spectral differences)
3. Flux-based onset detection algorithm integrated
4. Configurable sensitivity parameter (default: 1.0)
5. Cooldown mechanism prevents false positives (shared 100ms cooldown)
6. Spectral history maintained with sliding window (2 frames minimum)
7. Hybrid detection mode combines energy + spectral flux
8. AudioParameters includes `beat_flux` field (boolean) for visualization
9. Existing energy-based beat detection continues to work unchanged
10. Default mode uses hybrid detection (energy OR flux)

**Integration Requirements:**

11. Existing beat detection (DSP-004) continues to work with no regressions
12. New SpectralFluxDetector follows BeatDetector pattern exactly
13. Integration with DspProcessor maintains current API (AudioParameters)
14. Braille character system used as default for all visualizations
15. No breaking changes to visualizer implementations

**Quality Requirements:**

16. Unit tests validate spectral flux calculation
17. Integration tests compare energy vs flux detection
18. Manual testing with harmonic instruments (piano, guitar) shows improved detection
19. Performance impact <1ms per frame (uses existing spectrum)
20. Documentation updated with algorithm explanation

---

## Technical Notes

**Integration Approach:**
- Create new SpectralFluxDetector struct parallel to BeatDetector
- Both detectors run in DspProcessor::process()
- Combine results: `beat = energy_beat || flux_beat` (hybrid mode)
- AudioParameters exposes both: `beat` (hybrid), `beat_flux` (spectral only)
- Uses existing `spectrum` field from AudioParameters (no additional FFT)

**Existing Pattern Reference:**
- Follow BeatDetector implementation (src/dsp/mod.rs:15-92)
- Use same cooldown mechanism (std::time::Instant)
- Similar history management (Vec with fixed size)
- Same sensitivity parameter pattern

**Key Constraints:**
- Must use existing FFT spectrum (no additional processing)
- Performance must remain <1ms overhead
- Must not break existing visualizers
- Braille rendering system is default for all visualizations

**Algorithm Overview:**
```rust
// Spectral flux formula
let flux = spectrum.iter()
    .zip(prev_spectrum.iter())
    .map(|(curr, prev)| (curr - prev).max(0.0))
    .sum::<f32>();

// Detect onset when flux exceeds threshold
let is_beat = flux > (avg_flux * threshold);
```

**Visualization Integration:**
- Braille character system (`src/visualization/braille.rs`) provides high-resolution rendering
- BrailleGrid offers 2×4 dot pattern per cell (4× vertical resolution)
- Default visualization uses Braille for smooth curves and accurate waveform display
- Beat flash effects work with both energy and flux-based detections

---

## Definition of Done

- [ ] SpectralFluxDetector struct implemented
- [ ] Flux calculation algorithm tested and validated
- [ ] Hybrid detection mode (energy + flux) integrated
- [ ] AudioParameters includes beat_flux field
- [ ] Existing energy-based detection unchanged (regression tested)
- [ ] Unit tests pass (flux calculation, onset detection)
- [ ] Integration tests pass (hybrid mode, performance)
- [ ] Manual testing with piano/guitar music shows improved detection
- [ ] Documentation updated with algorithm explanation
- [ ] Performance validated (<1ms overhead)
- [ ] Braille visualization system confirmed as default

---

## Risk and Compatibility Check

**Minimal Risk Assessment:**

**Primary Risk:** Spectral flux may trigger on spectral noise or non-musical content
**Mitigation:** Use dynamic threshold based on flux history (similar to energy approach)
**Rollback:** Disable flux detection, use energy-only mode (existing implementation)

**Compatibility Verification:**

- [x] No breaking changes to AudioParameters (additive field only)
- [x] Existing energy-based detection API unchanged
- [x] Visualizers continue to work with `beat` field (hybrid)
- [x] Performance impact negligible (uses existing spectrum)
- [x] Braille rendering system remains default

---

## Implementation Phases

### Phase 1: SpectralFluxDetector Structure

**File**: `src/dsp/mod.rs`

Create spectral flux detector parallel to energy-based detector:

```rust
/// Spectral flux beat detector for harmonic onset detection
///
/// Detects beat onsets by tracking changes in frequency spectrum.
/// More sensitive to harmonic instruments (piano, guitar) than energy-based detection.
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
    fn calculate_flux(&self, spectrum: &[f32]) -> f32 {
        spectrum
            .iter()
            .zip(self.prev_spectrum.iter())
            .map(|(curr, prev)| (curr - prev).max(0.0))
            .sum()
    }

    /// Detect beat based on spectral flux
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

        // 5. Calculate average flux
        let avg_flux = self.flux_history[..self.flux_history.len() - 1]
            .iter()
            .sum::<f32>()
            / (self.flux_history.len() - 1) as f32;

        // 6. Detect onset
        let threshold = avg_flux * (1.5 / self.sensitivity);
        let is_beat = flux > threshold && flux > 0.01;

        // 7. Update state
        if is_beat {
            self.last_beat_time = Some(Instant::now());
        }
        self.prev_spectrum.copy_from_slice(spectrum);

        is_beat
    }
}
```

---

### Phase 2: Integrate with DspProcessor

**File**: `src/dsp/mod.rs`

Add spectral flux detector to DspProcessor:

```rust
pub struct DspProcessor {
    fft_planner: FftPlanner<f32>,
    window_size: usize,
    sample_rate: u32,
    hann_window: Vec<f32>,
    scratch_buffer: Vec<Complex<f32>>,
    beat_detector: BeatDetector,           // Existing energy-based
    flux_detector: SpectralFluxDetector,   // NEW: Spectral flux
}

impl DspProcessor {
    pub fn new(sample_rate: u32, window_size: usize) -> Result<Self, DspError> {
        // ... existing validation and setup ...

        let beat_detector = BeatDetector::new(1.0, 0.1);
        let flux_detector = SpectralFluxDetector::new(1.0, 0.1, window_size / 2); // NEW

        Ok(Self {
            // ... existing fields ...
            beat_detector,
            flux_detector,
        })
    }

    pub fn process(&mut self, buffer: &AudioBuffer) -> AudioParameters {
        // 1. Get FFT spectrum (existing)
        let spectrum = self.process_buffer(buffer);

        // 2. Extract frequency bands (existing)
        let bass = self.extract_band(&spectrum, 20.0, 250.0);
        let mid = self.extract_band(&spectrum, 250.0, 4000.0);
        let treble = self.extract_band(&spectrum, 4000.0, 20000.0);

        // 3. Calculate overall amplitude (existing)
        let amplitude = self.calculate_rms(&buffer.samples);

        // 4. Detect beat - energy-based (existing)
        let beat_energy = self.beat_detector.detect(amplitude);

        // 5. Detect beat - spectral flux (NEW)
        let beat_flux = self.flux_detector.detect(&spectrum);

        // 6. Hybrid mode: combine both detectors
        let beat = beat_energy || beat_flux;

        // 7. Extract waveform (existing)
        let waveform = self.downsample_for_waveform(buffer, 512);

        AudioParameters {
            bass,
            mid,
            treble,
            amplitude,
            beat,        // Hybrid (energy OR flux)
            beat_flux,   // NEW: Flux-only (for debugging/visualization)
            spectrum,
            waveform,
        }
    }
}
```

---

### Phase 3: Extend AudioParameters

**File**: `src/dsp/mod.rs`

Add `beat_flux` field for visualization debugging:

```rust
#[derive(Debug, Clone, Default)]
pub struct AudioParameters {
    pub bass: f32,
    pub mid: f32,
    pub treble: f32,
    pub amplitude: f32,
    pub beat: bool,           // Hybrid: energy OR flux
    pub beat_flux: bool,      // NEW: Flux-only detection
    pub spectrum: Vec<f32>,
    pub waveform: Vec<f32>,
}
```

---

### Phase 4: Testing

**Unit Tests** (`src/dsp/mod.rs`):

```rust
#[test]
fn test_spectral_flux_calculation() {
    let detector = SpectralFluxDetector::new(1.0, 0.1, 1024);

    // Spectrum with no change -> flux = 0
    let spectrum = vec![0.5; 1024];
    detector.prev_spectrum.copy_from_slice(&spectrum);
    let flux = detector.calculate_flux(&spectrum);
    assert_eq!(flux, 0.0);

    // Spectrum with increase -> positive flux
    let new_spectrum = vec![0.7; 1024];
    let flux = detector.calculate_flux(&new_spectrum);
    assert!(flux > 0.0);

    // Spectrum with decrease -> flux = 0 (only positive differences)
    let lower_spectrum = vec![0.3; 1024];
    let flux = detector.calculate_flux(&lower_spectrum);
    assert!(flux < 1.0); // Should be near zero
}

#[test]
fn test_spectral_flux_detects_harmonic_onset() {
    let mut detector = SpectralFluxDetector::new(1.0, 0.1, 1024);

    // Build history with stable spectrum
    let stable = vec![0.1; 1024];
    for _ in 0..5 {
        detector.detect(&stable);
    }

    // Sudden spectral change (harmonic onset)
    let onset = vec![0.5; 1024];
    let is_beat = detector.detect(&onset);
    assert!(is_beat, "Should detect harmonic onset");
}

#[test]
fn test_hybrid_detection_combines_energy_and_flux() {
    let mut processor = DspProcessor::new(44100, 2048).unwrap();

    // Test energy-only detection (low freq sine)
    let bass_buffer = generate_sine_wave(100.0, 1.0, 44100, 2048);
    // ... build history and test ...

    // Test flux-only detection (harmonic content)
    // ... test with complex harmonic signal ...

    // Verify hybrid mode triggers on either
}
```

**Integration Tests** (`tests/spectral_flux_integration_test.rs`):

```rust
#[test]
fn test_spectral_flux_with_piano_notes() {
    // Test that spectral flux detects piano onsets
    // where energy-based detection might miss them
}

#[test]
fn test_braille_visualization_with_beat_detection() {
    // Test that braille grid renders beat-responsive visualizations
    // using the enhanced beat detection
}
```

---

## Manual Testing Procedure

```bash
# Build and run with loopback audio
cargo run --release -- --loopback

# Test Cases:
# 1. Play piano music - verify beats sync with note onsets
# 2. Play acoustic guitar - verify beats sync with strums
# 3. Play electronic music - verify both detectors work
# 4. Compare to energy-only mode (disable flux in code)
# 5. Verify braille visualizations respond to beats
```

---

## Performance Validation

**Expected Performance**:
- Spectral flux calculation: O(n) where n = spectrum length (1024)
- ~1024 floating point operations (subtraction + max)
- <0.5ms overhead on modern hardware
- Total beat detection (energy + flux): <1ms

**Benchmark**:
```rust
#[bench]
fn bench_spectral_flux_detection(b: &mut Bencher) {
    let mut detector = SpectralFluxDetector::new(1.0, 0.1, 1024);
    let spectrum = vec![0.5; 1024];
    b.iter(|| {
        detector.detect(black_box(&spectrum))
    });
}
// Expected: <0.5ms per call
```

---

## Dependencies

- **Depends on**:
  - DSP-001 (FFT processor)
  - DSP-004 (Energy-based beat detection - pattern reference)
  - Existing DspProcessor pipeline
  - AudioParameters structure
- **Blocks**: None (this is an enhancement)
- **Enables**:
  - More accurate beat detection for harmonic music
  - Improved visual sync for classical, jazz, acoustic genres
  - Foundation for advanced beat tracking (tempo estimation)

---

## Architecture References

- **DSP Component**: docs/architecture/README.md - DSP Processing section
- **Source Tree**: docs/architecture/source-tree.md - dsp/ module
- **Coding Standards**: docs/architecture/coding-standards.md - Rust style guide
- **Tech Stack**: docs/architecture/tech-stack.md - Real-time audio processing
- **Braille System**: src/visualization/braille.rs - High-resolution visualization

---

## Algorithm Theory

### Why Spectral Flux?

**Spectral Flux** measures the rate of change in the frequency spectrum over time. It's particularly effective for detecting onsets in harmonic instruments.

**Comparison of Approaches**:

| Approach | Best For | Limitation | Computational Cost |
|----------|----------|------------|-------------------|
| **Energy-based** (DSP-004) | Drums, bass, percussion | Misses harmonic onsets | Very low (RMS only) |
| **Spectral flux** (DSP-005) | Piano, guitar, strings | Can trigger on noise | Low (uses existing FFT) |
| **Hybrid** (DSP-005) | All music types | None (combines both) | Low (sum of both) |

**Spectral Flux Formula**:
```
flux = Σ max(0, spectrum[i] - prev_spectrum[i])
      i=0 to N
```

**Why only positive differences?**
- Onset = energy **increase** in spectrum
- Negative differences indicate energy **decrease** (decay)
- We only care about new sounds starting, not existing sounds fading

**Dynamic Threshold**:
```
threshold = avg_flux * (1.5 / sensitivity)
```
- Adapts to music dynamics (like energy-based)
- Factor of 1.5 tuned empirically
- Prevents false positives in quiet sections

---

## Notes for AI Agent

**Implementation is straightforward - this is a well-understood algorithm.**

### Key Points

1. **Reuse existing FFT**: Spectral flux uses the spectrum already calculated in process_buffer()
2. **Follow BeatDetector pattern**: Same structure, same cooldown, same sensitivity
3. **Hybrid is default**: `beat = energy_beat || flux_beat` catches all onsets
4. **Braille is default**: All visualizations use high-resolution Braille rendering system

### Common Pitfalls

- Don't forget to update `prev_spectrum` after each detect() call
- Only sum **positive** differences (flux = sum of increases only)
- Cooldown prevents both detectors from double-triggering
- Flux threshold tuning may need adjustment (1.5 is starting point)

### Success Indicators

1. Piano note onsets trigger beats (even without percussion)
2. Acoustic guitar strums trigger beats
3. Energy-based detection still works for drums
4. Hybrid mode catches more onsets than energy-only
5. No false positives in quiet passages
6. Performance stays under 1ms per frame
7. Braille visualizations render beat effects smoothly

### Time Estimate

**1-2 days total**:
- **4 hours**: Implement SpectralFluxDetector struct
- **2 hours**: Integrate with DspProcessor
- **2 hours**: Unit tests and validation
- **2 hours**: Manual testing with real music
- **2 hours**: Documentation and tuning

### Validation Checklist

- [ ] Spectral flux calculation is correct (sum of positive differences)
- [ ] Dynamic threshold adapts to music dynamics
- [ ] Cooldown prevents rapid re-triggering
- [ ] Hybrid mode (energy OR flux) works correctly
- [ ] Piano/guitar onsets detected that energy-based misses
- [ ] Existing energy-based detection still works
- [ ] Performance under 1ms per frame
- [ ] Braille visualizations respond to enhanced beat detection

This enhancement brings professional-grade beat detection to CrabMusic!
