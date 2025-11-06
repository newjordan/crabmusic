// Sine wave visualizer
// MVP visualizer that renders an audio-reactive sine wave

use super::{character_sets::CharacterSet, lerp, GridBuffer, Visualizer};
use crate::dsp::AudioParameters;
use std::f32::consts::PI;

/// Configuration for sine wave visualizer
#[derive(Debug, Clone)]
pub struct SineWaveConfig {
    /// Sensitivity to amplitude changes (multiplier)
    pub amplitude_sensitivity: f32,
    /// Sensitivity to frequency changes (multiplier)
    pub frequency_sensitivity: f32,
    /// Sensitivity to thickness changes (multiplier)
    pub thickness_sensitivity: f32,
    /// Base number of wave cycles across screen
    pub base_frequency: f32,
    /// Smoothing factor (0.0-1.0, higher = smoother)
    pub smoothing_factor: f32,
    /// Phase increment per frame
    pub phase_speed: f32,
}

impl Default for SineWaveConfig {
    fn default() -> Self {
        Self {
            amplitude_sensitivity: 10.0, // Increased from 2.0 - much more sensitive
            frequency_sensitivity: 3.0,  // Increased from 2.0
            thickness_sensitivity: 8.0,  // Increased from 5.0 for thicker, more visible lines
            base_frequency: 2.0,
            smoothing_factor: 0.5, // Good balance between smooth and responsive
            phase_speed: 0.15,     // Slightly faster animation
        }
    }
}

/// Sine wave visualizer
///
/// Renders an audio-reactive sine wave that responds to bass, mid, and amplitude.
///
/// # Audio Parameter Mapping
/// - `amplitude` → wave amplitude (vertical height)
/// - `bass` → wave thickness
/// - `mid` → wave frequency/speed
///
/// # Examples
///
/// ```
/// use crabmusic::visualization::{SineWaveVisualizer, SineWaveConfig, Visualizer, GridBuffer};
/// use crabmusic::dsp::AudioParameters;
///
/// let mut viz = SineWaveVisualizer::new(SineWaveConfig::default());
/// let mut grid = GridBuffer::new(80, 24);
/// let params = AudioParameters::default();
///
/// viz.update(&params);
/// viz.render(&mut grid);
/// ```
pub struct SineWaveVisualizer {
    /// Current wave phase for animation
    phase: f32,
    /// Current amplitude (smoothed)
    amplitude: f32,
    /// Current frequency (smoothed)
    frequency: f32,
    /// Current line thickness (smoothed)
    thickness: f32,
    /// Configuration
    config: SineWaveConfig,
    /// Beat flash effect (0.0-1.0, decays over time)
    beat_flash: f32,
    /// Character set for rendering (smooth gradients)
    charset: CharacterSet,
    /// Color scheme for rendering
    color_scheme: super::color_schemes::ColorScheme,
    /// Bass frequency content (20-250 Hz)
    bass: f32,
    /// Mid frequency content (250-4000 Hz)
    mid: f32,
    /// Treble frequency content (4000-20000 Hz)
    treble: f32,
}

impl SineWaveVisualizer {
    /// Create a new sine wave visualizer
    ///
    /// # Arguments
    /// * `config` - Configuration for the visualizer
    ///
    /// # Returns
    /// A new SineWaveVisualizer instance
    ///
    /// # Examples
    ///
    /// ```
    /// use crabmusic::visualization::{SineWaveVisualizer, SineWaveConfig, Visualizer};
    ///
    /// let viz = SineWaveVisualizer::new(SineWaveConfig::default());
    /// assert_eq!(viz.name(), "Sine Wave");
    /// ```
    pub fn new(config: SineWaveConfig, charset: CharacterSet) -> Self {
        Self {
            phase: 0.0,
            amplitude: 0.0,
            frequency: config.base_frequency,
            thickness: 1.0,
            config,
            beat_flash: 0.0,
            charset,
            color_scheme: super::color_schemes::ColorScheme::new(
                super::color_schemes::ColorSchemeType::Monochrome,
            ),
            bass: 0.0,
            mid: 0.0,
            treble: 0.0,
        }
    }

    /// Update the character set for rendering
    ///
    /// Allows changing the character set at runtime for different visual styles
    pub fn set_charset(&mut self, charset: CharacterSet) {
        self.charset = charset;
    }

    /// Update the color scheme for rendering
    ///
    /// Allows changing the color scheme at runtime for different visual styles
    pub fn set_color_scheme(&mut self, color_scheme: super::color_schemes::ColorScheme) {
        self.color_scheme = color_scheme;
    }

