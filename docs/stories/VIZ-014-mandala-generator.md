# VIZ-014: Mandala Generator

**Epic**: Sacred Geometry
**Priority**: P1 (High - User excitement area)
**Estimated Effort**: 5-7 days
**Status**: ✅ Complete

## Description

Implement a procedural **Mandala Generator** that creates audio-reactive mandala patterns with radial symmetry. Mandalas are spiritual and ritual symbols in Hinduism and Buddhism, representing the universe. They feature intricate geometric patterns with rotational symmetry.

This visualizer will generate mandalas with configurable symmetry (4-fold, 6-fold, 8-fold, 12-fold) and make them audio-reactive by:
- Rotating layers at different speeds based on frequency bands
- Pulsing pattern complexity based on amplitude
- Expanding/contracting rings based on bass
- Color-cycling through the pattern based on audio spectrum
- Beat-synchronized pattern changes

The mandala will be built from layers of geometric primitives (circles, lines, arcs) arranged with radial symmetry, creating mesmerizing, kaleidoscopic patterns.

## Acceptance Criteria

- [ ] `MandalaVisualizer` struct implements `Visualizer` trait
- [ ] Renders mandala with configurable radial symmetry (4, 6, 8, 12-fold)
- [ ] Uses `BrailleGrid` for smooth, high-resolution rendering
- [ ] Multiple pattern layers that can rotate independently
- [ ] Audio-reactive features:
  - [ ] Bass drives ring expansion/contraction
  - [ ] Mid frequencies control layer rotation speeds
  - [ ] Treble affects pattern complexity/detail
  - [ ] Beat detection triggers pattern evolution
  - [ ] Amplitude affects overall scale
  - [ ] Frequency bands map to different layers
- [ ] Configuration struct `MandalaConfig` with parameters:
  - [ ] `symmetry` (4, 6, 8, 12) - rotational symmetry order
  - [ ] `num_layers` (1-5) - number of pattern layers
  - [ ] `base_radius` (10-50) - innermost ring radius
  - [ ] `rotation_speed` (0.0-2.0) - rotation multiplier
  - [ ] `pulse_intensity` (0.0-1.0) - how much bass affects size
  - [ ] `complexity` (0.0-1.0) - pattern detail level
  - [ ] `use_color` (bool) - enable color schemes
- [ ] Pattern templates (at least 3 different mandala styles)
- [ ] Smooth animation (60 FPS capable)
- [ ] Integration with existing color schemes
- [ ] Keyboard control to cycle through symmetry modes
- [ ] Unit tests for symmetry math and pattern generation
- [ ] Integration test with audio parameters
- [ ] Documentation with examples

## Technical Approach

### 1. Mandala Structure

A mandala consists of concentric layers, each with radial symmetry:

```rust
pub struct MandalaVisualizer {
    config: MandalaConfig,
    color_scheme: ColorScheme,
    
    // Animation state
    layer_rotations: Vec<f32>,  // Rotation angle for each layer
    pulse_scale: f32,            // Current pulse scale (1.0 = normal)
    beat_flash: f32,             // Beat flash intensity (0.0-1.0)
    pattern_phase: f32,          // Pattern evolution phase
    
    // Smoothed audio parameters
    amplitude: f32,
    bass: f32,
    mid: f32,
    treble: f32,
    
    // Pattern templates
    current_template: MandalaTemplate,
}

#[derive(Debug, Clone)]
pub struct MandalaLayer {
    /// Radius of this layer (relative to center)
    radius: f32,
    /// Pattern elements in this layer
    elements: Vec<PatternElement>,
    /// Rotation speed multiplier
    rotation_speed: f32,
}

#[derive(Debug, Clone)]
pub enum PatternElement {
    Circle { radius: f32 },
    Line { length: f32 },
    Arc { radius: f32, start_angle: f32, end_angle: f32 },
    Petal { length: f32, width: f32 },
    Star { points: usize, inner_radius: f32, outer_radius: f32 },
}
```

### 2. Radial Symmetry

The key to mandalas is radial symmetry - repeating patterns around a center:

```rust
impl MandalaVisualizer {
    /// Draw a pattern element with radial symmetry
    fn draw_with_symmetry(
        &self,
        braille: &mut BrailleGrid,
        center_x: usize,
        center_y: usize,
        element: &PatternElement,
        layer_rotation: f32,
        color: Color,
    ) {
        let symmetry = self.config.symmetry;
        let angle_step = TAU / symmetry as f32;
        
        // Draw element at each symmetry position
        for i in 0..symmetry {
            let angle = i as f32 * angle_step + layer_rotation;
            
            match element {
                PatternElement::Circle { radius } => {
                    let x = center_x as f32 + radius * angle.cos();
                    let y = center_y as f32 + radius * angle.sin();
                    braille.draw_circle_with_color(x as usize, y as usize, *radius as usize, color);
                }
                PatternElement::Line { length } => {
                    let x1 = center_x as f32;
                    let y1 = center_y as f32;
                    let x2 = x1 + length * angle.cos();
                    let y2 = y1 + length * angle.sin();
                    braille.draw_line_with_color(x1 as usize, y1 as usize, x2 as usize, y2 as usize, color);
                }
                PatternElement::Arc { radius, start_angle, end_angle } => {
                    self.draw_arc(braille, center_x, center_y, *radius, 
                                 angle + start_angle, angle + end_angle, color);
                }
                PatternElement::Petal { length, width } => {
                    self.draw_petal(braille, center_x, center_y, *length, *width, angle, color);
                }
                PatternElement::Star { points, inner_radius, outer_radius } => {
                    self.draw_star(braille, center_x, center_y, *points, 
                                  *inner_radius, *outer_radius, angle, color);
                }
            }
        }
    }
}
```

### 3. Audio Reactivity

```rust
impl Visualizer for MandalaVisualizer {
    fn update(&mut self, params: &AudioParameters) {
        // Smooth audio parameters
        let smoothing = 0.15;
        self.amplitude = lerp(self.amplitude, params.amplitude, smoothing);
        self.bass = lerp(self.bass, params.bass, smoothing);
        self.mid = lerp(self.mid, params.mid, smoothing);
        self.treble = lerp(self.treble, params.treble, smoothing);
        
        // Update layer rotations (each layer rotates at different speed)
        for (i, rotation) in self.layer_rotations.iter_mut().enumerate() {
            let speed_multiplier = 1.0 + (i as f32 * 0.3);
            *rotation += self.mid * self.config.rotation_speed * 0.02 * speed_multiplier;
        }
        
        // Update pulse scale based on bass
        let target_scale = 1.0 + self.bass * self.config.pulse_intensity * 0.3;
        self.pulse_scale = lerp(self.pulse_scale, target_scale, 0.2);
        
        // Update pattern phase based on treble (affects complexity)
        self.pattern_phase += self.treble * 0.01;
        
        // Update beat flash
        if params.beat_detected {
            self.beat_flash = 1.0;
            // On beat, occasionally evolve pattern
            if self.pattern_phase > TAU {
                self.pattern_phase = 0.0;
                self.evolve_pattern();
            }
        } else {
            self.beat_flash *= 0.85; // Decay
        }
    }
    
    fn render(&self, grid: &mut GridBuffer) {
        let width = grid.width();
        let height = grid.height();
        let mut braille = BrailleGrid::new(width, height);
        
        let center_x = braille.dot_width() / 2;
        let center_y = braille.dot_height() / 2;
        let scale = (self.amplitude * 0.5 + 0.5) * self.pulse_scale;
        
        // Render each layer
        let layers = self.generate_layers();
        for (layer_idx, layer) in layers.iter().enumerate() {
            let layer_rotation = self.layer_rotations[layer_idx];
            let layer_radius = layer.radius * scale;
            
            // Color based on layer and audio
            let color_intensity = (layer_idx as f32 / layers.len() as f32 + self.pattern_phase) % 1.0;
            let mut color = if self.config.use_color {
                self.color_scheme.get_color(color_intensity)
                    .unwrap_or(Color::new(255, 255, 255))
            } else {
                Color::new(255, 255, 255)
            };
            
            // Apply beat flash
            if self.beat_flash > 0.0 {
                let boost = 1.0 + self.beat_flash * 0.5;
                color = Color::new(
                    (color.r as f32 * boost).min(255.0) as u8,
                    (color.g as f32 * boost).min(255.0) as u8,
                    (color.b as f32 * boost).min(255.0) as u8,
                );
            }
            
            // Draw each element in the layer with radial symmetry
            for element in &layer.elements {
                self.draw_with_symmetry(
                    &mut braille,
                    center_x,
                    center_y,
                    element,
                    layer_rotation,
                    color,
                );
            }
        }
        
        braille.transfer_to_grid(grid);
    }
}
```

### 4. Pattern Templates

Pre-defined mandala styles:

```rust
#[derive(Debug, Clone)]
pub enum MandalaTemplate {
    Lotus,      // Petal-based design
    Geometric,  // Lines and circles
    Star,       // Star patterns
    Floral,     // Organic curves
    Cosmic,     // Circles and arcs
}

impl MandalaVisualizer {
    fn generate_layers(&self) -> Vec<MandalaLayer> {
        match self.current_template {
            MandalaTemplate::Lotus => self.generate_lotus_layers(),
            MandalaTemplate::Geometric => self.generate_geometric_layers(),
            MandalaTemplate::Star => self.generate_star_layers(),
            MandalaTemplate::Floral => self.generate_floral_layers(),
            MandalaTemplate::Cosmic => self.generate_cosmic_layers(),
        }
    }
    
    fn generate_lotus_layers(&self) -> Vec<MandalaLayer> {
        let mut layers = Vec::new();
        let base_radius = self.config.base_radius;
        
        for i in 0..self.config.num_layers {
            let radius = base_radius + (i as f32 * 15.0);
            let petal_size = 10.0 + (i as f32 * 3.0);
            
            layers.push(MandalaLayer {
                radius,
                elements: vec![
                    PatternElement::Petal {
                        length: petal_size,
                        width: petal_size * 0.6,
                    }
                ],
                rotation_speed: 1.0 + (i as f32 * 0.2),
            });
        }
        
        layers
    }
    
    // Similar methods for other templates...
}
```

### 5. Configuration

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MandalaConfig {
    /// Rotational symmetry (4, 6, 8, 12)
    #[serde(default = "default_symmetry")]
    pub symmetry: usize,
    
    /// Number of layers (1-5)
    #[serde(default = "default_num_layers")]
    pub num_layers: usize,
    
    /// Base radius in dots (10-50)
    #[serde(default = "default_base_radius")]
    pub base_radius: f32,
    
    /// Rotation speed multiplier (0.0-2.0)
    #[serde(default = "default_rotation_speed")]
    pub rotation_speed: f32,
    
    /// Pulse intensity (0.0-1.0)
    #[serde(default = "default_pulse_intensity")]
    pub pulse_intensity: f32,
    
    /// Pattern complexity (0.0-1.0)
    #[serde(default = "default_complexity")]
    pub complexity: f32,
    
    /// Enable color
    #[serde(default = "default_use_color")]
    pub use_color: bool,
    
    /// Template name
    #[serde(default = "default_template")]
    pub template: String,
}

fn default_symmetry() -> usize { 8 }
fn default_num_layers() -> usize { 3 }
fn default_base_radius() -> f32 { 20.0 }
fn default_rotation_speed() -> f32 { 1.0 }
fn default_pulse_intensity() -> f32 { 0.5 }
fn default_complexity() -> f32 { 0.7 }
fn default_use_color() -> bool { true }
fn default_template() -> String { "lotus".to_string() }
```

## Dependencies

**Depends on**: 
- VIZ-001 (GridBuffer) ✅ Complete
- VIZ-003 (Coverage Algorithm) ✅ Complete
- VIZ-004 (Visualizer trait) ✅ Complete
- Braille rendering system ✅ Complete

**Blocks**: 
- VIZ-016 (Recursive Pattern System) - can use Mandala as base pattern

## Testing Requirements

### Unit Tests
- [ ] Radial symmetry math (4, 6, 8, 12-fold)
- [ ] Pattern element positioning
- [ ] Layer generation for each template
- [ ] Audio parameter smoothing
- [ ] Rotation calculations

### Integration Tests
- [ ] Render with each template
- [ ] Render with different symmetry orders
- [ ] Audio reactivity (bass, mid, treble, beat)
- [ ] Color scheme integration
- [ ] Performance test (60 FPS with 3 layers)

### Visual Tests
- [ ] Symmetry is perfect (no misalignment)
- [ ] Layers rotate smoothly at different speeds
- [ ] Pulse effect is visible
- [ ] Beat flash is noticeable
- [ ] Pattern evolution works
- [ ] Colors cycle smoothly

## Success Metrics

- ✅ Recognizable as mandala pattern
- ✅ Perfect radial symmetry
- ✅ Smooth 60 FPS animation
- ✅ Audio reactivity is clear and pleasing
- ✅ Multiple templates provide variety
- ✅ Works with all color schemes
- ✅ Configurable via YAML
- ✅ Meditative, hypnotic quality

## Future Enhancements

- More pattern templates (10+ styles)
- Custom pattern editor
- Fractal mandalas (recursive layers)
- 3D perspective effects
- Particle systems within mandala
- Integration with VIZ-016 for nested patterns

