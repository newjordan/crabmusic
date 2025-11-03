# VIZ-015: 3D Waveform Tunnel Visualizer

**Status**: In Progress  
**Priority**: High  
**Complexity**: Medium-High  
**Estimated Effort**: 3-4 hours

---

## Overview

Create a **3D Waveform Tunnel** visualizer that captures snapshots of sine waves and moves them toward the camera, creating a topological/layered depth effect. The waveforms appear to "crawl" toward the viewer in a repeating fashion, with staggered layers creating a tunnel-like perspective.

### Visual Concept

```
Far away (small):     ~~~
                     ~~~~~
                    ~~~~~~~
Getting closer:    ~~~~~~~~~
                  ~~~~~~~~~~~
                 ~~~~~~~~~~~~~
Near (large):   ~~~~~~~~~~~~~~~
```

Each layer is a frozen snapshot of the waveform at a moment in time, scaled based on distance from camera. As layers move forward, they grow larger, creating depth perception.

---

## User Story

**As a** music visualizer user  
**I want** to see waveforms moving toward me in 3D space  
**So that** I can experience an immersive tunnel effect with topological depth

---

## Technical Design

### Core Structure

```rust
pub struct WaveformTunnelVisualizer {
    color_scheme: ColorScheme,
    layers: VecDeque<WaveformSnapshot>,
    max_layers: usize,
    layer_spacing: f32,
    speed: f32,
    amplitude: f32,
    frequency: f32,
    bass: f32,
    mid: f32,
    treble: f32,
}

struct WaveformSnapshot {
    samples: Vec<f32>,
    depth: f32,  // 0.0 = far away, 1.0 = at camera
    phase: f32,
}
```

### Algorithm

1. **Capture Snapshots**
   - Each frame, capture current waveform state
   - Store amplitude, frequency, phase at that moment
   - Add to front of layer queue

2. **Move Layers Forward**
   - Each layer moves toward camera at `speed` rate
   - `depth += speed * delta_time`
   - When depth >= 1.0, remove layer (reached camera)

3. **Render with Perspective**
   - Far layers (depth near 0.0): Small scale, top of screen
   - Near layers (depth near 1.0): Large scale, bottom of screen
   - Scale factor: `scale = 0.2 + (depth * 0.8)` (20% to 100%)
   - Y position: `y_center = height * depth` (top to bottom)

4. **Topological Stagger**
   - Layers spaced by `layer_spacing` in depth
   - More spacing = fewer layers, more distinct
   - Less spacing = more layers, denser tunnel

### Rendering Strategy

```rust
for layer in &self.layers {
    let scale = 0.2 + (layer.depth * 0.8);
    let y_center = (height as f32 * layer.depth) as usize;
    let amplitude_scaled = layer.amplitude * scale;
    
    // Render waveform at this depth
    for x in 0..width {
        let sample = calculate_wave_sample(x, layer);
        let y_offset = (sample * amplitude_scaled) as i32;
        let y = (y_center as i32 + y_offset).clamp(0, height as i32 - 1);
        
        // Use Braille for smooth curves
        render_braille_point(grid, x, y, color);
    }
}
```

### Color Mapping

- **Depth-based color**: Far = dim, near = bright
- **Frequency-based hue**: Use color scheme with frequency modulation
- **Intensity**: `intensity = 0.3 + (depth * 0.7)` for depth cueing

---

## Implementation Plan

### Phase 1: Core Structure (30 min)
- [ ] Create `src/visualization/waveform_tunnel.rs`
- [ ] Define `WaveformTunnelVisualizer` struct
- [ ] Define `WaveformSnapshot` struct
- [ ] Implement `new()` constructor with default settings

### Phase 2: Snapshot & Movement (45 min)
- [ ] Implement snapshot capture in `update()`
- [ ] Store current waveform state (amplitude, frequency, phase)
- [ ] Implement layer movement (depth += speed)
- [ ] Remove layers when depth >= 1.0
- [ ] Maintain max_layers limit

### Phase 3: Perspective Rendering (60 min)
- [ ] Calculate scale based on depth
- [ ] Calculate y_center based on depth
- [ ] Render each layer with perspective scaling
- [ ] Use Braille characters for smooth curves
- [ ] Apply depth-based color intensity

### Phase 4: Integration (30 min)
- [ ] Add to `VisualizerMode` enum
- [ ] Add to visualizer cycling
- [ ] Export in `visualization/mod.rs`
- [ ] Update main.rs to create visualizer

### Phase 5: Testing & Polish (45 min)
- [ ] Test with different music types
- [ ] Test with effects (bloom, phosphor, scanlines)
- [ ] Adjust speed, spacing, max_layers for best look
- [ ] Write unit tests
- [ ] Document keyboard controls (if any)

---

## Configuration

