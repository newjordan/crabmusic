# [DSP-006] Tempo Detection and BPM Estimation - Brownfield Enhancement

**Epic**: DSP Processing
**Priority**: P2 (Enhancement to Existing Beat Detection)
**Estimated Effort**: 1.5-2 days
**Status**: Draft

---

## Description

Add **tempo detection and BPM (Beats Per Minute) estimation** to analyze beat intervals and provide musical timing context to visualizers. This enables tempo-synced visual effects, automatic parameter adjustment, and enhanced user experience with tempo-aware visualizations.

**Current System**: Beat detection (DSP-004, DSP-005) identifies beat onsets but doesn't track tempo or timing patterns. Visualizers receive discrete beat events but lack musical context (tempo, meter, phase).

**The Enhancement**: Add tempo analysis that:
- Tracks inter-onset intervals (IOI) between detected beats
- Estimates BPM using median filtering for stability
- Provides confidence metric for tempo reliability
- Exposes BPM to visualizers through AudioParameters
- Optionally auto-adjusts beat detection cooldown based on tempo
- Handles tempo changes gracefully (accelerando/ritardando)

**Key Benefit**: Visualizers can create tempo-synced effects (pulsing at exact BPM, animations that match musical timing, beat phase indicators) and beat detection becomes more adaptive to music tempo.

---

## User Story

**As a** CrabMusic user,
**I want** visualizations that sync with the tempo of the music,
**So that** visual effects pulse at the exact BPM, animations match musical timing, and the experience feels musically coherent rather than just reactive to individual beats.

---

## Story Context

**Existing System Integration:**
- Integrates with: DspProcessor in `src/dsp/mod.rs`
- Technology: Rust with std::time for high-precision timing
- Follows pattern: Beat detection structs (BeatDetector, SpectralFluxDetector)
- Touch points: DspProcessor::process(), AudioParameters, beat detectors

**Existing Beat Detection**:
- DSP-004: Energy-based beat detection (tracks beat times)
- DSP-005: Spectral flux beat detection (hybrid mode)
- Both use std::time::Instant for beat timestamps
- Cooldown mechanism uses fixed 100ms (600 BPM max)

**Tempo Detection Approach**:
- Track time intervals between consecutive beats
- Calculate BPM: `60.0 / interval_seconds`
- Use median of recent intervals for stability
- Filter outliers (too fast/slow to be musical tempo)
- Typical music range: 60-180 BPM (1000ms-333ms intervals)

---

## Acceptance Criteria

**Functional Requirements:**

1. TempoDetector struct created with beat interval tracking
2. Inter-onset interval (IOI) calculation from beat timestamps
3. BPM estimation using median filtering (last 8 beats)
4. Outlier filtering removes non-musical intervals (>2s or <0.25s)
5. Confidence metric indicates tempo stability (0.0-1.0)
6. Handles tempo changes with windowed analysis
7. AudioParameters includes `bpm` field (f32) and `tempo_confidence` (f32)
8. Default tempo (120 BPM) when insufficient data
9. Configurable smoothing factor for tempo stability

**Integration Requirements:**

10. Existing beat detection continues to work unchanged
11. TempoDetector receives beat events from hybrid detector
12. Integration with DspProcessor maintains current API
13. Braille visualization system remains default for rendering
14. No breaking changes to existing visualizers

**Quality Requirements:**

15. Unit tests validate BPM calculation accuracy
16. Integration tests with synthetic beat patterns
17. Manual testing with real music (various tempos)
18. Performance impact <0.1ms per frame
19. Documentation includes algorithm explanation and usage examples

**Optional Enhancement (Stretch Goal):**

20. Auto-adjust beat detection cooldown based on detected BPM

---

## Technical Notes

**Integration Approach:**
- Create TempoDetector struct that receives beat events
- Track timestamps of recent beats (sliding window)
- Calculate IOI between consecutive beats
- Estimate BPM using median of recent IOIs
- Expose BPM and confidence through AudioParameters

**Existing Pattern Reference:**
- Follow BeatDetector structure (src/dsp/mod.rs)
- Use std::time::Instant for high-precision timing
- Similar history management (Vec with fixed size)
- Same integration pattern in DspProcessor::process()

