# EFFECTS-004: Scanline Effect

**Status**: In Progress  
**Priority**: P1 (High)  
**Estimated Effort**: 2-3 hours  
**Dependencies**: EFFECTS-007 (Effect Pipeline Framework) ✅  
**Blocks**: None

---

## 📋 Story Description

Implement CRT-style horizontal scanlines as the first concrete visual effect. This effect adds alternating dark lines across the display to simulate the scan lines visible on vintage CRT monitors, creating an authentic retro aesthetic.

This is intentionally the **simplest effect** to implement, serving as a validation that the Effect Pipeline Framework works correctly for real-world effects.

---

## 🎯 Acceptance Criteria

### Functional Requirements
- [ ] `ScanlineEffect` struct implements the `Effect` trait
- [ ] Scanlines are drawn as horizontal lines across the entire grid
- [ ] Configurable scanline spacing (e.g., every 2nd, 3rd, or 4th row)
- [ ] Configurable scanline intensity (0.0 = invisible, 1.0 = maximum darkness)
- [ ] Scanlines dim existing content without replacing it
- [ ] Effect can be toggled on/off via the Effect trait interface
- [ ] Effect respects the `enabled` flag

### Visual Requirements
- [ ] Scanlines are evenly spaced across the display
- [ ] Scanlines create a subtle darkening effect (not complete blackout)
- [ ] Effect works with all color schemes
- [ ] Effect works with all visualizers (sine wave, spectrum, oscilloscope)
- [ ] No flickering or visual artifacts

### Performance Requirements
- [ ] Scanline effect completes in <1ms for 200x100 grid
- [ ] No memory allocations during apply()
- [ ] Minimal CPU overhead

### Testing Requirements
- [ ] Unit tests for ScanlineEffect creation
- [ ] Unit tests for enable/disable functionality
- [ ] Unit tests for intensity control
- [ ] Unit tests for spacing configuration
- [ ] Performance benchmark test
- [ ] Visual validation with all visualizers

---

## 🏗️ Technical Approach

### Implementation Strategy

**File**: `src/effects/scanline.rs`

```rust
pub struct ScanlineEffect {
    enabled: bool,
    intensity: f32,      // 0.0 = invisible, 1.0 = maximum darkness
    spacing: usize,      // Draw scanline every N rows (e.g., 2 = every other row)
}

impl ScanlineEffect {
    pub fn new(spacing: usize) -> Self;
    pub fn spacing(&self) -> usize;
    pub fn set_spacing(&mut self, spacing: usize);
}

impl Effect for ScanlineEffect {
    fn apply(&mut self, grid: &mut GridBuffer, _params: &AudioParameters) {
        if !self.enabled { return; }
        
        for y in (0..grid.height()).step_by(self.spacing) {
            for x in 0..grid.width() {
                // Dim the cell by reducing color brightness
                if let Some(cell) = grid.get_cell_mut(x, y) {
                    if let Some(color) = cell.foreground_color {
                        // Reduce RGB values by intensity factor
                        let factor = 1.0 - (self.intensity * 0.5); // Max 50% dimming
                        cell.foreground_color = Some(Color::new(
                            (color.r as f32 * factor) as u8,
                            (color.g as f32 * factor) as u8,
                            (color.b as f32 * factor) as u8,
                        ));
                    }
                }
            }
        }
    }
    
    fn name(&self) -> &str { "Scanline" }
    fn is_enabled(&self) -> bool { self.enabled }
    fn set_enabled(&mut self, enabled: bool) { self.enabled = enabled; }
    fn intensity(&self) -> f32 { self.intensity }
    fn set_intensity(&mut self, intensity: f32) { self.intensity = intensity.clamp(0.0, 1.0); }
}
```

### Algorithm Details

1. **Iterate through scanline rows**: Use `step_by(spacing)` to process every Nth row
2. **Dim each cell**: Multiply RGB values by `(1.0 - intensity * 0.5)` to darken
3. **Preserve content**: Only modify color, not character
4. **Skip empty cells**: Only process cells with foreground color

### Integration Points

**Add to main.rs initialization**:
```rust
// Add scanline effect to pipeline
effect_pipeline.add_effect(Box::new(effects::scanline::ScanlineEffect::new(2)));
```

**Export in src/effects/mod.rs**:
```rust
pub mod scanline;
```

---

## 🧪 Testing Strategy

### Unit Tests
- `test_scanline_new()` - Verify default initialization
- `test_scanline_enable_disable()` - Toggle functionality
- `test_scanline_intensity()` - Intensity clamping and application
- `test_scanline_spacing()` - Spacing configuration
- `test_scanline_apply()` - Verify scanlines are drawn correctly
- `test_scanline_disabled()` - Verify disabled effect does nothing
- `test_scanline_performance()` - Benchmark <1ms for 200x100 grid

### Integration Tests
- Test with sine wave visualizer
- Test with spectrum visualizer
- Test with oscilloscope visualizer
- Test with different color schemes
- Test with different spacing values (1, 2, 3, 4)
- Test with different intensity values (0.0, 0.5, 1.0)

---

## 📊 Success Metrics

- ✅ All unit tests pass
- ✅ Performance benchmark <1ms
- ✅ Visual validation with all visualizers
- ✅ No regressions in existing functionality
- ✅ Code review approved
- ✅ Documentation complete

---

## 🚀 Implementation Notes

### Design Decisions

1. **Spacing over frequency**: Use `spacing` (every Nth row) instead of frequency for simplicity
2. **Max 50% dimming**: Prevent complete blackout by capping at 50% darkness
3. **No audio reactivity**: Keep scanlines static for authentic CRT look (can add later)
4. **Color-only modification**: Preserve characters, only dim colors

### Future Enhancements

- [ ] Audio-reactive scanline intensity (pulse with beat)
- [ ] Animated scanlines (rolling effect)
- [ ] Configurable scanline color (not just dimming)
- [ ] Vertical scanlines option
- [ ] Interlaced scanline pattern

### Known Limitations

- Only works with colored output (no effect on monochrome)
- Fixed horizontal orientation
- No sub-pixel positioning (terminal grid aligned)

---

## 📝 Checklist

- [ ] Create `src/effects/scanline.rs`
- [ ] Implement `ScanlineEffect` struct
- [ ] Implement `Effect` trait for `ScanlineEffect`
- [ ] Export module in `src/effects/mod.rs`
- [ ] Add to effect pipeline in `src/main.rs`
- [ ] Write unit tests
- [ ] Write performance benchmark
- [ ] Test with all visualizers
- [ ] Update documentation
- [ ] Code review
- [ ] Commit and push

---

## 🔗 Related Stories

- **EFFECTS-007**: Effect Pipeline Framework (dependency) ✅
- **EFFECTS-003**: Bloom Effect (next effect)
- **EFFECTS-005**: CRT Curve & Distortion (complementary retro effect)
- **EFFECTS-006**: Phosphor Glow Effect (complementary retro effect)

---

**Story Created**: 2025-01-03  
**Last Updated**: 2025-01-03

