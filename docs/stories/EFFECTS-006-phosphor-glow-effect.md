# EFFECTS-006: Phosphor Glow Effect

**Status**: In Progress  
**Priority**: Medium  
**Estimated Effort**: 4 hours  
**Dependencies**: EFFECTS-007 (Effect Pipeline Framework)

## Overview

Implement a phosphor glow effect that simulates the temporal persistence of CRT monitor phosphors. Bright pixels fade slowly over time, creating trailing effects that give the visualization a retro CRT monitor feel.

## Background

Old CRT monitors used phosphor coatings that would glow when struck by electrons. The phosphor would continue to glow for a short time after being excited, creating a "persistence" or "ghosting" effect. This was especially noticeable with bright, moving elements.

This effect is perfect for music visualizations because:
- Bright peaks leave glowing trails as they move
- Creates smooth motion blur for fast-moving elements
- Adds warmth and character to the visualization
- Authentic retro aesthetic

## Goals

1. ✅ Implement temporal persistence (frame-to-frame color memory)
2. ✅ Configurable decay rate (how fast the glow fades)
3. ✅ Configurable intensity (strength of persistence effect)
4. ✅ Efficient implementation (minimal memory overhead)
5. ✅ Works with all visualizers

## Technical Design

### PhosphorGlowEffect Structure

```rust
pub struct PhosphorGlowEffect {
    enabled: bool,
    intensity: f32,        // 0.0-1.0, strength of persistence
    decay_rate: f32,       // 0.0-1.0, how fast glow fades (higher = faster fade)
    previous_frame: Vec<Option<Color>>, // Store previous frame colors
}
```

### Algorithm

1. **Store current frame**: Before applying effect, capture current grid state
2. **Blend with previous frame**: For each cell, blend current color with previous frame color
3. **Apply decay**: Fade previous frame colors by decay_rate
4. **Composite**: Mix current and decayed previous frame based on intensity

### Blending Formula

```
decayed_color = previous_color * (1.0 - decay_rate)
final_color = mix(current_color, decayed_color, intensity)
```

Where:
- `decay_rate = 0.0` → no decay (infinite persistence)
- `decay_rate = 1.0` → instant decay (no persistence)
- `intensity = 0.0` → no effect (current frame only)
- `intensity = 1.0` → maximum persistence

### Default Values

- **Decay rate**: 0.3 (moderate fade, ~3-4 frames to disappear)
- **Intensity**: 0.7 (strong but not overwhelming)

## Implementation Plan

### Phase 1: Core Implementation ✅

1. Create `src/effects/phosphor.rs`
2. Implement `PhosphorGlowEffect` struct
3. Implement `Effect` trait:
   - `apply()` - Blend current frame with decayed previous frame
   - `name()` - Return "Phosphor"
   - Standard enable/intensity methods

### Phase 2: Integration ✅

1. Export `phosphor` module in `src/effects/mod.rs`
2. Add to effect pipeline in `src/main.rs`
3. Add keyboard control ('P' key to toggle)
4. Update UI overlay to show phosphor status

### Phase 3: Testing ✅

1. Test with sine wave visualizer (smooth trails)
2. Test with spectrum analyzer (peak trails)
3. Test with oscilloscope (waveform trails)
4. Verify performance (should be fast - just color blending)

## Acceptance Criteria

- [ ] PhosphorGlowEffect implemented with temporal persistence
- [ ] Configurable decay_rate and intensity
- [ ] Previous frame buffer managed efficiently
- [ ] 'P' key toggles phosphor on/off
- [ ] '[' and ']' keys adjust intensity when phosphor selected
- [ ] UI overlay shows phosphor status (e.g., "P:70%")
- [ ] Works with all three visualizers
- [ ] No visible performance impact
- [ ] Unit tests for core logic
- [ ] Documentation complete

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_phosphor_new() {
    let effect = PhosphorGlowEffect::new(0.3, 0.7);
    assert_eq!(effect.decay_rate(), 0.3);
    assert_eq!(effect.intensity(), 0.7);
}

#[test]
fn test_phosphor_decay() {
    // Test that colors fade over time
}

#[test]
fn test_phosphor_blend() {
    // Test color blending between frames
}
```

### Manual Testing

1. **Sine wave**: Should see smooth trailing waves
2. **Spectrum**: Peaks should leave glowing trails as they move
3. **Oscilloscope**: Waveform should have motion blur
4. **Intensity control**: '[' and ']' should adjust trail length
5. **Toggle**: 'P' should clearly enable/disable effect

## Performance Considerations

- **Memory**: One additional frame buffer (width × height × Color)
- **CPU**: Simple color blending per cell (very fast)
- **Expected overhead**: < 1ms per frame

## Future Enhancements

- [ ] Different decay curves (linear, exponential, logarithmic)
- [ ] Color-specific decay (different rates for R, G, B)
- [ ] Brightness-dependent decay (brighter pixels persist longer)
- [ ] Phosphor "bloom" (bright pixels spread slightly as they fade)

## References

- CRT phosphor persistence: https://en.wikipedia.org/wiki/Phosphor#Persistence
- Retro CRT shader techniques
- LibRetro phosphor persistence implementation

## Notes

- This effect works best when applied AFTER other effects (bloom, scanlines)
- Should be last in the effect pipeline for best results
- Creates natural motion blur without complex algorithms
- Very efficient - just frame buffer and color blending