**Key Constraints:**
- Must handle tempo changes gracefully
- Outlier filtering prevents false BPM from noise
- Confidence metric helps visualizers decide whether to use BPM
- Performance must remain negligible (<0.1ms)
- Braille rendering system is default for all visualizations

**Algorithm Overview:**
```rust
// Calculate inter-onset interval (IOI)
let interval = current_beat_time - last_beat_time;
beat_intervals.push(interval);

// Calculate BPM from median interval
let median_interval = median(beat_intervals);
let bpm = 60.0 / median_interval.as_secs_f32();

// Calculate confidence from interval variance
let variance = calculate_variance(beat_intervals);
let confidence = 1.0 / (1.0 + variance);
```

**Visualization Integration:**
- Visualizers can access `params.bpm` for tempo-synced effects
- Confidence metric (`params.tempo_confidence`) indicates reliability
- Braille system provides high-resolution rendering for tempo indicators
- Example: Pulse effect at exact BPM, progress bars synced to beats

---

## Definition of Done

- [ ] TempoDetector struct implemented
- [ ] IOI calculation and BPM estimation working
- [ ] Median filtering provides stable BPM
- [ ] Outlier filtering removes non-musical intervals
- [ ] Confidence metric implemented
- [ ] AudioParameters includes `bpm` and `tempo_confidence` fields
- [ ] Integration with DspProcessor complete
- [ ] Existing beat detection unchanged (regression tested)
- [ ] Unit tests pass (BPM calculation, outlier filtering)
- [ ] Integration tests pass (synthetic patterns)
- [ ] Manual testing with real music validates accuracy
- [ ] Documentation updated with algorithm and usage
- [ ] Performance validated (<0.1ms overhead)
- [ ] Braille visualization system confirmed as default

---

## Risk and Compatibility Check

**Minimal Risk Assessment:**

**Primary Risk:** Tempo estimation may be unstable during tempo changes or complex rhythms
**Mitigation:** Use median filtering and confidence metric; visualizers can ignore low-confidence BPM
**Rollback:** Remove BPM field from AudioParameters, tempo detection disabled

**Compatibility Verification:**

- [x] No breaking changes to AudioParameters (additive fields only)
- [x] Existing beat detection API unchanged
- [x] Visualizers continue to work without using BPM
- [x] Performance impact negligible (<0.1ms per frame)
- [x] Braille rendering system remains default

---

## Implementation Phases

### Phase 1: TempoDetector Structure

**File**: `src/dsp/mod.rs`

Create tempo detector that tracks beat intervals:

```rust
/// Tempo detector for BPM estimation
///
/// Analyzes inter-onset intervals (IOI) between beats to estimate tempo.
/// Uses median filtering for stability and provides confidence metric.
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

        // Need at least 3 beats to estimate tempo
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

        // Use median interval for stability
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
        self.confidence = 1.0 / (1.0 + variance * 100.0).min(1.0);
    }

    /// Get current BPM estimate
    fn bpm(&self) -> f32 {
        self.current_bpm
    }

    /// Get confidence in tempo estimate (0.0-1.0)
    fn confidence(&self) -> f32 {
        self.confidence
    }
}
```

---

### Phase 2: Integrate with DspProcessor

**File**: `src/dsp/mod.rs`

Add tempo detector to DspProcessor:

