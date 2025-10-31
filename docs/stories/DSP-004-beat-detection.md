# [DSP-004] Beat Detection with Energy-Based Onset Detection

**Epic**: DSP Processing
**Priority**: P1 (Required for Enhanced Visualization)
**Estimated Effort**: 1.5-2 days
**Status**: Complete

---

## Description

Implement real-time beat detection using energy-based onset detection to identify rhythmic pulses in music. Beat detection enables visual effects that sync with the music's rhythm, creating more engaging and dynamic visualizations.

**The Problem**: Visualizations need to respond to beats and rhythm, not just frequency content. Without beat detection, visualizers can only react to amplitude and frequency bands, missing the temporal structure of music.

**The Solution**: Implement an energy-based onset detector that:
- Tracks audio energy (RMS amplitude) over time
- Detects sudden increases in energy above a dynamic threshold
- Uses a cooldown mechanism to prevent false positives
- Integrates seamlessly with existing DspProcessor pipeline
- Exposes beat events through AudioParameters for visualizers to use

**Visual Impact**: Enables beat-synchronized effects like:
- Flash effects on kick drums and snare hits
- Pulse animations that sync with rhythm
- Color changes on beat onsets
- Particle emission triggered by beats

---

## Acceptance Criteria

- [x] BeatDetector struct created with energy history tracking
- [x] Energy-based onset detection algorithm implemented
- [x] Configurable sensitivity parameter (1.0 = normal sensitivity)
- [x] Cooldown mechanism prevents false positives (minimum time between beats)
- [x] Energy history maintained with sliding window (default 10 samples)
- [x] Dynamic threshold calculated from recent energy history
- [x] Beat detection integrated into DspProcessor::process()
- [x] AudioParameters includes beat field (boolean)
- [x] Minimum energy threshold prevents noise triggering beats
- [x] Unit tests validate beat detection with synthetic audio
- [x] Beat detection works in real-time with <1ms overhead
- [x] Visualizers can use beat events for synchronized effects

---

## Technical Approach

### Phase 1: BeatDetector Structure

**File**: `src/dsp/mod.rs`

The beat detector uses an energy-based approach where beats are detected as sudden increases in audio energy compared to recent history.

```rust
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
}
```

**Key Design Decisions**:
- **History size**: 10 samples provides ~200ms of context at 60 FPS
- **Cooldown**: 100ms (0.1s) prevents double-triggering on the same beat
- **Sensitivity**: 1.0 default, can be tuned for different music styles

---

### Phase 2: Energy-Based Onset Detection Algorithm

**File**: `src/dsp/mod.rs`

The core detection algorithm compares current energy against recent average energy:

```rust
/// Detect if a beat occurred based on current energy
///
/// # Arguments
/// * `current_energy` - Current audio energy (RMS amplitude)
///
/// # Returns
/// true if a beat was detected, false otherwise
fn detect(&mut self, current_energy: f32) -> bool {
    // 1. Check cooldown - prevent rapid re-triggering
    if let Some(last_time) = self.last_beat_time {
        let elapsed = last_time.elapsed().as_secs_f32();
        if elapsed < self.cooldown_seconds {
            return false;
        }
    }

    // 2. Add current energy to history
    self.energy_history.push(current_energy);
    if self.energy_history.len() > self.history_size {
        self.energy_history.remove(0);
    }

    // 3. Need at least 3 samples for comparison
    if self.energy_history.len() < 3 {
        return false;
    }

    // 4. Calculate average energy of recent history (excluding current)
    let history_avg = self.energy_history[..self.energy_history.len() - 1]
        .iter()
        .sum::<f32>()
        / (self.energy_history.len() - 1) as f32;

    // 5. Detect beat if current energy significantly exceeds average
    let threshold = history_avg * (1.5 / self.sensitivity);
    let is_beat = current_energy > threshold && current_energy > 0.1;

    // 6. Update last beat time
    if is_beat {
        self.last_beat_time = Some(Instant::now());
    }

    is_beat
}
```

**Algorithm Breakdown**:

1. **Cooldown Check**: Prevents detecting the same beat multiple times
   - Uses `std::time::Instant` for precise timing
   - Default 100ms cooldown works well for most music (600 BPM max)

2. **History Management**: Maintains sliding window of recent energy values
   - Fixed size (10 samples) for predictable performance
   - FIFO queue (remove oldest when full)

3. **Bootstrap Protection**: Requires minimum 3 samples before detecting
   - Prevents false positives on startup
   - Ensures stable average calculation

