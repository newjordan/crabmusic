// 3D Waveform Tunnel Visualizer
// Captures waveform snapshots and moves them toward camera with perspective scaling

use super::{lerp, BrailleGrid, GridBuffer, Visualizer};
use crate::dsp::AudioParameters;
use crate::visualization::color_schemes::ColorScheme;
use std::collections::VecDeque;
use std::f32::consts::PI;

/// A snapshot of the waveform at a moment in time
/// This is a FROZEN capture - the waveform samples never change after capture
#[derive(Debug, Clone)]
struct WaveformSnapshot {
    /// Pre-calculated waveform samples (frozen at capture time)
    samples: Vec<f32>,
    /// Y position on screen (in rows)
    y_position: usize,
    /// Color intensity for this snapshot
    intensity: f32,
}

/// 3D Waveform Tunnel Visualizer
///
/// Creates a scrolling cascade of frozen waveform snapshots.
/// New waves are drawn at the bottom, and the entire canvas scrolls upward.
pub struct WaveformTunnelVisualizer {
    /// Color scheme for rendering
    color_scheme: ColorScheme,
    /// Queue of waveform snapshots (scrolling upward)
    layers: VecDeque<WaveformSnapshot>,
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
    /// Rows to scroll per frame
    scroll_speed: usize,
    /// Frame counter for scroll timing
    frame_counter: usize,
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
            phase: 0.0,
            amplitude: 1.0,              // Start with full amplitude
            frequency: 3.0,              // Start with 3 cycles
            bass: 0.5,                   // Start with some bass
            mid: 0.5,                    // Start with some mid
            treble: 0.5,                 // Start with some treble
            base_frequency: 3.0,         // 3 cycles across width for more visible waves
            amplitude_sensitivity: 12.0, // Sensitive to amplitude changes
            frequency_sensitivity: 4.0,  // Moderate frequency sensitivity
            smoothing_factor: 0.25,      // Balanced responsiveness
            phase_speed: 0.15,           // Faster animation speed
            scroll_speed: 1,             // Scroll 1 row per frame
            frame_counter: 0,
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

    /// Calculate waveform samples for current state (to be frozen in snapshot)
    fn calculate_waveform_samples(&self, num_samples: usize) -> Vec<f32> {
        let mut samples = Vec::with_capacity(num_samples);
        for i in 0..num_samples {
            let t = i as f32 / num_samples as f32;
            let angle = t * 2.0 * PI * self.frequency + self.phase;
            let sample = angle.sin() * self.amplitude;
            samples.push(sample);
        }
        samples
    }