```rust
pub struct DspProcessor {
    fft_planner: FftPlanner<f32>,
    window_size: usize,
    sample_rate: u32,
    hann_window: Vec<f32>,
    scratch_buffer: Vec<Complex<f32>>,
    beat_detector: BeatDetector,
    flux_detector: SpectralFluxDetector,
    tempo_detector: TempoDetector,  // NEW: Tempo analysis
}

impl DspProcessor {
    pub fn new(sample_rate: u32, window_size: usize) -> Result<Self, DspError> {
        // ... existing validation and setup ...

        let beat_detector = BeatDetector::new(1.0, 0.1);
        let flux_detector = SpectralFluxDetector::new(1.0, 0.1, window_size / 2);
        let tempo_detector = TempoDetector::new(60.0, 180.0); // 60-180 BPM range

        Ok(Self {
            // ... existing fields ...
            beat_detector,
            flux_detector,
            tempo_detector,
        })
    }

    pub fn process(&mut self, buffer: &AudioBuffer) -> AudioParameters {
        // 1. Get FFT spectrum
        let spectrum = self.process_buffer(buffer);

        // 2. Extract frequency bands
        let bass = self.extract_band(&spectrum, 20.0, 250.0);
        let mid = self.extract_band(&spectrum, 250.0, 4000.0);
        let treble = self.extract_band(&spectrum, 4000.0, 20000.0);

        // 3. Calculate overall amplitude
        let amplitude = self.calculate_rms(&buffer.samples);

        // 4. Detect beat - energy-based
        let beat_energy = self.beat_detector.detect(amplitude);

        // 5. Detect beat - spectral flux
        let beat_flux = self.flux_detector.detect(&spectrum);

        // 6. Hybrid mode: combine both detectors
        let beat = beat_energy || beat_flux;

        // 7. Update tempo detector if beat occurred (NEW)
        if beat {
            self.tempo_detector.register_beat(Instant::now());
        }

        // 8. Get tempo estimate (NEW)
        let bpm = self.tempo_detector.bpm();
        let tempo_confidence = self.tempo_detector.confidence();

        // 9. Extract waveform
        let waveform = self.downsample_for_waveform(buffer, 512);

        AudioParameters {
            bass,
            mid,
            treble,
            amplitude,
            beat,
            beat_flux,
            bpm,              // NEW: Tempo in BPM
            tempo_confidence, // NEW: Confidence metric
            spectrum,
            waveform,
        }
    }
}
```

---

### Phase 3: Extend AudioParameters

**File**: `src/dsp/mod.rs`

Add tempo fields to AudioParameters:

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

    /// Beat detected (hybrid: energy OR flux)
    pub beat: bool,

    /// Beat detected by spectral flux only
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
    pub spectrum: Vec<f32>,

    /// Waveform samples for oscilloscope visualization
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
```

---

### Phase 4: Testing

**Unit Tests** (`src/dsp/mod.rs`):

```rust
#[test]
fn test_tempo_detector_creation() {
    let detector = TempoDetector::new(60.0, 180.0);
    assert_eq!(detector.current_bpm, 120.0); // Default tempo
    assert_eq!(detector.confidence, 0.0);     // No confidence yet
    assert_eq!(detector.beat_times.len(), 0);
}

#[test]
fn test_tempo_detector_requires_minimum_beats() {
    let mut detector = TempoDetector::new(60.0, 180.0);

    // Register 2 beats - insufficient for tempo
    let now = Instant::now();
    detector.register_beat(now);
    detector.register_beat(now + Duration::from_millis(500));

    assert_eq!(detector.confidence(), 0.0);
}

#[test]
fn test_tempo_detector_estimates_120_bpm() {
    let mut detector = TempoDetector::new(60.0, 180.0);

    // 120 BPM = 500ms per beat
    let now = Instant::now();
    for i in 0..5 {
        detector.register_beat(now + Duration::from_millis(i * 500));
    }

    let bpm = detector.bpm();
    assert!(
        (bpm - 120.0).abs() < 5.0,
        "Expected ~120 BPM, got {}",
        bpm
    );
    assert!(detector.confidence() > 0.5, "Should have reasonable confidence");
}

#[test]
fn test_tempo_detector_filters_outliers() {
    let mut detector = TempoDetector::new(60.0, 180.0);

    // Mix of valid beats and outliers
    let now = Instant::now();
    detector.register_beat(now);
    detector.register_beat(now + Duration::from_millis(500)); // 120 BPM
    detector.register_beat(now + Duration::from_millis(1000)); // 120 BPM
    detector.register_beat(now + Duration::from_millis(5000)); // OUTLIER (too slow)
    detector.register_beat(now + Duration::from_millis(5500)); // 120 BPM

    // Should ignore outlier and estimate ~120 BPM
    let bpm = detector.bpm();
    assert!(
        (bpm - 120.0).abs() < 10.0,
        "Should filter outlier, got {} BPM",
        bpm
    );
}

