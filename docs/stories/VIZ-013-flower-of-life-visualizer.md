# VIZ-013: Flower of Life Visualizer

**Epic**: Sacred Geometry
**Priority**: P1 (High - User excitement area)
**Estimated Effort**: 5-7 days
**Status**: ✅ Complete

## Description

Implement the classic **Flower of Life** sacred geometry pattern as an audio-reactive visualizer. The Flower of Life is an ancient geometric pattern consisting of multiple evenly-spaced, overlapping circles arranged in a hexagonal pattern, creating a flower-like appearance.

This visualizer will render the pattern using Braille characters for smooth circles and make it audio-reactive by:
- Expanding/contracting rings based on bass/amplitude
- Rotating the entire pattern based on mid frequencies
- Pulsing individual circles on beat detection
- Color-cycling through the pattern based on treble

The Flower of Life is one of the most recognizable sacred geometry patterns and will provide a mesmerizing, meditative visualization experience.

## Acceptance Criteria

- [ ] `FlowerOfLifeVisualizer` struct implements `Visualizer` trait
- [ ] Renders overlapping circles in hexagonal pattern (classic 7-circle or 19-circle configuration)
- [ ] Uses `BrailleGrid` for smooth, high-resolution circles
- [ ] Configurable number of rings (1-5 rings, where ring 1 = 7 circles, ring 2 = 19 circles, etc.)
- [ ] Audio-reactive features:
  - [ ] Bass drives ring expansion/contraction (pulsing effect)
  - [ ] Mid frequencies control rotation speed
  - [ ] Beat detection triggers flash/pulse
  - [ ] Amplitude affects overall scale
  - [ ] Treble drives color cycling
- [ ] Configuration struct `FlowerOfLifeConfig` with parameters:
  - [ ] `num_rings` (1-5) - number of concentric rings
  - [ ] `base_radius` (10-50) - base circle radius in dots
  - [ ] `rotation_speed` (0.0-2.0) - rotation multiplier
  - [ ] `pulse_intensity` (0.0-1.0) - how much bass affects size
  - [ ] `use_color` (bool) - enable color schemes
- [ ] Smooth animation (60 FPS capable)
- [ ] Integration with existing color schemes
- [ ] Keyboard control to cycle through ring counts (in main app)
- [ ] Unit tests for circle positioning math
- [ ] Integration test with audio parameters
- [ ] Documentation with examples

## Technical Approach

### 1. Flower of Life Geometry

The Flower of Life is constructed by:
1. Start with a central circle
2. Place 6 circles around it, each centered on the circumference of the central circle
3. Continue adding rings by placing circles at intersection points

**Mathematical approach**:
```rust
// For a hexagonal pattern, circles are placed at 60° intervals
// Ring 0: 1 circle (center)
// Ring 1: 6 circles around center
// Ring 2: 12 circles around ring 1
// Ring 3: 18 circles around ring 2

fn calculate_circle_positions(num_rings: usize, base_radius: f32) -> Vec<(f32, f32)> {
    let mut positions = vec![(0.0, 0.0)]; // Center circle
    
    for ring in 1..=num_rings {
        let num_circles = ring * 6;
        let ring_radius = base_radius * 2.0 * ring as f32;
        
        for i in 0..num_circles {
            let angle = (i as f32 / num_circles as f32) * TAU;
            let x = ring_radius * angle.cos();
            let y = ring_radius * angle.sin();
            positions.push((x, y));
        }
    }
    
    positions
}
```

### 2. Audio Reactivity

