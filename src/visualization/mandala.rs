// Mandala Generator
// Sacred geometry with radial symmetry and layered patterns

use super::{BrailleGrid, Color, GridBuffer, Visualizer};
use crate::dsp::AudioParameters;
use crate::visualization::color_schemes::{ColorScheme, ColorSchemeType};
use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;

/// Configuration for Mandala visualizer
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

    /// Enable color
    #[serde(default = "default_use_color")]
    pub use_color: bool,
}

fn default_symmetry() -> usize {
    8
}
fn default_num_layers() -> usize {
    3
}
fn default_base_radius() -> f32 {
    15.0
}
fn default_rotation_speed() -> f32 {
    1.0
}
fn default_pulse_intensity() -> f32 {
    0.5
}
fn default_use_color() -> bool {
    true
}

impl Default for MandalaConfig {
    fn default() -> Self {
        Self {
            symmetry: default_symmetry(),
            num_layers: default_num_layers(),
            base_radius: default_base_radius(),
            rotation_speed: default_rotation_speed(),
            pulse_intensity: default_pulse_intensity(),
            use_color: default_use_color(),
        }
    }
}

/// Mandala visualizer - radial symmetry with layered patterns
pub struct MandalaVisualizer {
    config: MandalaConfig,
    color_scheme: ColorScheme,

    // Animation state
    layer_rotations: Vec<f32>,
    pulse_scale: f32,
    beat_flash: f32,
    pattern_phase: f32,

    // Smoothed audio parameters
    amplitude: f32,
    bass: f32,
    mid: f32,
    treble: f32,
}

impl MandalaVisualizer {
    /// Create a new Mandala visualizer
    pub fn new(config: MandalaConfig) -> Self {
        let layer_rotations = vec![0.0; config.num_layers];

        Self {
            config,
            color_scheme: ColorScheme::new(ColorSchemeType::Rainbow),
            layer_rotations,
            pulse_scale: 1.0,
            beat_flash: 0.0,
            pattern_phase: 0.0,
            amplitude: 0.0,
            bass: 0.0,
            mid: 0.0,
            treble: 0.0,
        }
    }

    /// Set the color scheme
    pub fn set_color_scheme(&mut self, scheme: ColorScheme) {
        self.color_scheme = scheme;
    }

    /// Update config and resize layer rotations if needed
    pub fn update_config(&mut self, config: MandalaConfig) {
        if config.num_layers != self.config.num_layers {
            self.layer_rotations.resize(config.num_layers, 0.0);
        }
        self.config = config;
    }

    /// Draw a line with radial symmetry
    fn draw_radial_lines(
        &self,
        braille: &mut BrailleGrid,
        center_x: f32,
        center_y: f32,
        length: f32,
        layer_rotation: f32,
        color: Color,
    ) {
        let angle_step = TAU / self.config.symmetry as f32;

        for i in 0..self.config.symmetry {
            let angle = i as f32 * angle_step + layer_rotation;
            let x1 = center_x;
            let y1 = center_y;
            let x2 = x1 + length * angle.cos();
            let y2 = y1 + length * angle.sin();

            // Non-AA integer line drawing
            let dot_w = braille.dot_width() as f32;
            let dot_h = braille.dot_height() as f32;
            let xi1 = x1.round().clamp(0.0, dot_w - 1.0) as usize;
            let yi1 = y1.round().clamp(0.0, dot_h - 1.0) as usize;
            let xi2 = x2.round().clamp(0.0, dot_w - 1.0) as usize;
            let yi2 = y2.round().clamp(0.0, dot_h - 1.0) as usize;
            braille.draw_line_with_color(xi1, yi1, xi2, yi2, color);
        }
    }

    /// Draw circles with radial symmetry
    fn draw_radial_circles(
        &self,
        braille: &mut BrailleGrid,
        center_x: f32,
        center_y: f32,
        orbit_radius: f32,
        circle_radius: f32,
        layer_rotation: f32,
        color: Color,
    ) {
        let angle_step = TAU / self.config.symmetry as f32;

        for i in 0..self.config.symmetry {
            let angle = i as f32 * angle_step + layer_rotation;
            let x = center_x + orbit_radius * angle.cos();
            let y = center_y + orbit_radius * angle.sin();

            // Non-AA integer circle drawing
            let cx_i = x.round().clamp(0.0, braille.dot_width() as f32 - 1.0) as usize;
            let cy_i = y.round().clamp(0.0, braille.dot_height() as f32 - 1.0) as usize;
            let r_i = circle_radius.max(1.0).round() as usize;
            braille.draw_circle(cx_i, cy_i, r_i, color);
        }
    }