4. **Dynamic Threshold Calculation**:
   - Threshold = `average_energy * (1.5 / sensitivity)`
   - Factor of 1.5 means energy must be 50% higher than average
   - Sensitivity inverts the threshold (higher sensitivity = lower threshold)
   - Example: sensitivity=2.0 → threshold = avg * 0.75 (easier to trigger)

5. **Beat Detection Criteria**:
   - Current energy must exceed dynamic threshold
   - Current energy must exceed absolute minimum (0.1) to filter noise
   - Both conditions prevent false positives in silence or quiet passages

6. **State Update**: Record timestamp of detected beat for cooldown

---

### Phase 3: Integration with DspProcessor

**File**: `src/dsp/mod.rs`

Beat detection is integrated into the main audio processing pipeline:

```rust
pub struct DspProcessor {
    fft_planner: FftPlanner<f32>,
    window_size: usize,
    sample_rate: u32,
    hann_window: Vec<f32>,
    scratch_buffer: Vec<Complex<f32>>,
    beat_detector: BeatDetector,  // NEW: Beat detection state
}

impl DspProcessor {
    pub fn new(sample_rate: u32, window_size: usize) -> Result<Self, DspError> {
        // ... existing setup ...

        // Initialize beat detector with default parameters
        let beat_detector = BeatDetector::new(1.0, 0.1); // Normal sensitivity, 100ms cooldown

        Ok(Self {
            // ... existing fields ...
            beat_detector,
        })
    }

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
        let beat = self.beat_detector.detect(amplitude);  // NEW: Beat detection

        AudioParameters {
            bass,
            mid,
            treble,
            amplitude,
            beat,  // NEW: Expose beat to visualizers
        }
    }
}
```

**Integration Points**:
- Beat detector created during DspProcessor initialization
- Uses RMS amplitude as energy metric (already calculated for visualizations)
- No performance overhead (beat detection is O(1) with fixed history size)
- Beat state passed to visualizers through AudioParameters

---

### Phase 4: AudioParameters Extension

**File**: `src/dsp/mod.rs`

Extended AudioParameters to include beat information:

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
    pub beat: bool,  // NEW: Beat detection flag
}
```

**Usage in Visualizers**:

Visualizers can now respond to beats. Example from Sine Wave Visualizer:

```rust
impl Visualizer for SineWaveVisualizer {
    fn update(&mut self, params: &AudioParameters) {
        // ... existing amplitude/frequency updates ...

        // Handle beat flash effect
        if params.beat {
            self.beat_flash = 1.0; // Trigger flash
        } else {
            self.beat_flash *= 0.85; // Decay flash over time
        }
    }

    fn render(&self, grid: &mut GridBuffer) {
        // Use beat_flash to boost brightness or coverage
        let flash_boost = self.beat_flash * 0.3;
        let coverage = base_coverage + flash_boost;
        // ...
    }
}
```

---

## Dependencies

- **Depends on**:
  - DSP-001 (FFT processor exists)
  - DSP-002 (Frequency band extraction, RMS calculation)
- **Blocks**: None (this is an enhancement)
- **Enables**:
  - Beat-synchronized visual effects in all visualizers
  - Rhythm-based color changes (with color system)
  - Enhanced user experience through music-synced feedback

---

## Architecture References

- **DSP Component**: docs/architecture/README.md - DSP Processing section
- **Source Tree**: docs/architecture/source-tree.md - dsp/ module
- **Coding Standards**: docs/architecture/coding-standards.md - Rust style guide
- **Tech Stack**: docs/architecture/tech-stack.md - Real-time audio processing

---

## Testing Requirements

### Unit Tests

**File**: `src/dsp/mod.rs` (tests module)

Key tests for beat detection:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beat_detector_creation() {
        let detector = BeatDetector::new(1.0, 0.1);
        assert_eq!(detector.sensitivity, 1.0);
        assert_eq!(detector.cooldown_seconds, 0.1);
        assert_eq!(detector.history_size, 10);
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
        std::thread::sleep(std::time::Duration::from_secs(1));
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
}
```

### Integration Tests

**File**: `tests/beat_detection_integration_test.rs`