```rust
pub struct FlowerOfLifeVisualizer {
    config: FlowerOfLifeConfig,
    color_scheme: ColorScheme,
    
    // Animation state
    rotation: f32,           // Current rotation angle
    pulse_scale: f32,        // Current pulse scale (1.0 = normal)
    beat_flash: f32,         // Beat flash intensity (0.0-1.0)
    
    // Smoothed audio parameters
    amplitude: f32,
    bass: f32,
    mid: f32,
    treble: f32,
}

impl Visualizer for FlowerOfLifeVisualizer {
    fn update(&mut self, params: &AudioParameters) {
        // Smooth audio parameters
        let smoothing = 0.15;
        self.amplitude = lerp(self.amplitude, params.amplitude, smoothing);
        self.bass = lerp(self.bass, params.bass, smoothing);
        self.mid = lerp(self.mid, params.mid, smoothing);
        self.treble = lerp(self.treble, params.treble, smoothing);
        
        // Update rotation based on mid frequencies
        self.rotation += self.mid * self.config.rotation_speed * 0.02;
        
        // Update pulse scale based on bass
        let target_scale = 1.0 + self.bass * self.config.pulse_intensity * 0.3;
        self.pulse_scale = lerp(self.pulse_scale, target_scale, 0.2);
        
        // Update beat flash
        if params.beat_detected {
            self.beat_flash = 1.0;
        } else {
            self.beat_flash *= 0.85; // Decay
        }
    }
    
    fn render(&self, grid: &mut GridBuffer) {
        let width = grid.width();
        let height = grid.height();
        let mut braille = BrailleGrid::new(width, height);
        
        // Calculate center and scale
        let center_x = braille.dot_width() / 2;
        let center_y = braille.dot_height() / 2;
        let scale = self.amplitude * 0.5 + 0.5; // 0.5-1.0 range
        
        // Get circle positions
        let positions = self.calculate_circle_positions();
        
        // Draw each circle
        for (i, (x, y)) in positions.iter().enumerate() {
            // Apply rotation
            let angle = self.rotation;
            let rotated_x = x * angle.cos() - y * angle.sin();
            let rotated_y = x * angle.sin() + y * angle.cos();
            
            // Apply pulse scale
            let scaled_x = rotated_x * self.pulse_scale * scale;
            let scaled_y = rotated_y * self.pulse_scale * scale;
            
            // Calculate screen position
            let screen_x = center_x as f32 + scaled_x;
            let screen_y = center_y as f32 + scaled_y;
            
            // Calculate radius with pulse
            let radius = self.config.base_radius * self.pulse_scale * scale;
            
            // Calculate color based on position and treble
            let color_intensity = (i as f32 / positions.len() as f32 + self.treble) % 1.0;
            let color = if self.config.use_color {
                self.color_scheme.get_color(color_intensity)
                    .unwrap_or(Color::new(255, 255, 255))
            } else {
                Color::new(255, 255, 255)
            };
            
            // Draw circle with beat flash boost
            let brightness = 1.0 + self.beat_flash * 0.5;
            let final_color = Color::new(
                (color.r as f32 * brightness).min(255.0) as u8,
                (color.g as f32 * brightness).min(255.0) as u8,
                (color.b as f32 * brightness).min(255.0) as u8,
            );
            
            braille.draw_circle_with_color(
                screen_x as usize,
                screen_y as usize,
                radius as usize,
                final_color,
            );
        }
        
        // Transfer braille to grid
        braille.transfer_to_grid(grid);
    }
}
```

### 3. Configuration

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowerOfLifeConfig {
    /// Number of rings (1-5)
    #[serde(default = "default_num_rings")]
    pub num_rings: usize,
    
    /// Base circle radius in dots (10-50)
    #[serde(default = "default_base_radius")]
    pub base_radius: f32,
    
    /// Rotation speed multiplier (0.0-2.0)
    #[serde(default = "default_rotation_speed")]
    pub rotation_speed: f32,
    
    /// Pulse intensity (0.0-1.0)
    #[serde(default = "default_pulse_intensity")]
    pub pulse_intensity: f32,
    
    /// Enable color
    #[serde(default = "default_use_color")]
    pub use_color: bool,
}