    /// Render a single FROZEN waveform line at a specific Y position
    fn render_layer(&self, braille: &mut BrailleGrid, snapshot: &WaveformSnapshot, grid_height: usize) {
        let dot_width = braille.dot_width();
        let dot_height = braille.dot_height();

        // Calculate the Y center in dot coordinates
        let cell_y = snapshot.y_position;
        if cell_y >= grid_height {
            return; // Off screen
        }

        let dot_center_y = cell_y * 4 + 2; // Convert cell Y to dot Y (middle of cell)

        // Get color from color scheme
        let color = match self.color_scheme.get_color(snapshot.intensity) {
            Some(c) => c,
            None => super::Color::new(128, 128, 128), // Fallback gray
        };

        // Draw thin sine wave line using FROZEN samples from snapshot
        let mut prev_x = 0;
        let mut prev_y = dot_center_y;

        let num_samples = snapshot.samples.len();
        for dot_x in 0..dot_width {
            // Get frozen sample from snapshot (interpolate if needed)
            let sample_index = (dot_x as f32 / dot_width as f32 * num_samples as f32) as usize;
            let sample_index = sample_index.min(num_samples - 1);
            let sample = snapshot.samples[sample_index];

            // Amplitude scale - fixed size for all waves
            let amplitude_scale = dot_height as f32 * 0.15; // 15% of height
            let y_offset = (sample * amplitude_scale) as i32;
            let dot_y = (dot_center_y as i32 + y_offset).clamp(0, dot_height as i32 - 1) as usize;

            // Draw line from previous point to current point (smooth anti-aliased)
            if dot_x > 0 {
                braille.draw_line_with_color(prev_x, prev_y, dot_x, dot_y, color);
            }

            prev_x = dot_x;
            prev_y = dot_y;
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

        // Increment frame counter
        self.frame_counter += 1;

        // Every frame, scroll all layers up
        if self.frame_counter % self.scroll_speed == 0 {
            for layer in &mut self.layers {
                if layer.y_position > 0 {
                    layer.y_position -= 1;
                }
            }

            // Remove layers that scrolled off the top
            self.layers.retain(|layer| layer.y_position > 0);

            // CAPTURE NEW FROZEN SNAPSHOT at the bottom
            let num_samples = 200; // High resolution for smooth curves
            let samples = self.calculate_waveform_samples(num_samples);

            // Calculate intensity from frequency content
            let freq_intensity = self.bass * 0.3 + self.mid * 0.4 + self.treble * 0.3;
            let intensity = (0.5 + freq_intensity * 0.5).min(1.0);

            let snapshot = WaveformSnapshot {
                samples, // Frozen waveform samples
                y_position: 100, // Start at bottom (will be adjusted in render based on actual height)
                intensity,
            };

            // Add to back of queue (bottom of screen)
            self.layers.push_back(snapshot);
        }
    }

    fn render(&self, grid: &mut GridBuffer) {
        // Clear grid
        grid.clear();

        let width = grid.width();
        let height = grid.height();

        // Create Braille grid for high-resolution rendering
        let mut braille = BrailleGrid::new(width, height);

        // Render all layers at their current Y positions
        for layer in &self.layers {
            self.render_layer(&mut braille, layer, height);
        }

        // Convert Braille grid back to regular grid
        for cell_y in 0..height {
            for cell_x in 0..width {
                let braille_char = braille.get_char(cell_x, cell_y);
                let braille_color = braille.get_color(cell_x, cell_y);

                if braille_char != ' ' {
                    let cell = grid.get_cell_mut(cell_x, cell_y);
                    cell.character = braille_char;
                    cell.foreground_color = braille_color;
                }
            }
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
        assert_eq!(visualizer.scroll_speed, 1);
    }

    #[test]
    fn test_snapshot_capture() {
        let color_scheme = ColorScheme::new(ColorSchemeType::Monochrome);
        let mut visualizer = WaveformTunnelVisualizer::new(color_scheme);
        let params = AudioParameters::default();

        visualizer.update(&params);
        assert_eq!(visualizer.layers.len(), 1);
        // Verify frozen samples were captured
        assert_eq!(visualizer.layers[0].samples.len(), 200);
    }

    #[test]
    fn test_layer_scrolling() {
        let color_scheme = ColorScheme::new(ColorSchemeType::Monochrome);
        let mut visualizer = WaveformTunnelVisualizer::new(color_scheme);
        let params = AudioParameters::default();

        visualizer.update(&params);
        let initial_y = visualizer.layers[0].y_position;

        visualizer.update(&params);
        let second_y = visualizer.layers[1].y_position;

        // Second layer should be below first (higher Y)
        assert!(second_y >= initial_y);
    }

    #[test]
    fn test_layer_removal() {
        let color_scheme = ColorScheme::new(ColorSchemeType::Monochrome);
        let mut visualizer = WaveformTunnelVisualizer::new(color_scheme);
        let params = AudioParameters::default();

        // Add many layers
        for _ in 0..150 {
            visualizer.update(&params);
        }

        // Verify layers at y_position 0 are removed
        for layer in &visualizer.layers {
            assert!(layer.y_position > 0);
        }
    }

    #[test]
    fn test_continuous_flow() {
        let color_scheme = ColorScheme::new(ColorSchemeType::Monochrome);
        let mut visualizer = WaveformTunnelVisualizer::new(color_scheme);
        let params = AudioParameters::default();

        // Add several layers
        for _ in 0..10 {
            visualizer.update(&params);
        }

        // Should have multiple layers
        assert!(visualizer.layers.len() > 0);
    }
}