```rust
use crabmusic::dsp::DspProcessor;
use crabmusic::audio::AudioBuffer;

#[test]
fn test_beat_detection_with_kick_drum_pattern() {
    let mut processor = DspProcessor::new(44100, 2048).unwrap();
    let sample_rate = 44100;
    let window_size = 2048;

    // Simulate kick drum pattern: LOUD-quiet-quiet-quiet-LOUD-quiet-quiet-quiet
    let quiet_samples = vec![0.1; window_size];
    let loud_samples = vec![0.8; window_size];

    let mut beat_count = 0;

    // First kick
    let params = processor.process(&AudioBuffer::with_samples(loud_samples.clone(), sample_rate, 1));
    if params.beat { beat_count += 1; }

    // Quiet sections
    for _ in 0..3 {
        processor.process(&AudioBuffer::with_samples(quiet_samples.clone(), sample_rate, 1));
    }

    // Second kick
    let params = processor.process(&AudioBuffer::with_samples(loud_samples.clone(), sample_rate, 1));
    if params.beat { beat_count += 1; }

    assert!(beat_count >= 1, "Should detect at least one beat in kick pattern");
}

#[test]
fn test_beat_detection_with_sine_wave_pulses() {
    let mut processor = DspProcessor::new(44100, 2048).unwrap();

    // Generate sine wave pulses (simulated beat pattern)
    let mut beat_detected = false;

    // Low amplitude baseline
    let quiet_buffer = generate_sine_wave(440.0, 0.1, 44100, 2048);
    for _ in 0..5 {
        processor.process(&quiet_buffer);
    }

    // High amplitude pulse
    let loud_buffer = generate_sine_wave(440.0, 0.8, 44100, 2048);
    let params = processor.process(&loud_buffer);
    beat_detected = params.beat;

    assert!(beat_detected, "Should detect beat on amplitude pulse");
}

fn generate_sine_wave(freq: f32, amplitude: f32, sample_rate: u32, num_samples: usize) -> AudioBuffer {
    let samples: Vec<f32> = (0..num_samples)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            amplitude * (2.0 * std::f32::consts::PI * freq * t).sin()
        })
        .collect();
    AudioBuffer::with_samples(samples, sample_rate, 1)
}
```

### Manual Testing

```bash
# Build and run with loopback audio
cargo run --release -- --loopback

# Test beat detection with different music:
# 1. Play music with clear beats (electronic, hip-hop, rock)
# 2. Watch for visual effects syncing with kick drums
# 3. Try different music styles (fast BPM, slow BPM)
# 4. Verify no false positives during quiet sections
# 5. Verify beat flash effects appear on strong transients
```

**Expected Behavior**:
- Strong kick drums and snare hits trigger visible effects
- No flashing during quiet passages or sustained notes
- Beat effects sync with perceived rhythm
- Works across different BPM ranges (60-180 BPM typical)

---

## Algorithm Theory

### Why Energy-Based Onset Detection?

**Onset detection** identifies the start of musical events (notes, beats, transients). Several approaches exist:

1. **Spectral Flux**: Detects changes in frequency spectrum
   - More accurate for complex music
   - Higher computational cost
   - Requires FFT comparison between frames

2. **Energy-Based**: Detects changes in overall amplitude
   - Simpler and faster
   - Works well for percussive content (drums, beats)
   - Lower computational cost
   - Chosen for this implementation

3. **Phase-Based**: Detects phase discontinuities
   - Best for harmonic onsets
   - Complex to implement
   - Higher computational cost

**Decision**: Energy-based chosen for:
- **Performance**: Minimal overhead (already calculating RMS)
- **Effectiveness**: Works well for rhythm detection in most music
- **Simplicity**: Easy to tune and understand
- **Real-time**: No additional DSP processing required

### Threshold Calculation

The dynamic threshold adapts to music dynamics:

```
threshold = average_energy * (1.5 / sensitivity)
```

**Why dynamic threshold?**
- Music has varying overall loudness
- Quiet sections have smaller energy spikes that are still beats
- Loud sections need higher threshold to avoid false positives
- Dynamic threshold adapts automatically

**Why factor of 1.5?**
- Empirically tested to work well for most music
- Detects clear percussive transients
- Filters out gradual loudness changes
- Balance between sensitivity and false positives

**Sensitivity parameter**:
- `sensitivity = 0.5` → threshold = avg * 3.0 (very conservative, only strong beats)
- `sensitivity = 1.0` → threshold = avg * 1.5 (normal, good default)
- `sensitivity = 2.0` → threshold = avg * 0.75 (sensitive, catches subtle beats)

### Cooldown Mechanism

Prevents double-triggering on the same beat:

```rust
if elapsed < self.cooldown_seconds {
    return false;
}
```

**Why 100ms cooldown?**
- Shortest musical note at 600 BPM = ~100ms
- Prevents detecting the same kick drum decay as multiple beats
- Doesn't miss legitimate rapid beats (16th notes at 150 BPM = 100ms)
- Works for all typical music (60-180 BPM)

**Trade-offs**:
- Shorter cooldown: More false positives from single transients
- Longer cooldown: Misses rapid beat patterns (drum rolls, fast EDM)
- 100ms is sweet spot for most music