fn default_num_rings() -> usize { 2 }
fn default_base_radius() -> f32 { 20.0 }
fn default_rotation_speed() -> f32 { 1.0 }
fn default_pulse_intensity() -> f32 { 0.5 }
fn default_use_color() -> bool { true }
```

### 4. File Structure

```
src/visualization/
├── mod.rs                    # Add flower_of_life module
├── flower_of_life.rs         # FlowerOfLifeVisualizer implementation
└── ...
```

### 5. BrailleGrid Circle Drawing

Need to add `draw_circle_with_color` method to `BrailleGrid`:

```rust
impl BrailleGrid {
    /// Draw a circle outline using Bresenham's circle algorithm
    pub fn draw_circle_with_color(&mut self, cx: usize, cy: usize, radius: usize, color: Color) {
        // Bresenham's circle algorithm
        let mut x = 0;
        let mut y = radius as i32;
        let mut d = 3 - 2 * radius as i32;
        
        while x <= y {
            self.set_dot_with_color_safe(cx, cy, x, y, color);
            self.set_dot_with_color_safe(cx, cy, -x, y, color);
            self.set_dot_with_color_safe(cx, cy, x, -y, color);
            self.set_dot_with_color_safe(cx, cy, -x, -y, color);
            self.set_dot_with_color_safe(cx, cy, y, x, color);
            self.set_dot_with_color_safe(cx, cy, -y, x, color);
            self.set_dot_with_color_safe(cx, cy, y, -x, color);
            self.set_dot_with_color_safe(cx, cy, -y, -x, color);
            
            if d < 0 {
                d += 4 * x + 6;
            } else {
                d += 4 * (x - y) + 10;
                y -= 1;
            }
            x += 1;
        }
    }
}
```

## Dependencies

**Depends on**: 
- VIZ-001 (GridBuffer) ✅ Complete
- VIZ-003 (Coverage Algorithm) ✅ Complete
- VIZ-004 (Visualizer trait) ✅ Complete
- Braille rendering system ✅ Complete

**Blocks**: 
- VIZ-016 (Recursive Pattern System) - can use Flower of Life as base pattern

## Testing Requirements

### Unit Tests
- [ ] Circle position calculation for different ring counts
- [ ] Hexagonal pattern geometry validation
- [ ] Audio parameter smoothing
- [ ] Rotation math (verify circles stay in pattern)
- [ ] Pulse scale calculation

### Integration Tests
- [ ] Render with default config
- [ ] Render with different ring counts (1-5)
- [ ] Audio reactivity (bass, mid, treble, beat)
- [ ] Color scheme integration
- [ ] Performance test (60 FPS with 3 rings)

### Visual Tests
- [ ] Static render looks like Flower of Life
- [ ] Circles overlap correctly
- [ ] Rotation is smooth
- [ ] Pulse effect is visible
- [ ] Beat flash is noticeable
- [ ] Colors cycle smoothly

## Implementation Notes

### Sacred Geometry Accuracy
- Use proper hexagonal spacing (60° intervals)
- Ensure circles overlap at exactly 1 radius distance
- Maintain symmetry during rotation and scaling

### Performance Considerations
- Pre-calculate circle positions when config changes
- Use Bresenham's algorithm for efficient circle drawing
- Limit to 5 rings maximum (91 circles) for performance
- Consider caching circle paths if performance is an issue

### Audio Mapping Philosophy
- **Bass** → Physical expansion (makes pattern breathe)
- **Mid** → Rotation (creates hypnotic spinning)
- **Treble** → Color cycling (adds visual interest)
- **Beat** → Flash (emphasizes rhythm)
- **Amplitude** → Overall scale (responds to volume)

## Success Metrics

- ✅ Recognizable as Flower of Life pattern
- ✅ Smooth 60 FPS animation
- ✅ Audio reactivity is clear and pleasing
- ✅ Works with all color schemes
- ✅ Configurable via YAML
- ✅ No visual artifacts or glitches
- ✅ Meditative, mesmerizing quality

## Future Enhancements (Post-MVP)

- Seed of Life variant (7 circles only, different arrangement)
- Filled circles vs outline circles option
- Individual circle pulse (each circle pulses independently)
- 3D perspective (circles at different depths)
- Metatron's Cube overlay (connect circle centers)
- Integration with VIZ-016 for recursive nested patterns

