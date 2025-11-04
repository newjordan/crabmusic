# [DSP-007] Multi-Band Beat Detection (MilkDrop-Inspired)

**Epic**: DSP Processing
**Priority**: P2 (Enhancement to Existing Beat Detection)
**Estimated Effort**: 1-2 days
**Status**: Draft

---

## Description

Enhance the existing beat detection system by implementing multi-band detection, inspired by classic music visualizers like MilkDrop. This involves applying beat detection logic not just to the overall audio energy, but also to the energy of specific frequency bands (e.g., bass, mid, treble) independently.

**Goal**: Allow visualizations to respond differently to various rhythmic components, such as triggering a heavy "thump" effect for a bass kick and a sharp "flash" for a snare hit or hi-hat. This provides a more nuanced and musically intelligent visual response.

---

## User Story

**As a** CrabMusic user,
**I want** the visualizer to react distinctly to different rhythmic elements like kick drums and snares,
**So that** the visual experience is more dynamic, detailed, and accurately reflects the musical texture.

---

## Acceptance Criteria

*   `DspProcessor` is modified to instantiate and utilize separate `BeatDetector` instances for at least the `bass` and `treble` frequency bands.
*   The `DspProcessor::process()` method calls the `detect()` method on these band-specific detectors using the respective band energies.
*   The `AudioParameters` struct is updated to include new boolean fields: `bass_beat: bool` and `treble_beat: bool`.
*   The main `beat: bool` field in `AudioParameters` continues to represent a hybrid beat (e.g., `overall_energy_beat || spectral_flux_beat || bass_beat || treble_beat`).
*   Unit tests are added to `src/dsp/mod.rs` to verify the correct functionality of band-specific beat detection (e.g., a pure bass tone triggers `bass_beat` but not `treble_beat`).
*   Integration tests confirm that visualizers can access and react to `bass_beat` and `treble_beat` independently.
*   Performance impact remains negligible (<1ms overhead per frame).
*   Configuration options for band-specific sensitivity and cooldown are considered and potentially added to `BeatDetectionConfig`.

---

## Technical Approach

### Phase 1: `DspProcessor` and `AudioParameters` Extension

**File**: `src/dsp/mod.rs`

1.  **Add Band-Specific Detectors**:
    *   Modify the `DspProcessor` struct to include additional `BeatDetector` instances, e.g., `bass_beat_detector: BeatDetector` and `treble_beat_detector: BeatDetector`.
    *   Initialize these new detectors in `DspProcessor::new()`.

2.  **Process Band Energies**:
    *   In `DspProcessor::process()`, after extracting `bass`, `mid`, and `treble` energies, call the `detect()` method on the new band-specific detectors:
        ```rust
        let bass_beat = self.bass_beat_detector.detect(bass);
        let treble_beat = self.treble_beat_detector.detect(treble);
        ```
    *   Update the main `beat` flag to include these new detections:
        ```rust
        let beat = beat_energy || beat_flux || bass_beat || treble_beat;
        ```

3.  **Extend `AudioParameters`**:
    *   Add `pub bass_beat: bool` and `pub treble_beat: bool` to the `AudioParameters` struct.
    *   Populate these fields in the `DspProcessor::process()` return.

### Phase 2: Configuration (Optional but Recommended)

**File**: `src/config.rs` (and `src/dsp/mod.rs`)

1.  **Update `BeatDetectionConfig`**:
    *   Add fields for `bass_sensitivity`, `bass_cooldown_seconds`, `treble_sensitivity`, `treble_cooldown_seconds`.
2.  **Apply Configuration**:
    *   Modify `DspProcessor::configure_beat_detection()` to apply these new configuration values to the respective band-specific `BeatDetector` instances.

### Phase 3: Testing

**File**: `src/dsp/mod.rs` (unit tests) and `tests/` (integration tests)

1.  **Unit Tests**:
    *   Add tests that use synthetic audio (e.g., a pure low-frequency sine wave) to ensure `bass_beat` triggers correctly while `treble_beat` does not, and vice-versa.
2.  **Integration Tests**:
    *   Create or modify integration tests to verify that visualizers can access and respond to the new `bass_beat` and `treble_beat` flags.
    *   Test with more complex synthetic patterns that simulate kick and snare hits.

---

## Dependencies

*   **Depends on**: DSP-004 (Energy-based beat detection), DSP-005 (Spectral flux beat detection).
*   **Enables**: More detailed and responsive visual effects, improved musicality of visualizations.

---

## Notes for AI Agent

*   Re-use the existing `BeatDetector` struct and its logic. The primary change is instantiating and managing multiple instances for different frequency bands.
*   Ensure the `AudioParameters` struct is updated correctly and that the `Default` implementation is adjusted.
*   Consider how to best expose configuration for these new detectors (e.g., separate fields in `BeatDetectionConfig` or a more generic band-specific configuration map).
*   Performance is critical; ensure the added detectors do not introduce significant overhead.

---