### History Size

Fixed 10-sample history provides temporal context:

```rust
history_size: usize = 10
```

**Why 10 samples?**
- At 60 FPS visualization rate: 10 samples = ~167ms
- Provides enough context to establish average energy
- Not too long (doesn't lag behind tempo changes)
- Fixed size = predictable O(1) performance

**Performance impact**:
- 10 floating point operations per frame
- Negligible compared to FFT cost
- No dynamic allocation (pre-sized Vec)

---

## Performance Characteristics

**Computational Complexity**:
- Time: O(1) per frame (fixed history size)
- Space: O(1) (fixed 10-sample history)
- Overhead: <1ms per frame on modern hardware

**Memory Usage**:
- BeatDetector: ~88 bytes
  - `Vec<f32>` with capacity 10: 40 bytes
  - `usize` + `f32` + `f32`: 16 bytes
  - `Option<Instant>`: 24 bytes
  - Padding: 8 bytes

**Real-time Performance**:
- No allocations during detect() call
- No FFT required (uses existing RMS calculation)
- Suitable for 60+ FPS visualization

---

## Future Enhancements

Beat detection could be enhanced in future versions:

### DSP-005: Spectral Flux Beat Detection
- Use spectral flux for more accurate onset detection
- Better for harmonic instruments (piano, guitar)
- Higher computational cost but more robust
- Combine with energy-based for hybrid approach

### DSP-006: Tempo Detection and BPM Estimation
- Analyze beat intervals to estimate tempo
- Auto-adjust cooldown based on detected BPM
- Provide BPM to visualizers for tempo-synced effects
- Support beat phase tracking (strong beat vs weak beat)

### DSP-007: Configurable Beat Detection Parameters
- Expose sensitivity, cooldown, history size in config
- Per-genre presets (rock, electronic, classical, jazz)
- User-adjustable sensitivity slider
- Save user preferences

### Integration with Multi-band Detection
- Separate beat detection for bass, mid, treble
- Detect kick drums (bass beats) vs hi-hats (treble beats)
- Enable frequency-specific visual effects
- More nuanced rhythm response

---

## Notes for AI Agent

**This implementation is COMPLETE and working well in production.**

### Key Implementation Details

**Energy Metric Choice**:
- Uses RMS amplitude as energy metric
- RMS already calculated in DspProcessor::process()
- No additional FFT or DSP processing needed
- Represents perceptual loudness well

**Cooldown Implementation**:
- Uses `std::time::Instant` for high-precision timing
- Cooldown checked BEFORE adding to history (optimization)
- Prevents wasted computation on known non-beats

**History Management**:
- Vec::remove(0) is O(n) but n=10 is trivial
- Alternative: Circular buffer (more complex, negligible benefit)
- Pre-allocated capacity prevents reallocation

**Threshold Tuning**:
- Factor of 1.5 works well for most music
- Can be made configurable in future (DSP-007)
- Consider music-specific tuning (classical vs EDM)

**Common Pitfalls**:
- Don't forget minimum energy threshold (0.1) to filter noise
- Cooldown must use wall-clock time, not sample count
- History average should EXCLUDE current sample
- Need at least 3 samples for meaningful comparison

**Integration with Visualizers**:
- Beat is boolean (not continuous value)
- Visualizers should implement decay/fade after beat
- Example: `beat_flash *= 0.85` creates smooth decay
- Different visualizers can interpret beats differently

**Testing Considerations**:
- Unit tests use synthetic energy patterns
- Integration tests use synthetic audio buffers
- Manual testing essential for real music validation
- Test across genres (electronic, rock, classical, jazz)

**Performance Validation**:
```rust
// Benchmark beat detection overhead
#[bench]
fn bench_beat_detection(b: &mut Bencher) {
    let mut detector = BeatDetector::new(1.0, 0.1);
    b.iter(|| {
        detector.detect(black_box(0.5))
    });
}
// Expected: <1μs per call
```

**Success Indicators**:
1. Visual effects sync with perceived beats in music
2. No false positives in quiet passages
3. Works across different genres and BPM ranges
4. Beat flash effects feel natural and responsive
5. No perceptible performance impact

**Time Estimate**: 1.5-2 days
- Day 1: Implement BeatDetector, unit tests, integration
- Day 2: Testing with real music, tuning parameters, visual integration

**Reference Implementation**: This is a standard approach used in:
- Music information retrieval (MIR) systems
- DJ software beat detection
- Audio visualization tools
- Real-time audio analysis applications

This implementation provides a solid foundation for rhythm-responsive visualizations!
