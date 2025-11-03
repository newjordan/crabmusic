// 3D Waveform Tunnel Visualizer
// Captures waveform snapshots and moves them toward camera with perspective scaling

use super::{lerp, GridBuffer, Visualizer};
use crate::dsp::AudioParameters;
use crate::visualization::color_schemes::ColorScheme;
use std::collections::VecDeque;
use std::f32::consts::PI;

/// A snapshot of the waveform at a moment in time
#[derive(Debug, Clone)]
struct WaveformSnapshot {
    /// Amplitude at capture time
    amplitude: f32,
    /// Frequency at capture time
    frequency: f32,
    /// Phase at capture time
    phase: f32,
    /// Depth in tunnel (0.0 = far away, 1.0 = at camera)
    depth: f32,
    /// Bass content at capture time
    bass: f32,
    /// Mid content at capture time
    mid: f32,
    /// Treble content at capture time
    treble: f32,
}

/// 3D Waveform Tunnel Visualizer
///
/// Creates a tunnel effect by capturing waveform snapshots and moving them
/// toward the camera with perspective scaling. Far layers appear small at the
/// top, near layers appear large at the bottom, creating depth perception.
pub struct WaveformTunnelVisualizer {
    /// Color scheme for rendering
    color_scheme: ColorScheme,
    /// Queue of waveform snapshots (layers in tunnel)
    layers: VecDeque<WaveformSnapshot>,
    /// Maximum number of layers in tunnel
    max_layers: usize,
    /// Speed at which layers move toward camera (depth units per frame)
    speed: f32,
    /// Current phase for waveform generation
    phase: f32,
    /// Current amplitude (smoothed)
    amplitude: f32,
    /// Current frequency (smoothed)
    frequency: f32,
    /// Current bass (smoothed)
    bass: f32,
    /// Current mid (smoothed)
    mid: f32,
    /// Current treble (smoothed)
    treble: f32,
    /// Base frequency (cycles across width)
    base_frequency: f32,
    /// Amplitude sensitivity multiplier
    amplitude_sensitivity: f32,
    /// Frequency sensitivity multiplier
    frequency_sensitivity: f32,
    /// Smoothing factor (0.0-1.0)
    smoothing_factor: f32,
    /// Phase speed (radians per frame)
    phase_speed: f32,
}

impl WaveformTunnelVisualizer {
    /// Create a new waveform tunnel visualizer
    ///
    /// # Arguments
    /// * `color_scheme` - Color scheme for rendering
    ///
    /// # Returns
    /// A new WaveformTunnelVisualizer instance
    pub fn new(color_scheme: ColorScheme) -> Self {
        Self {
            color_scheme,
            layers: VecDeque::new(),
            max_layers: 25,              // 25 layers for good depth
            speed: 0.025,                // 2.5% depth per frame (40 frames to traverse)
            phase: 0.0,
            amplitude: 0.0,
            frequency: 2.0,
            bass: 0.0,
            mid: 0.0,
            treble: 0.0,
            base_frequency: 2.0,         // 2 cycles across width
            amplitude_sensitivity: 8.0,  // Sensitive to amplitude changes
            frequency_sensitivity: 3.0,  // Moderate frequency sensitivity
            smoothing_factor: 0.3,       // Fairly responsive
            phase_speed: 0.1,            // Moderate animation speed
        }
    }

    /// Update the color scheme for rendering
    pub fn set_color_scheme(&mut self, color_scheme: ColorScheme) {
        self.color_scheme = color_scheme;
    }

    /// Get the current color scheme
    pub fn color_scheme(&self) -> &ColorScheme {
        &self.color_scheme
    }

    /// Calculate waveform sample at given x position for a snapshot
    fn calculate_wave_sample(&self, x: f32, width: f32, snapshot: &WaveformSnapshot) -> f32 {
        let t = x / width;
        let angle = t * 2.0 * PI * snapshot.frequency + snapshot.phase;
        angle.sin() * snapshot.amplitude
    }

    /// Render a single layer with perspective scaling
    fn render_layer(&self, grid: &mut GridBuffer, snapshot: &WaveformSnapshot) {
        let width = grid.width();
        let height = grid.height();

        // Calculate scale based on depth (0.2 to 1.0)
        let scale = 0.15 + (snapshot.depth * 0.85);

        // Calculate y center based on depth (top to bottom)
        let y_center = (height as f32 * snapshot.depth) as usize;

        // Calculate color intensity based on depth (far = dim, near = bright)
        let intensity = 0.2 + (snapshot.depth * 0.8);

        // Modulate intensity with frequency content
        let freq_intensity = snapshot.bass * 0.3 + snapshot.mid * 0.4 + snapshot.treble * 0.3;
        let final_intensity = (intensity * (0.5 + freq_intensity * 0.5)).min(1.0);

        // Render waveform across width
        for x in 0..width {
            let sample = self.calculate_wave_sample(x as f32, width as f32, snapshot);
            let y_offset = (sample * scale * height as f32 * 0.4) as i32;
            let y = (y_center as i32 + y_offset).clamp(0, height as i32 - 1) as usize;

            // Get color from color scheme
            let color = self.color_scheme.get_color(final_intensity);

            // Use Braille characters for smooth curves
            let cell = grid.get_cell_mut(x, y);
            cell.character = '⠿'; // Full Braille block for solid appearance
            cell.foreground_color = color;

            // Add thickness by drawing adjacent pixels
            if scale > 0.5 {
                // Thicker lines for closer layers
                if y > 0 {
                    let cell_above = grid.get_cell_mut(x, y - 1);
                    if cell_above.character == ' ' {
                        cell_above.character = '⠿';
                        cell_above.foreground_color = color;
                    }
                }
                if y < height - 1 {
                    let cell_below = grid.get_cell_mut(x, y + 1);
                    if cell_below.character == ' ' {
                        cell_below.character = '⠿';
                        cell_below.foreground_color = color;
                    }
                }
            }
        }
    }
}