### Default Settings
```rust
max_layers: 20,           // 20 layers in tunnel
layer_spacing: 0.05,      // 5% depth between layers
speed: 0.02,              // 2% depth per frame (50 frames to traverse)
base_frequency: 2.0,      // 2 cycles across width
amplitude_sensitivity: 1.5,
frequency_sensitivity: 3.0,
```

### Adjustable Parameters
- **Speed**: How fast layers move toward camera
- **Layer spacing**: Distance between layers (affects density)
- **Max layers**: How many layers in tunnel (affects depth)
- **Amplitude sensitivity**: How much audio affects wave height
- **Frequency sensitivity**: How much audio affects wave cycles

---

## Acceptance Criteria

### Must Have
- [x] Waveform snapshots captured each frame
- [x] Layers move toward camera with perspective scaling
- [x] Far layers small at top, near layers large at bottom
- [x] Smooth continuous motion (no stuttering)
- [x] Works with all color schemes
- [x] Integrates with visualizer cycling ('V' key)

### Should Have
- [ ] Depth-based color intensity (far = dim, near = bright)
- [ ] Frequency-based color modulation
- [ ] Smooth Braille rendering for curves
- [ ] Works well with phosphor effect (trails!)
- [ ] Works well with bloom effect (glowing layers)

### Nice to Have
- [ ] Keyboard control for speed adjustment
- [ ] Keyboard control for layer spacing
- [ ] Rotation/twist effect (layers rotate as they approach)
- [ ] Stereo width (left/right channel separation)

---

## Testing Strategy

### Manual Testing
1. **Basic functionality**
   - Run with `cargo run --loopback`
   - Press 'V' to cycle to Waveform Tunnel
   - Verify layers move toward camera
   - Verify perspective scaling (small → large)

2. **With music**
   - Test with bass-heavy music (layers should pulse)
   - Test with high-frequency music (more wave cycles)
   - Test with varying tempo (speed should feel natural)

3. **With effects**
   - Enable phosphor ('H') - should create trailing tunnel
   - Enable bloom ('B') - layers should glow
   - Enable scanlines ('S') - CRT effect on tunnel

### Unit Tests
- `test_tunnel_new()` - Default initialization
- `test_snapshot_capture()` - Snapshot creation
- `test_layer_movement()` - Depth progression
- `test_layer_removal()` - Remove when depth >= 1.0
- `test_max_layers_limit()` - Limit to max_layers

---

## Performance Considerations

### Memory
- **Snapshot storage**: ~20 layers × 512 samples × 4 bytes = ~40 KB
- **Negligible overhead**: Well within acceptable limits

### CPU
- **Snapshot capture**: O(1) - just copy current state
- **Layer movement**: O(n) where n = max_layers (~20)
- **Rendering**: O(n × width) = O(20 × 200) = ~4000 operations
- **Expected overhead**: < 1ms per frame

### Optimization Opportunities
- Use fixed-size array instead of VecDeque if max_layers is constant
- Pre-calculate scale factors for common depths
- Use lookup table for sine wave samples

---

## Visual Examples

### Tunnel Effect
```
Far:    ~~~     (small, dim)
       ~~~~~
      ~~~~~~~
     ~~~~~~~~~
    ~~~~~~~~~~~
   ~~~~~~~~~~~~~
  ~~~~~~~~~~~~~~~
 ~~~~~~~~~~~~~~~~~
~~~~~~~~~~~~~~~~~~~  (large, bright)
```

### With Beat
```
Beat detected:
Far:    ~~~     (compressed)
       ~~~~~
      ~~~~~~~
     ~~~~~~~~~
    ~~~~~~~~~~~
   ~~~~~~~~~~~~~
  ~~~~~~~~~~~~~~~
 ~~~~~~~~~~~~~~~~~
~~~~~~~~~~~~~~~~~~~  (expanded, bright)
```

### With Phosphor Effect
```
Trailing layers create motion blur:
Far:    ░░░     (faded trail)
       ▒▒▒▒▒
      ▓▓▓▓▓▓▓
     ███████████
    ███████████████
   █████████████████
  ███████████████████
 █████████████████████
███████████████████████  (solid, bright)
```

---

## Notes

- This visualizer combines elements of sine wave (smooth curves) and spectrogram (depth/time)
- The "topological" look comes from staggered layers with perspective
- Phosphor effect will make this look INCREDIBLE (trailing tunnel)
- Could be extended with rotation for spiral tunnel effect
- Could add stereo separation (left channel = left side, right = right side)

---

## Related Stories

- **VIZ-001**: Sine Wave Visualizer (base waveform rendering)
- **VIZ-010**: Spectrogram Visualizer (depth/time concept)
- **EFFECTS-006**: Phosphor Glow Effect (will enhance tunnel trails)
- **VIZ-014**: 3D Spectrum Bars (related 3D concept)

---

## Success Metrics

- **Visual impact**: "Super cool" rating from user
- **Performance**: Maintains 60 FPS with 20 layers
- **Effect synergy**: Looks amazing with phosphor trails
- **Music reactivity**: Clearly responds to bass, frequency, amplitude