    /// Draw petal shapes with radial symmetry
    fn draw_radial_petals(
        &self,
        braille: &mut BrailleGrid,
        center_x: f32,
        center_y: f32,
        length: f32,
        width: f32,
        layer_rotation: f32,
        color: Color,
    ) {
        let angle_step = TAU / self.config.symmetry as f32;

        for i in 0..self.config.symmetry {
            let angle = i as f32 * angle_step + layer_rotation;

            // Draw petal as two curved lines
            let tip_x = center_x + length * angle.cos();
            let tip_y = center_y + length * angle.sin();

            // Left curve
            let left_angle = angle - width / length;
            let left_x = center_x + (length * 0.7) * left_angle.cos();
            let left_y = center_y + (length * 0.7) * left_angle.sin();
            // Non-AA integer line drawing for petals (left)
            let dot_w = braille.dot_width() as f32;
            let dot_h = braille.dot_height() as f32;
            let cxi = center_x.round().clamp(0.0, dot_w - 1.0) as usize;
            let cyi = center_y.round().clamp(0.0, dot_h - 1.0) as usize;
            let lxi = left_x.round().clamp(0.0, dot_w - 1.0) as usize;
            let lyi = left_y.round().clamp(0.0, dot_h - 1.0) as usize;
            let txi = tip_x.round().clamp(0.0, dot_w - 1.0) as usize;
            let tyi = tip_y.round().clamp(0.0, dot_h - 1.0) as usize;
            braille.draw_line_with_color(cxi, cyi, lxi, lyi, color);
            braille.draw_line_with_color(lxi, lyi, txi, tyi, color);

            // Right curve
            let right_angle = angle + width / length;
            let right_x = center_x + (length * 0.7) * right_angle.cos();
            let right_y = center_y + (length * 0.7) * right_angle.sin();
            // Non-AA integer line drawing for petals (right)
            let rxi = right_x.round().clamp(0.0, dot_w - 1.0) as usize;
            let ryi = right_y.round().clamp(0.0, dot_h - 1.0) as usize;
            braille.draw_line_with_color(cxi, cyi, rxi, ryi, color);
            braille.draw_line_with_color(rxi, ryi, txi, tyi, color);
        }
    }
}

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

        // Update pattern phase based on treble
        self.pattern_phase += self.treble * 0.01;

        // Update beat flash
        if params.beat {
            self.beat_flash = 1.0;
        } else {
            self.beat_flash *= 0.85; // Decay
        }
    }

    fn render(&self, grid: &mut GridBuffer) {
        grid.clear();

        let width = grid.width();
        let height = grid.height();
        let mut braille = BrailleGrid::new(width, height);


        let center_x = braille.dot_width() as f32 / 2.0;
        let center_y = braille.dot_height() as f32 / 2.0;
        let scale = (self.amplitude * 0.5 + 0.5) * self.pulse_scale;

        // Render each layer
        for layer_idx in 0..self.config.num_layers {
            let layer_rotation = self.layer_rotations[layer_idx];
            let layer_radius = self.config.base_radius + (layer_idx as f32 * 12.0);
            let scaled_radius = layer_radius * scale;

            // Color based on layer and audio
            let color_intensity =
                (layer_idx as f32 / self.config.num_layers as f32 + self.pattern_phase) % 1.0;
            let mut color = if self.config.use_color {
                self.color_scheme
                    .get_color(color_intensity)
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

            // Draw different patterns for each layer
            match layer_idx % 3 {
                0 => {
                    // Radial lines
                    self.draw_radial_lines(
                        &mut braille,
                        center_x,
                        center_y,
                        scaled_radius,
                        layer_rotation,
                        color,
                    );
                }
                1 => {
                    // Circles at symmetry points
                    let circle_radius = 5.0 * scale;
                    self.draw_radial_circles(
                        &mut braille,
                        center_x,
                        center_y,
                        scaled_radius,
                        circle_radius,
                        layer_rotation,
                        color,
                    );
                }
                2 => {
                    // Petal shapes
                    let petal_length = scaled_radius * 0.8;
                    let petal_width = 0.3;
                    self.draw_radial_petals(
                        &mut braille,
                        center_x,
                        center_y,
                        petal_length,
                        petal_width,
                        layer_rotation,
                        color,
                    );
                }
                _ => unreachable!(),
            }
        }

        // Transfer braille to grid
        for cell_y in 0..height {
            for cell_x in 0..width {
                let ch = braille.get_char(cell_x, cell_y);
                if ch != '⠀' {
                    if let Some(color) = braille.get_color(cell_x, cell_y) {
                        grid.set_cell_with_color(cell_x, cell_y, ch, color);
                    } else {
                        grid.set_cell(cell_x, cell_y, ch);
                    }
                }
            }
        }
    }

    fn name(&self) -> &str {
        "Mandala"
    }
}

/// Linear interpolation helper
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visualizer_creation() {
        let config = MandalaConfig::default();
        let viz = MandalaVisualizer::new(config);
        assert_eq!(viz.name(), "Mandala");
        assert_eq!(viz.layer_rotations.len(), 3);
    }

    #[test]
    fn test_config_update() {
        let config = MandalaConfig::default();
        let mut viz = MandalaVisualizer::new(config);

        let new_config = MandalaConfig {
            num_layers: 5,
            ..Default::default()
        };
        viz.update_config(new_config);

        assert_eq!(viz.layer_rotations.len(), 5);
    }

    #[test]
    fn test_audio_update() {
        let config = MandalaConfig::default();
        let mut viz = MandalaVisualizer::new(config);

        let params = AudioParameters {
            amplitude: 0.8,
            bass: 0.6,
            mid: 0.5,
            treble: 0.4,
            beat: true,
            beat_flux: false,
            bpm: 120.0,
            tempo_confidence: 0.8,
            spectrum: vec![],
            waveform: vec![],
            waveform_left: vec![],
            waveform_right: vec![],
        };

        viz.update(&params);

        assert!(viz.amplitude > 0.0);
        assert!(viz.bass > 0.0);
        assert_eq!(viz.beat_flash, 1.0);
    }
}