#[test]
fn test_tempo_detector_handles_tempo_change() {
    let mut detector = TempoDetector::new(60.0, 180.0);

    let now = Instant::now();

    // Start at 120 BPM (500ms intervals)
    for i in 0..4 {
        detector.register_beat(now + Duration::from_millis(i * 500));
    }
    let bpm1 = detector.bpm();
    assert!((bpm1 - 120.0).abs() < 5.0);

    // Change to 140 BPM (428ms intervals)
    for i in 4..8 {
        let offset = 2000 + (i - 4) * 428;
        detector.register_beat(now + Duration::from_millis(offset));
    }
    let bpm2 = detector.bpm();
    assert!(
        (bpm2 - 140.0).abs() < 10.0,
        "Should adapt to new tempo, got {}",
        bpm2
    );
}

#[test]
fn test_tempo_detector_confidence_decreases_with_variance() {
    let mut detector = TempoDetector::new(60.0, 180.0);

    let now = Instant::now();

    // Stable tempo - high confidence
    for i in 0..5 {
        detector.register_beat(now + Duration::from_millis(i * 500));
    }
    let stable_confidence = detector.confidence();

    // Reset and test unstable tempo - low confidence
    let mut detector2 = TempoDetector::new(60.0, 180.0);
    let intervals = [500, 450, 550, 480, 520]; // Variable timing
    let mut offset = 0;
    for interval in intervals {
        detector2.register_beat(now + Duration::from_millis(offset));
        offset += interval;
    }
    let unstable_confidence = detector2.confidence();

    assert!(
        stable_confidence > unstable_confidence,
        "Stable tempo should have higher confidence"
    );
}

#[test]
fn test_tempo_integration_with_beat_detection() {
    let mut processor = DspProcessor::new(44100, 2048).unwrap();

    // Simulate 120 BPM beat pattern (500ms per beat)
    // At 60 FPS, that's ~30 frames per beat

    let quiet = AudioBuffer::with_samples(vec![0.1; 2048], 44100, 1);
    let loud = AudioBuffer::with_samples(vec![0.5; 2048], 44100, 1);

    // Build up beat history
    for _ in 0..3 {
        // Quiet frames
        for _ in 0..29 {
            processor.process(&quiet);
        }
        // Beat frame
        let params = processor.process(&loud);
        assert!(params.beat, "Should detect beat");
    }

    // After 3 beats, should have tempo estimate
    let params = processor.process(&quiet);
    assert!(params.tempo_confidence > 0.0, "Should have some confidence");

    // BPM should be within reasonable range
    assert!(
        params.bpm >= 60.0 && params.bpm <= 180.0,
        "BPM {} should be in musical range",
        params.bpm
    );
}
```

**Integration Tests** (`tests/tempo_detection_integration_test.rs`):

```rust
use crabmusic::dsp::DspProcessor;
use crabmusic::audio::AudioBuffer;
use std::time::Duration;
use std::thread;

#[test]
fn test_tempo_detection_with_synthetic_beat_pattern() {
    let mut processor = DspProcessor::new(44100, 2048).unwrap();

    // Generate beat pattern at 120 BPM (500ms per beat)
    let quiet = AudioBuffer::with_samples(vec![0.1; 2048], 44100, 1);
    let loud = AudioBuffer::with_samples(vec![0.8; 2048], 44100, 1);

    // Process beats with 500ms spacing
    for _ in 0..5 {
        let params = processor.process(&loud);
        assert!(params.beat);

        // Wait 500ms
        thread::sleep(Duration::from_millis(500));

        // Process quiet frames in between
        for _ in 0..5 {
            processor.process(&quiet);
        }
    }

    // Check tempo estimate
    let params = processor.process(&quiet);
    assert!(params.tempo_confidence > 0.5, "Should have confidence");
    assert!(
        (params.bpm - 120.0).abs() < 10.0,
        "Expected ~120 BPM, got {}",
        params.bpm
    );
}

