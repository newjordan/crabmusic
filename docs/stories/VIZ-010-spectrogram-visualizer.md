# VIZ-010: Spectrogram (Waterfall) Visualizer

**Status**: In Progress  
**Priority**: High  
**Estimated Effort**: 6 hours  
**Dependencies**: VIZ-004 (Visualizer Trait), DSP-001 (FFT Processor)

## Overview

Implement a spectrogram visualizer that displays frequency content over time as a scrolling waterfall display. This creates a heat map showing how frequencies change, with time flowing from top to bottom (or bottom to top).

## Background

A spectrogram is a visual representation of the spectrum of frequencies in a signal as they vary with time. It's one of the most powerful audio analysis tools, showing:
- Which frequencies are present at any given time
- How frequencies evolve over time
- Patterns in music (beats, melodies, harmonics)

In our implementation:
- Each row represents a moment in time
- Each column represents a frequency bin
- Color intensity represents amplitude/energy
- Display scrolls continuously (waterfall effect)

## Goals

1. ✅ Implement scrolling waterfall display
2. ✅ Use frequency spectrum from FFT
3. ✅ Color-coded intensity (heat map)
4. ✅ Configurable scroll direction (up or down)
5. ✅ Works with all color schemes
6. ✅ Smooth scrolling with history buffer

## Technical Design

### SpectrogramVisualizer Structure

```rust
pub struct SpectrogramVisualizer {
    color_scheme: ColorScheme,
    history_buffer: VecDeque<Vec<f32>>,  // Circular buffer of spectrum snapshots
    max_history: usize,                   // How many rows of history to keep
    scroll_direction: ScrollDirection,    // Up or Down
}

pub enum ScrollDirection {
    Up,    // New data appears at bottom, scrolls up
    Down,  // New data appears at top, scrolls down
}
```

### Algorithm

1. **Capture spectrum**: Get current frequency spectrum from AudioParameters
2. **Add to history**: Push new spectrum to history buffer
3. **Scroll display**: Shift all rows by one
4. **Render rows**: For each row in history:
   - Map frequency bins to columns
   - Map amplitude to color intensity
   - Draw using Braille characters for high resolution

### Frequency Mapping

- Use logarithmic frequency mapping (like spectrum analyzer)
- Focus on audible range (20 Hz - 20 kHz)
- More resolution in lower frequencies (where music lives)

### Color Mapping

- Map amplitude (0.0-1.0) to color intensity
- Use ColorScheme to get colors
- Brighter colors = higher amplitude
- Darker colors = lower amplitude

### History Buffer

- Use `VecDeque` for efficient push/pop
- Keep last N frames (where N = grid height)
- Oldest frames scroll off the top/bottom

## Implementation Plan

### Phase 1: Core Implementation ✅

1. Create `src/visualization/spectrogram.rs`
2. Implement `SpectrogramVisualizer` struct
3. Implement `Visualizer` trait:
   - `update()` - Add new spectrum to history
   - `render()` - Draw scrolling waterfall
   - `name()` - Return "Spectrogram"

### Phase 2: Integration ✅

1. Export `spectrogram` module in `src/visualization/mod.rs`
2. Add `Spectrogram` variant to `VisualizerMode` enum
3. Add to visualizer cycling in `src/main.rs`
4. Update UI overlay to show spectrogram controls

### Phase 3: Testing ✅

1. Test with different music genres
2. Test with all color schemes
3. Test with effects (bloom, scanlines, phosphor)
4. Verify smooth scrolling
5. Test scroll direction toggle

## Acceptance Criteria

- [ ] SpectrogramVisualizer implemented with scrolling waterfall
- [ ] Uses frequency spectrum from AudioParameters
- [ ] Color-coded intensity using ColorScheme
- [ ] History buffer for smooth scrolling
- [ ] Configurable scroll direction
- [ ] 'V' key cycles to spectrogram mode
- [ ] 'D' key toggles scroll direction (up/down)
- [ ] UI overlay shows spectrogram controls
- [ ] Works with all color schemes
- [ ] Works with all effects (bloom, scanlines, phosphor)
- [ ] Smooth 60 FPS performance
- [ ] Documentation complete

## Testing Strategy

### Manual Testing

1. **Basic display**: Verify waterfall scrolls smoothly
2. **Frequency accuracy**: Play pure tones, verify correct frequency position
3. **Color mapping**: Verify loud = bright, quiet = dark
4. **Scroll direction**: Toggle with 'D' key, verify direction changes
5. **Color schemes**: Cycle through all schemes, verify colors work
6. **Effects**: Enable bloom/scanlines/phosphor, verify they enhance display
7. **Music genres**: Test with bass-heavy, treble-heavy, full-spectrum music

## Performance Considerations

- **Memory**: History buffer = height × num_bins × f32 (~10-20 KB)
- **CPU**: Just copying spectrum + rendering (very fast)
- **Expected overhead**: < 1ms per frame

## Visual Design

```
Spectrogram (Scroll Down):
┌────────────────────────────────────┐
│ ████████░░░░░░░░░░░░░░░░░░░░░░░░░░ │ ← Oldest (top)
│ ██████████░░░░░░░░░░░░░░░░░░░░░░░░ │
│ ████████████░░░░░░░░░░░░░░░░░░░░░░ │
│ ██████████████░░░░░░░░░░░░░░░░░░░░ │
│ ████████████████░░░░░░░░░░░░░░░░░░ │
│ ██████████████████░░░░░░░░░░░░░░░░ │
│ ████████████████████░░░░░░░░░░░░░░ │
│ ██████████████████████░░░░░░░░░░░░ │ ← Newest (bottom)
└────────────────────────────────────┘
  Low Freq ──────────────→ High Freq
```

## Future Enhancements

- [ ] Frequency axis labels (Hz markers)
- [ ] Time axis labels (seconds ago)
- [ ] Zoom controls (frequency range)
- [ ] Variable scroll speed
- [ ] Pause/resume scrolling
- [ ] Export spectrogram as image

## References

- Spectrogram: https://en.wikipedia.org/wiki/Spectrogram
- Waterfall display: https://en.wikipedia.org/wiki/Waterfall_plot
- Audio spectrograms in music production

## Notes

- This visualizer will look AMAZING with phosphor effect (trails!)
- Scroll direction preference may vary by user
- Consider making scroll direction persistent in config
- Braille rendering gives us 4x vertical resolution (perfect for spectrograms)