impl Visualizer for WaveformTunnelVisualizer {
    fn update(&mut self, params: &AudioParameters) {
        // Apply smoothing to audio parameters
        let smoothing = self.smoothing_factor;

        self.amplitude = lerp(
            self.amplitude,
            params.amplitude * self.amplitude_sensitivity,
            smoothing,
        );

        self.frequency = lerp(
            self.frequency,
            self.base_frequency + params.mid * self.frequency_sensitivity,
            smoothing,
        );

        self.bass = lerp(self.bass, params.bass, smoothing);
        self.mid = lerp(self.mid, params.mid, smoothing);
        self.treble = lerp(self.treble, params.treble, smoothing);

        // Update phase for animation
        self.phase += self.phase_speed;
        if self.phase > 2.0 * PI {
            self.phase -= 2.0 * PI;
        }

        // Capture new snapshot
        let snapshot = WaveformSnapshot {
            amplitude: self.amplitude,
            frequency: self.frequency,
            phase: self.phase,
            depth: 0.0, // Start at far distance
            bass: self.bass,
            mid: self.mid,
            treble: self.treble,
        };

        // Add to front of queue
        self.layers.push_front(snapshot);

        // Move all layers toward camera
        for layer in &mut self.layers {
            layer.depth += self.speed;
        }

        // Remove layers that have reached camera
        while let Some(layer) = self.layers.back() {
            if layer.depth >= 1.0 {
                self.layers.pop_back();
            } else {
                break;
            }
        }

        // Limit to max layers
        while self.layers.len() > self.max_layers {
            self.layers.pop_back();
        }
    }

    fn render(&self, grid: &mut GridBuffer) {
        // Clear grid
        grid.clear();

        // Render layers from back to front (far to near)
        // This ensures near layers overdraw far layers
        for layer in self.layers.iter().rev() {
            self.render_layer(grid, layer);
        }
    }

    fn name(&self) -> &str {
        "Waveform Tunnel"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visualization::color_schemes::ColorSchemeType;

    #[test]
    fn test_tunnel_new() {
        let color_scheme = ColorScheme::new(ColorSchemeType::Monochrome);
        let visualizer = WaveformTunnelVisualizer::new(color_scheme);
        assert_eq!(visualizer.name(), "Waveform Tunnel");
        assert_eq!(visualizer.layers.len(), 0);
        assert_eq!(visualizer.max_layers, 25);
    }

    #[test]
    fn test_snapshot_capture() {
        let color_scheme = ColorScheme::new(ColorSchemeType::Monochrome);
        let mut visualizer = WaveformTunnelVisualizer::new(color_scheme);
        let params = AudioParameters::default();

        visualizer.update(&params);
        assert_eq!(visualizer.layers.len(), 1);
        // First snapshot starts at 0.0 but immediately moves by speed
        assert_eq!(visualizer.layers[0].depth, visualizer.speed);
    }

    #[test]
    fn test_layer_movement() {
        let color_scheme = ColorScheme::new(ColorSchemeType::Monochrome);
        let mut visualizer = WaveformTunnelVisualizer::new(color_scheme);
        let params = AudioParameters::default();

        visualizer.update(&params);
        let initial_depth = visualizer.layers[0].depth;

        visualizer.update(&params);
        let second_depth = visualizer.layers[1].depth;

        assert!(second_depth > initial_depth);
    }

    #[test]
    fn test_layer_removal() {
        let color_scheme = ColorScheme::new(ColorSchemeType::Monochrome);
        let mut visualizer = WaveformTunnelVisualizer::new(color_scheme);
        visualizer.speed = 0.5; // Fast speed for testing
        let params = AudioParameters::default();

        // Add layers until one reaches camera
        for _ in 0..5 {
            visualizer.update(&params);
        }

        // Verify layers at depth >= 1.0 are removed
        for layer in &visualizer.layers {
            assert!(layer.depth < 1.0);
        }
    }

    #[test]
    fn test_max_layers_limit() {
        let color_scheme = ColorScheme::new(ColorSchemeType::Monochrome);
        let mut visualizer = WaveformTunnelVisualizer::new(color_scheme);
        visualizer.max_layers = 10;
        visualizer.speed = 0.01; // Slow speed so layers don't reach camera
        let params = AudioParameters::default();

        // Add more than max_layers
        for _ in 0..20 {
            visualizer.update(&params);
        }

        // Should be limited to max_layers
        assert_eq!(visualizer.layers.len(), 10);
    }
}