#[test]
fn test_braille_visualization_with_tempo() {
    // Test that visualizations can use tempo info
    // for tempo-synced effects with braille rendering
    let mut processor = DspProcessor::new(44100, 2048).unwrap();

    // ... generate beat pattern ...

    let params = processor.process(&quiet);
    if params.tempo_confidence > 0.7 {
        // Calculate pulse period from BPM
        let pulse_period_seconds = 60.0 / params.bpm;
        assert!(pulse_period_seconds > 0.0);
    }
}
```

---

## Manual Testing Procedure

```bash
# Build and run with loopback audio
cargo run --release -- --loopback

# Test Cases:
# 1. Play music with clear, stable tempo (electronic, pop)
#    - Verify BPM matches perceived tempo
#    - Check confidence is high (>0.7)
#
# 2. Play music with variable tempo (classical, rubato)
#    - Verify confidence is low when tempo changes
#    - Ensure BPM adapts to new tempo
#
# 3. Play very fast music (160+ BPM)
#    - Verify BPM stays in range (60-180)
#    - Check beat detection doesn't miss beats
#
# 4. Play very slow music (60-70 BPM)
#    - Verify BPM is detected correctly
#    - Ensure visualizations sync properly
#
# 5. Test braille visualizations respond to tempo
#    - Visual effects should pulse at detected BPM
#    - Low confidence should disable tempo sync

# Log BPM to console for verification
# Add debug output in main.rs:
println!("BPM: {:.1} (confidence: {:.2})", params.bpm, params.tempo_confidence);
```

---

## Performance Validation

**Expected Performance**:
- Tempo calculation: O(n log n) where n = history size (8 beats)
- Sorting for median: ~8 comparisons
- Variance calculation: 8 floating point operations
- Total overhead: <0.1ms per frame
- Only runs when beat detected (not every frame)

**Benchmark**:
```rust
#[bench]
fn bench_tempo_detection(b: &mut Bencher) {
    let mut detector = TempoDetector::new(60.0, 180.0);
    let now = Instant::now();
    b.iter(|| {
        detector.register_beat(black_box(now))
    });
}
// Expected: <100μs per beat registration
```

---

## Dependencies

- **Depends on**:
  - DSP-004 (Energy-based beat detection)
  - DSP-005 (Spectral flux beat detection - hybrid mode)
  - Existing DspProcessor pipeline
  - std::time::Instant for timing
- **Blocks**: None (this is an enhancement)
- **Enables**:
  - Tempo-synced visualizations
  - Auto-adaptive beat detection parameters (future)
  - Musical context for advanced effects
  - Beat phase tracking (strong/weak beats - future)

---

## Architecture References

- **DSP Component**: docs/architecture/README.md - DSP Processing section
- **Source Tree**: docs/architecture/source-tree.md - dsp/ module
- **Coding Standards**: docs/architecture/coding-standards.md - Rust style guide
- **Tech Stack**: docs/architecture/tech-stack.md - Real-time audio processing
- **Braille System**: src/visualization/braille.rs - High-resolution visualization

---

## Algorithm Theory

### Inter-Onset Interval (IOI) Analysis

**Inter-Onset Interval (IOI)** is the time between consecutive beat onsets. Tempo is the reciprocal of the average IOI:

```
BPM = 60.0 / IOI_seconds
```

**Example**:
- 120 BPM → IOI = 0.5 seconds (500ms)
- 140 BPM → IOI = 0.428 seconds (428ms)
- 90 BPM → IOI = 0.667 seconds (667ms)

### Why Median Filtering?

**Median** is more robust to outliers than **mean**:

| Intervals (ms) | Mean | Median | Actual Tempo |
|----------------|------|--------|--------------|
| 500, 500, 500, 500, 500 | 500 | 500 | 120 BPM ✓ |
| 500, 500, 2000, 500, 500 | 800 | 500 | 120 BPM ✓ |
| 500, 450, 550, 480, 520 | 500 | 500 | 120 BPM ✓ |

**Why it matters**:
- Missed beats create long intervals (outliers)
- False positives create short intervals (outliers)
- Median ignores these, mean gets skewed

### Confidence Metric

Confidence measures tempo **stability** based on variance:

```rust
confidence = 1.0 / (1.0 + variance * scale_factor)
```

**Interpretation**:
- Low variance → High confidence (stable tempo)
- High variance → Low confidence (unstable/rubato)
- Scale factor (100.0) tuned empirically

**Use in Visualizers**:
```rust
if params.tempo_confidence > 0.7 {
    // High confidence - safe to sync with tempo
    pulse_visualizer.sync_to_bpm(params.bpm);
} else {
    // Low confidence - use default timing or disable sync
    pulse_visualizer.use_default_timing();
}
```

### Outlier Filtering

Musical tempo range: **60-180 BPM**
- Below 60 BPM: Either very slow music OR missed beats
- Above 180 BPM: Either very fast music OR double-triggering

**Filter Strategy**:
```rust
let bpm = 60.0 / interval;
if bpm >= min_bpm && bpm <= max_bpm {
    // Accept as valid musical tempo
    intervals.push(interval);
}
```

**Edge Cases**:
- Extremely slow music (40 BPM): Extend min_bpm if needed
- Extremely fast music (200+ BPM): Extend max_bpm if needed
- Default range works for 95% of music

### Handling Tempo Changes

**Windowed Analysis**: Only track recent 8 beats
- Old beats expire from history
- New tempo emerges within ~8 beats
- Confidence drops during transition, then recovers

**Accelerando/Ritardando**:
- Confidence naturally decreases (high variance)
- Median adapts gradually to new tempo
- Visualizers can respond to low confidence by disabling sync

---

## Optional Enhancement (Future)

### Auto-Adjust Beat Detection Cooldown

Once tempo is known, optimize cooldown:

```rust
// Adjust cooldown based on detected BPM
let optimal_cooldown = 60.0 / (bpm * 2.0); // Half a beat

