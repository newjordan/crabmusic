# [VIS-001] Harmonic-Reactive Visualization Switching

**Epic**: Advanced Visualizations
**Priority**: P3 (New Feature, Complex)
**Estimated Effort**: 3-5 days
**Status**: Draft

---

## Description

Implement a system that analyzes the harmonic content of the music to identify prominent frequency ratios (e.g., musical intervals like a perfect fifth, 3:2). Based on the detected harmonic character, the system will automatically select or influence a visualization algorithm that reflects that character. For instance, detecting a strong 3:2 harmony could trigger a Lissajous-curve visualizer set to a 3:2 frequency ratio.

**Goal**: Create a deeply synergistic experience where the visualization's geometry directly reflects the music's harmonic structure, offering a more intelligent and responsive visual accompaniment to the music.

---

## User Story

**As a** CrabMusic user,
**I want** the visualizer to dynamically change patterns based on the harmonic content of the music (e.g., specific musical intervals),
**So that** the visual experience feels profoundly integrated with the underlying musical structure, creating complex and beautiful patterns like Lissajous curves that match the harmonies.

----- 

## Acceptance Criteria

*   A new `HarmonicAnalyzer` module is created within `src/dsp/`.
*   The `HarmonicAnalyzer` can process the FFT `spectrum` to identify the most prominent frequency peaks.
*   The analyzer can accurately determine and simplify common frequency ratios (e.g., 1:1, 1:2, 2:1, 3:2, 4:3) from detected peaks in synthetic test tones.
*   The `AudioParameters` struct is updated to include `dominant_ratio: Option<(u32, u32)>` (representing simplified integer ratio) and `ratio_confidence: f32` (indicating the clarity/stability of the detected ratio).
*   A mechanism is implemented in the main application loop to dynamically select or influence visualizers based on the `dominant_ratio` present in `AudioParameters`.
*   A basic `LissajousVisualizer` is created or modified to dynamically adjust its X:Y frequency ratio based on the `dominant_ratio` from `AudioParameters`.
*   Unit tests validate the `HarmonicAnalyzer`'s ability to detect and simplify ratios from various synthetic spectra.
*   Integration tests demonstrate the dynamic switching or influencing of the `LissajousVisualizer` based on a changing dominant ratio.
*   Performance impact of harmonic analysis remains negligible (<1ms overhead).

---

## Technical Approach

### Phase 1: `HarmonicAnalyzer` Module

**File**: `src/dsp/harmonic_analyzer.rs` (new module)

1.  **Peak Detection**: Implement logic to identify the most prominent peaks in the FFT `spectrum`. This could involve filtering noise and identifying local maxima above a certain threshold.
2.  **Ratio Calculation**: For the top N peaks, calculate the frequency ratios between them. Focus on the strongest and most stable relationships.
3.  **Ratio Simplification**: Convert complex float ratios into simplified integer ratios (e.g., 1.499 -> 3:2). This is crucial for matching musical intervals.
4.  **Confidence Metric**: Develop a `ratio_confidence` metric based on the clarity of the peaks, the precision of the ratio, and stability over time.

### Phase 2: `DspProcessor` Integration

**File**: `src/dsp/mod.rs`

1.  **Instantiate Analyzer**: Add `harmonic_analyzer: HarmonicAnalyzer` to the `DspProcessor` struct and initialize it in `DspProcessor::new()`.
2.  **Process and Expose**: In `DspProcessor::process()`, call the `HarmonicAnalyzer` with the current `spectrum`. Retrieve the `dominant_ratio` and `ratio_confidence`.
3.  **Extend `AudioParameters`**: Add `pub dominant_ratio: Option<(u32, u32)>` and `pub ratio_confidence: f32` to the `AudioParameters` struct.

### Phase 3: Dynamic Visualizer Management

**File**: `src/main.rs` (or a new `src/visualization/manager.rs`)

1.  **Visualizer Manager**: Create a system that can hold multiple visualizer instances (`Box<dyn Visualizer>`).
2.  **Logic for Switching**: Within the main application loop (e.g., `main.rs`), check `audio_params.dominant_ratio` and `audio_params.ratio_confidence` each frame.
3.  **Lissajous Visualizer**: Develop or adapt a `LissajousVisualizer` that can accept `dominant_ratio` as parameters for its X and Y axis frequencies/phases. When a confident ratio is detected, switch to or configure this visualizer.

---

## Dependencies

*   **Depends on**: DSP-001 (FFT processor), `waveform_left` and `waveform_right` extraction (for Lissajous curves).
*   **Enables**: A new paradigm of musically reactive visuals, showcasing harmonic complexity.

---

## Notes for AI Agent

*   The `HarmonicAnalyzer` is a new, complex component. Break down its implementation into smaller steps (peak detection, ratio calculation, simplification, confidence).
*   For peak detection, consider existing digital signal processing techniques for identifying local maxima in spectral data.
*   Ratio simplification is a mathematical task (e.g., using Euclidean algorithm to find GCD).
*   The visualizer switching mechanism should be flexible enough to allow for adding more ratio-responsive visualizers in the future.
*   Start with detecting simple, clear ratios first, expanding complexity later.

---
