# EFFECTS-007: Effect Pipeline Framework

**Epic**: Visual Effects System
**Priority**: P0 (Foundation - blocks all other effect stories)
**Estimated Effort**: 3-4 days
**Status**: ✅ Complete

## Description

Build the foundational framework for post-processing visual effects that can be applied to visualizer output. This framework will enable effects like scanlines, bloom, CRT distortion, and phosphor glow to be layered on top of any visualizer (sine wave, spectrum, oscilloscope).

The effect pipeline sits between the visualizer's `render()` call and the final terminal display, allowing effects to transform the `GridBuffer` before it reaches the screen. Effects should be composable, configurable, and performant enough to run at 60 FPS.

This is the critical foundation that enables all subsequent effect stories (EFFECTS-003 through EFFECTS-006). The design must be flexible enough to support both simple effects (scanlines) and complex multi-pass effects (bloom with blur).

## Acceptance Criteria

- [ ] `Effect` trait defined with clear interface for transforming GridBuffer
- [ ] `EffectPipeline` struct that manages ordered list of effects
- [ ] Effects can be enabled/disabled at runtime without recreating pipeline
- [ ] Effects can be added/removed from pipeline dynamically
- [ ] Effect parameters can be adjusted at runtime (intensity, strength, etc.)
- [ ] Pipeline processes effects in order with minimal overhead (<2ms for empty pipeline)
- [ ] Integration point in main render loop (between visualizer.render() and renderer.render())
- [ ] Example "passthrough" effect that demonstrates the interface
- [ ] Example "test pattern" effect for validation (e.g., grid overlay)
- [ ] Comprehensive unit tests for Effect trait and EffectPipeline
- [ ] Documentation with examples of implementing custom effects

## Technical Approach

### 1. Effect Trait Design

```rust
/// Trait for post-processing visual effects
///
/// Effects transform a GridBuffer in-place or create a new buffer.
/// They can read audio parameters for audio-reactive effects.
pub trait Effect {
    /// Apply the effect to a grid buffer
    ///
    /// # Arguments
    /// * `grid` - Grid buffer to transform (may be modified in-place)
    /// * `params` - Current audio parameters (for audio-reactive effects)
    ///
    /// # Returns
    /// Transformed grid buffer (may be the same buffer or a new one)
    fn apply(&mut self, grid: &mut GridBuffer, params: &AudioParameters);
    
    /// Get effect name for debugging/UI
    fn name(&self) -> &str;
    
    /// Check if effect is currently enabled
    fn is_enabled(&self) -> bool;
    
    /// Enable or disable the effect
    fn set_enabled(&mut self, enabled: bool);
    
    /// Get effect intensity (0.0-1.0)
    fn intensity(&self) -> f32;
    
    /// Set effect intensity (0.0-1.0)
    fn set_intensity(&mut self, intensity: f32);
}
```

### 2. Effect Pipeline

```rust
/// Pipeline for composing multiple effects
pub struct EffectPipeline {
    effects: Vec<Box<dyn Effect>>,
    enabled: bool,
}

impl EffectPipeline {
    pub fn new() -> Self;
    pub fn add_effect(&mut self, effect: Box<dyn Effect>);
    pub fn remove_effect(&mut self, name: &str) -> Option<Box<dyn Effect>>;
    pub fn apply(&mut self, grid: &mut GridBuffer, params: &AudioParameters);
    pub fn set_enabled(&mut self, enabled: bool);
    pub fn is_enabled(&self) -> bool;
    pub fn get_effect_mut(&mut self, name: &str) -> Option<&mut Box<dyn Effect>>;
}
```

### 3. Integration with Main Loop

Current flow:
```
AudioParameters → Visualizer::update() → Visualizer::render() → GridBuffer → TerminalRenderer::render()
```

New flow with effects:
```
AudioParameters → Visualizer::update() → Visualizer::render() → GridBuffer 
    → EffectPipeline::apply() → GridBuffer → TerminalRenderer::render()
```

Integration point in `main.rs`:
```rust
// 4. Render visualization to grid
let (width, height) = self.renderer.dimensions();
let mut grid = GridBuffer::new(width as usize, height as usize);
self.visualizer.render(&mut grid);

// 5. Apply post-processing effects (NEW!)
self.effect_pipeline.apply(&mut grid, &audio_params);

// 6. Add UI overlay
self.add_ui_overlay(&mut grid);

// 7. Update terminal display
self.renderer.render(&grid)?;
```

### 4. Example Effects for Validation

**PassthroughEffect** - Does nothing, validates interface:
```rust
pub struct PassthroughEffect {
    enabled: bool,
}

impl Effect for PassthroughEffect {
    fn apply(&mut self, grid: &mut GridBuffer, _params: &AudioParameters) {
        // Do nothing - passthrough
    }
    fn name(&self) -> &str { "Passthrough" }
    // ... other trait methods
}
```