// Prevent double-triggering while allowing fast beats
self.beat_detector.set_cooldown(optimal_cooldown);
```

**Benefits**:
- Fast music (180 BPM): Shorter cooldown (167ms)
- Slow music (60 BPM): Longer cooldown (500ms)
- More adaptive to music style

**Complexity**: Requires mutable cooldown in BeatDetector (refactor)

---

## Notes for AI Agent

**Tempo detection is a standard MIR (Music Information Retrieval) technique.**

### Key Points

1. **Use median, not mean**: Robust to outliers (missed beats, false positives)
2. **Filter outliers**: Only accept musical tempo range (60-180 BPM)
3. **Confidence metric**: Essential for visualizers to decide whether to use BPM
4. **Windowed history**: Allows adaptation to tempo changes
5. **Braille is default**: All visualizations use high-resolution rendering

### Common Pitfalls

- Don't forget to check confidence before using BPM in visualizers
- Median calculation requires sorting (use sort_by for f32)
- Need at least 3 beats for tempo estimate (2 intervals)
- Tempo changes cause confidence to drop temporarily (expected)
- Very fast/slow music may fall outside default range (adjustable)

### Success Indicators

1. Steady 120 BPM music reports BPM ~120 with high confidence
2. Variable tempo music reports low confidence
3. Tempo changes are detected within 8 beats
4. Outliers (missed beats) don't disrupt BPM estimate
5. Performance overhead negligible (<0.1ms)
6. Visualizers can create tempo-synced effects
7. Braille visualizations render tempo indicators smoothly

### Time Estimate

**1.5-2 days total**:
- **4 hours**: Implement TempoDetector struct
- **2 hours**: Integrate with DspProcessor
- **3 hours**: Unit tests and validation
- **2 hours**: Integration tests
- **2 hours**: Manual testing with real music
- **1 hour**: Documentation

### Validation Checklist

- [ ] BPM calculation accurate within ±5 BPM
- [ ] Median filtering removes outliers correctly
- [ ] Confidence metric reflects tempo stability
- [ ] Tempo changes detected and adapted to
- [ ] Performance under 0.1ms per frame
- [ ] Visualizers can use BPM for tempo-synced effects
- [ ] Low confidence prevents unreliable sync
- [ ] Braille visualization system confirmed as default

This enhancement enables musical tempo awareness for intelligent visualizations!
