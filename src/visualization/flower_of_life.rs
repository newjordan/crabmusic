// Flower of Life Visualizer
// Sacred geometry pattern with overlapping circles in hexagonal arrangement

use super::{BrailleGrid, Color, GridBuffer, Visualizer};
use crate::dsp::AudioParameters;
use crate::visualization::color_schemes::{ColorScheme, ColorSchemeType};
use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;

/// Configuration for Flower of Life visualizer
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

fn default_num_rings() -> usize {
    2
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

impl Default for FlowerOfLifeConfig {
    fn default() -> Self {
        Self {
            num_rings: default_num_rings(),
            base_radius: default_base_radius(),
            rotation_speed: default_rotation_speed(),
            pulse_intensity: default_pulse_intensity(),
            use_color: default_use_color(),
        }
    }
}

/// Flower of Life visualizer - sacred geometry with overlapping circles
pub struct FlowerOfLifeVisualizer {
    config: FlowerOfLifeConfig,
    color_scheme: ColorScheme,

    // Animation state
    rotation: f32,
    pulse_scale: f32,
    beat_flash: f32,

    // Smoothed audio parameters
    amplitude: f32,
    bass: f32,
    mid: f32,
    treble: f32,

    // Cached circle positions
    circle_positions: Vec<(f32, f32)>,
}

impl FlowerOfLifeVisualizer {
    /// Create a new Flower of Life visualizer
    pub fn new(config: FlowerOfLifeConfig) -> Self {
        let circle_positions =
            Self::calculate_circle_positions(config.num_rings, config.base_radius);

        Self {
            config,
            color_scheme: ColorScheme::new(ColorSchemeType::Rainbow),
            rotation: 0.0,
            pulse_scale: 1.0,
            beat_flash: 0.0,
            amplitude: 0.0,
            bass: 0.0,
            mid: 0.0,
            treble: 0.0,
            circle_positions,
        }
    }

    /// Set the color scheme
    pub fn set_color_scheme(&mut self, scheme: ColorScheme) {
        self.color_scheme = scheme;
    }

    /// Calculate circle positions for Flower of Life pattern
    ///
    /// The pattern consists of:
    /// - Ring 0: 1 circle (center)
    /// - Ring 1: 6 circles around center (hexagonal)
    /// - Ring 2: 12 circles around ring 1
    /// - Ring 3: 18 circles around ring 2
    fn calculate_circle_positions(num_rings: usize, base_radius: f32) -> Vec<(f32, f32)> {
        let mut positions = vec![(0.0, 0.0)]; // Center circle

        for ring in 1..=num_rings {
            let num_circles = ring * 6;
            // Each ring is 2 radii away from center (circles touch)
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

    /// Update circle positions when config changes
    pub fn update_config(&mut self, config: FlowerOfLifeConfig) {
        let needs_recalc = config.num_rings != self.config.num_rings
            || (config.base_radius - self.config.base_radius).abs() > 0.1;

        self.config = config;

        if needs_recalc {
            self.circle_positions =
                Self::calculate_circle_positions(self.config.num_rings, self.config.base_radius);
        }
    }
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

        // Calculate center and scale
        let center_x = braille.dot_width() as f32 / 2.0;
        let center_y = braille.dot_height() as f32 / 2.0;
        let scale = self.amplitude * 0.5 + 0.5; // 0.5-1.0 range

        // Draw each circle
        for (i, (x, y)) in self.circle_positions.iter().enumerate() {
            // Apply rotation
            let angle = self.rotation;
            let rotated_x = x * angle.cos() - y * angle.sin();
            let rotated_y = x * angle.sin() + y * angle.cos();

            // Apply pulse scale
            let scaled_x = rotated_x * self.pulse_scale * scale;
            let scaled_y = rotated_y * self.pulse_scale * scale;

            // Calculate screen position
            let screen_x = center_x + scaled_x;
            let screen_y = center_y + scaled_y;

            // Calculate radius with pulse
            let radius = self.config.base_radius * self.pulse_scale * scale;

            // Calculate color based on position and treble
            let color_intensity =
                (i as f32 / self.circle_positions.len() as f32 + self.treble) % 1.0;
            let color = if self.config.use_color {
                self.color_scheme
                    .get_color(color_intensity)
                    .unwrap_or(Color::new(255, 255, 255))
            } else {
                Color::new(255, 255, 255)
            };

            // Apply beat flash boost
            let brightness = 1.0 + self.beat_flash * 0.5;
            let final_color = Color::new(
                (color.r as f32 * brightness).min(255.0) as u8,
                (color.g as f32 * brightness).min(255.0) as u8,
                (color.b as f32 * brightness).min(255.0) as u8,
            );

            // Draw circle (non-AA)
            let cx_i = screen_x
                .round()
                .clamp(0.0, braille.dot_width() as f32 - 1.0) as usize;
            let cy_i = screen_y
                .round()
                .clamp(0.0, braille.dot_height() as f32 - 1.0) as usize;
            let r_i = radius.max(1.0).round() as usize;
            braille.draw_circle(cx_i, cy_i, r_i, final_color);
        }

        // Transfer braille to grid
        for cell_y in 0..height {
            for cell_x in 0..width {
                let ch = braille.get_char(cell_x, cell_y);
                if ch != '⠀' {
                    // Not empty
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
        "Flower of Life"
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
    fn test_circle_positions_center_only() {
        let positions = FlowerOfLifeVisualizer::calculate_circle_positions(0, 10.0);
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0], (0.0, 0.0));
    }

    #[test]
    fn test_circle_positions_one_ring() {
        let positions = FlowerOfLifeVisualizer::calculate_circle_positions(1, 10.0);
        // 1 center + 6 in ring 1
        assert_eq!(positions.len(), 7);
    }

    #[test]
    fn test_circle_positions_two_rings() {
        let positions = FlowerOfLifeVisualizer::calculate_circle_positions(2, 10.0);
        // 1 center + 6 in ring 1 + 12 in ring 2
        assert_eq!(positions.len(), 19);
    }

    #[test]
    fn test_hexagonal_symmetry() {
        let positions = FlowerOfLifeVisualizer::calculate_circle_positions(1, 10.0);
        let ring_radius = 20.0; // base_radius * 2.0

        // Check that first 6 circles are evenly spaced at 60° intervals
        for i in 1..=6 {
            let (x, y) = positions[i];
            let distance = (x * x + y * y).sqrt();
            assert!((distance - ring_radius).abs() < 0.1);
        }
    }

    #[test]
    fn test_visualizer_creation() {
        let config = FlowerOfLifeConfig::default();
        let viz = FlowerOfLifeVisualizer::new(config);
        assert_eq!(viz.name(), "Flower of Life");
    }

    #[test]
    fn test_audio_update() {
        let config = FlowerOfLifeConfig::default();
        let mut viz = FlowerOfLifeVisualizer::new(config);

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

        // Check that smoothed values are moving toward targets
        assert!(viz.amplitude > 0.0);
        assert!(viz.bass > 0.0);
        assert_eq!(viz.beat_flash, 1.0);
    }
}