**GridOverlayEffect** - Draws test grid for validation:
```rust
pub struct GridOverlayEffect {
    enabled: bool,
    intensity: f32,
    spacing: usize,  // Grid line spacing
}

impl Effect for GridOverlayEffect {
    fn apply(&mut self, grid: &mut GridBuffer, _params: &AudioParameters) {
        if !self.enabled { return; }
        
        let color = Color::new(
            (50.0 * self.intensity) as u8,
            (50.0 * self.intensity) as u8,
            (50.0 * self.intensity) as u8,
        );
        
        // Draw vertical lines
        for x in (0..grid.width()).step_by(self.spacing) {
            for y in 0..grid.height() {
                grid.set_cell_with_color(x, y, '│', color);
            }
        }
        
        // Draw horizontal lines
        for y in (0..grid.height()).step_by(self.spacing) {
            for x in 0..grid.width() {
                grid.set_cell_with_color(x, y, '─', color);
            }
        }
    }
    fn name(&self) -> &str { "GridOverlay" }
    // ... other trait methods
}
```

### 5. File Structure

```
src/effects/
├── mod.rs              # Effect trait, EffectPipeline
├── passthrough.rs      # PassthroughEffect (example)
└── grid_overlay.rs     # GridOverlayEffect (test pattern)
```

### 6. Performance Considerations

- Effects must be fast (<2ms per effect at 60 FPS)
- Use in-place transformations when possible (avoid allocations)
- Consider caching/memoization for expensive operations
- Profile with `cargo flamegraph` to identify bottlenecks
- Disabled effects should have zero overhead (early return)

## Dependencies

**Depends on**: 
- VIZ-001 (GridBuffer) ✅ Complete
- VIZ-004 (Visualizer trait) ✅ Complete
- RENDER-002 (Ratatui integration) ✅ Complete

**Blocks**: 
- EFFECTS-003 (Bloom Effect)
- EFFECTS-004 (Scanline Effect)
- EFFECTS-005 (CRT Curve & Distortion)
- EFFECTS-006 (Phosphor Glow Effect)

## Testing Requirements

### Unit Tests
- [ ] Effect trait implementation (passthrough, grid overlay)
- [ ] EffectPipeline add/remove effects
- [ ] EffectPipeline enable/disable
- [ ] Effect ordering (effects applied in correct sequence)
- [ ] Effect intensity adjustment
- [ ] Empty pipeline (no effects) has minimal overhead

### Integration Tests
- [ ] Pipeline integration with sine wave visualizer
- [ ] Pipeline integration with spectrum visualizer
- [ ] Pipeline integration with oscilloscope visualizer
- [ ] Multiple effects in sequence
- [ ] Runtime effect toggling

### Performance Tests
- [ ] Benchmark empty pipeline overhead (<0.5ms)
- [ ] Benchmark passthrough effect (<0.1ms)
- [ ] Benchmark grid overlay effect (<1ms)
- [ ] Full pipeline with 4 effects (<8ms total)

## Notes for AI Agent

### Implementation Order
1. **Create `src/effects/mod.rs`** with Effect trait and EffectPipeline
2. **Create `src/effects/passthrough.rs`** as simplest example
3. **Create `src/effects/grid_overlay.rs`** as visual test pattern
4. **Add to `src/lib.rs`**: `pub mod effects;`
5. **Integrate into `main.rs`**: Add EffectPipeline field to App struct
6. **Add keyboard controls**: Toggle effects on/off (e.g., 'E' key)
7. **Write tests**: Unit tests for trait and pipeline
8. **Document**: Add examples to module docs

### Key Design Decisions
- **In-place transformation**: Effects modify GridBuffer directly (no copying)
- **Audio-reactive**: Effects receive AudioParameters for beat sync, etc.
- **Runtime configuration**: All effects support enable/disable and intensity
- **Composable**: Pipeline applies effects in order, each sees previous effect's output
- **Zero-cost when disabled**: Disabled effects return immediately

### Integration Points
- `App` struct in `main.rs` needs `effect_pipeline: EffectPipeline` field
- Initialize in `App::new()`: `effect_pipeline: EffectPipeline::new()`
- Call in render loop after `visualizer.render()`: `self.effect_pipeline.apply(&mut grid, &audio_params)`
- Add keyboard handler for 'E' key to toggle effects

### Testing Strategy
- Start with PassthroughEffect to validate trait design
- Use GridOverlayEffect to visually confirm effects are being applied
- Benchmark with `cargo bench` or manual timing in render loop
- Test with all three visualizers to ensure compatibility

### Future Extensibility
This framework should support:
- Multi-pass effects (e.g., bloom = brighten + blur + composite)
- Effect parameters beyond intensity (per-effect configuration)
- Effect presets (saved combinations of effects with settings)
- Audio-reactive parameters (e.g., scanline speed synced to BPM)
- Effect transitions (smooth enable/disable, parameter interpolation)

## Success Metrics

- ✅ Effect trait compiles and is easy to implement
- ✅ Pipeline integrates cleanly into main loop
- ✅ GridOverlayEffect visibly overlays grid on visualizers
- ✅ Effects can be toggled on/off with 'E' key
- ✅ Performance: <2ms overhead for empty pipeline
- ✅ All tests pass
- ✅ Ready to implement concrete effects (scanlines, bloom, etc.)