    /// Calculate coverage for a grid cell
    ///
    /// Determines what percentage of the cell is covered by the sine wave.
    ///
    /// # Arguments
    /// * `x` - X coordinate (column)
    /// * `y` - Y coordinate (row)
    /// * `width` - Grid width
    /// * `height` - Grid height
    ///
    /// # Returns
    /// Coverage percentage (0.0-1.0)
    fn calculate_coverage(&self, x: usize, y: usize, width: usize, height: usize) -> f32 {
        // Normalize coordinates to 0.0-1.0
        let norm_x = x as f32 / width as f32;
        let norm_y = y as f32 / height as f32;

        // Calculate sine wave center position at this x coordinate
        let wave_x = norm_x * self.frequency * 2.0 * PI + self.phase;
        let wave_center_y = 0.5 + self.amplitude * wave_x.sin() * 0.4; // 0.4 keeps it on screen

        // Calculate distance from this cell to wave center
        let distance = (norm_y - wave_center_y).abs();

        // Convert distance to coverage based on thickness (anti-aliasing)
        let half_thickness = self.thickness / height as f32 / 2.0;

        let base_coverage = if distance < half_thickness {
            // Full coverage
            1.0
        } else if distance < half_thickness * 2.0 {
            // Partial coverage (anti-aliasing)
            1.0 - ((distance - half_thickness) / half_thickness)
        } else {
            // No coverage
            0.0
        };

        // Apply beat flash effect (boost coverage on beat)
        let flash_boost = self.beat_flash * 0.3; // Add up to 30% more coverage on beat
        (base_coverage + flash_boost).min(1.0)
    }
}

impl Visualizer for SineWaveVisualizer {
    fn update(&mut self, params: &AudioParameters) {
        // Apply smoothing to prevent jitter
        let smoothing = self.config.smoothing_factor;

        self.amplitude = lerp(
            self.amplitude,
            params.amplitude * self.config.amplitude_sensitivity,
            smoothing,
        );

        self.frequency = lerp(
            self.frequency,
            self.config.base_frequency + params.mid * self.config.frequency_sensitivity,
            smoothing,
        );

        self.thickness = lerp(
            self.thickness,
            1.0 + params.bass * self.config.thickness_sensitivity,
            smoothing,
        );

        // Store frequency band data for color modulation
        self.bass = lerp(self.bass, params.bass, smoothing);
        self.mid = lerp(self.mid, params.mid, smoothing);
        self.treble = lerp(self.treble, params.treble, smoothing);

        // Handle beat flash effect
        if params.beat {
            self.beat_flash = 1.0; // Trigger flash
        } else {
            self.beat_flash *= 0.85; // Decay flash over time
        }

        // Advance phase for animation
        self.phase += self.config.phase_speed;
        if self.phase > 2.0 * PI {
            self.phase -= 2.0 * PI;
        }
    }

    fn render(&self, grid: &mut GridBuffer) {
        // Clear grid first
        grid.clear();

        let width = grid.width();
        let height = grid.height();

        // Use HIGH-RESOLUTION Braille rendering (same as oscilloscope!)
        let mut braille = super::BrailleGrid::new(width, height);
        let dot_width = braille.dot_width(); // 2× width in dots
        let dot_height = braille.dot_height(); // 4× height in dots
        let dot_center_y = dot_height / 2;

        // Draw the sine wave using smooth Braille lines
        let mut prev_x = 0;
        let mut prev_y = dot_center_y;

        for dot_x in 0..dot_width {
            // Calculate sine wave position at this x coordinate (in dot space)
            let norm_x = dot_x as f32 / dot_width as f32;
            let wave_x = norm_x * self.frequency * 2.0 * std::f32::consts::PI + self.phase;
            let wave_center_y = 0.5 + self.amplitude * wave_x.sin() * 0.4;

            // Convert to dot coordinates
            let dot_y =
                (wave_center_y * dot_height as f32).clamp(0.0, (dot_height - 1) as f32) as usize;

            // Draw line from previous point to current point (smooth anti-aliased!)
            if dot_x > 0 {
                // Calculate intensity from amplitude and beat flash
                let intensity = (self.amplitude * 0.3 + self.beat_flash * 0.5).clamp(0.0, 1.0);

                // Get base color from color scheme
                let mut color = match self.color_scheme.get_color(intensity) {
                    Some(c) => c,
                    None => {
                        // Fallback to cyan/blue if monochrome
                        let color_val = (intensity * 200.0).min(255.0) as u8;
                        super::Color::new(0, color_val.saturating_add(50), color_val)
                    }
                };

                // Modulate color with frequency content for dynamic coloring!
                // Bass adds red tint
                let bass_tint = (self.bass * 80.0) as u8;
                color.r = color.r.saturating_add(bass_tint);

                // Treble adds blue tint
                let treble_tint = (self.treble * 60.0) as u8;
                color.b = color.b.saturating_add(treble_tint);

                // Mid adds green tint (subtle)
                let mid_tint = (self.mid * 40.0) as u8;
                color.g = color.g.saturating_add(mid_tint);

                braille.draw_line_with_color(prev_x, prev_y, dot_x, dot_y, color);
            }

            prev_x = dot_x;
            prev_y = dot_y;
        }

        // Convert Braille grid back to regular grid
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
        "Sine Wave"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sine_wave_visualizer_creation() {
        let viz = SineWaveVisualizer::new(
            SineWaveConfig::default(),
            crate::visualization::character_sets::get_character_set(
                crate::visualization::character_sets::CharacterSetType::Blocks,
            ),
        );
        assert_eq!(viz.name(), "Sine Wave");
        assert_eq!(viz.phase, 0.0);
        assert_eq!(viz.amplitude, 0.0);
    }

    #[test]
    fn test_sine_wave_config_default() {
        let config = SineWaveConfig::default();
        assert_eq!(config.amplitude_sensitivity, 10.0);
        assert_eq!(config.frequency_sensitivity, 3.0);
        assert_eq!(config.thickness_sensitivity, 8.0);
        assert_eq!(config.base_frequency, 2.0);
        assert_eq!(config.smoothing_factor, 0.5);
        assert_eq!(config.phase_speed, 0.15);
    }

    #[test]
    fn test_sine_wave_update() {
        let mut viz = SineWaveVisualizer::new(
            SineWaveConfig::default(),
            crate::visualization::character_sets::get_character_set(
                crate::visualization::character_sets::CharacterSetType::Blocks,
            ),
        );
        let params = AudioParameters {
            bass: 0.5,
            mid: 0.3,
            treble: 0.2,
            amplitude: 0.4,
            beat: false,
            beat_flux: false,
            bpm: 120.0,
            tempo_confidence: 0.0,
            spectrum: vec![],
            waveform: vec![],
            waveform_left: vec![],
            waveform_right: vec![],
        };

        let initial_phase = viz.phase;
        viz.update(&params);

        // Phase should advance
        assert!(viz.phase > initial_phase);

        // Values should be smoothed (not instant)
        assert!(viz.amplitude > 0.0);
        assert!(viz.amplitude < params.amplitude * viz.config.amplitude_sensitivity);
    }

    #[test]
    fn test_sine_wave_render() {
        let viz = SineWaveVisualizer::new(
            SineWaveConfig::default(),
            crate::visualization::character_sets::get_character_set(
                crate::visualization::character_sets::CharacterSetType::Blocks,
            ),
        );
        let mut grid = GridBuffer::new(80, 24);

        viz.render(&mut grid);

        // With default config and zero amplitude, wave might not be visible
        // This is expected - just verify render doesn't crash
        // Grid dimensions should be preserved
        assert_eq!(grid.width(), 80);
        assert_eq!(grid.height(), 24);
    }

    #[test]
    fn test_sine_wave_coverage_center() {
        let mut viz = SineWaveVisualizer::new(
            SineWaveConfig::default(),
            crate::visualization::character_sets::get_character_set(
                crate::visualization::character_sets::CharacterSetType::Blocks,
            ),
        );
        viz.amplitude = 0.5;
        viz.thickness = 2.0;

        // At phase=0, x=0, wave should be at center (0.5)
        let coverage_center = viz.calculate_coverage(40, 12, 80, 24);
        let coverage_off = viz.calculate_coverage(40, 0, 80, 24);

        assert!(coverage_center > coverage_off);
    }

    #[test]
    fn test_sine_wave_coverage_respects_thickness() {
        let mut viz = SineWaveVisualizer::new(
            SineWaveConfig::default(),
            crate::visualization::character_sets::get_character_set(
                crate::visualization::character_sets::CharacterSetType::Blocks,
            ),
        );
        viz.amplitude = 0.5;
        viz.phase = 0.0;

        // Test at a point slightly off the wave center
        viz.thickness = 1.0;
        let coverage_thin = viz.calculate_coverage(40, 14, 80, 24);

        viz.thickness = 5.0;
        let coverage_thick = viz.calculate_coverage(40, 14, 80, 24);

        // Thicker wave should have more coverage at the same point
        assert!(
            coverage_thick > coverage_thin,
            "Thick coverage ({}) should be > thin coverage ({})",
            coverage_thick,
            coverage_thin
        );
    }

    #[test]
    fn test_sine_wave_smoothing_prevents_jitter() {
        let mut viz = SineWaveVisualizer::new(
            SineWaveConfig {
                smoothing_factor: 0.1,
                ..Default::default()
            },
            crate::visualization::character_sets::get_character_set(
                crate::visualization::character_sets::CharacterSetType::Blocks,
            ),
        );

        let params = AudioParameters {
            amplitude: 1.0,
            ..Default::default()
        };

        viz.update(&params);
        let amp1 = viz.amplitude;

        viz.update(&params);
        let amp2 = viz.amplitude;

        // Should move toward target but not instantly
        assert!(amp2 > amp1);
        assert!(amp2 < 1.0 * viz.config.amplitude_sensitivity);
    }
}
