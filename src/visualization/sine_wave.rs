// Sine wave visualizer
// MVP visualizer that renders an audio-reactive sine wave

use super::{lerp, select_character_for_coverage, GridBuffer, Visualizer};
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
    pub fn new(config: SineWaveConfig) -> Self {
        Self {
            phase: 0.0,
            amplitude: 0.0,
            frequency: config.base_frequency,
            thickness: 1.0,
            config,
            beat_flash: 0.0,
        }
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

        // Render sine wave
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                let coverage = self.calculate_coverage(x, y, grid.width(), grid.height());
                let character = select_character_for_coverage(coverage);
                grid.set_cell(x, y, character);
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
        let viz = SineWaveVisualizer::new(SineWaveConfig::default());
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
        let mut viz = SineWaveVisualizer::new(SineWaveConfig::default());
        let params = AudioParameters {
            bass: 0.5,
            mid: 0.3,
            treble: 0.2,
            amplitude: 0.4,
            beat: false,
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
        let viz = SineWaveVisualizer::new(SineWaveConfig::default());
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
        let mut viz = SineWaveVisualizer::new(SineWaveConfig::default());
        viz.amplitude = 0.5;
        viz.thickness = 2.0;

        // At phase=0, x=0, wave should be at center (0.5)
        let coverage_center = viz.calculate_coverage(40, 12, 80, 24);
        let coverage_off = viz.calculate_coverage(40, 0, 80, 24);

        assert!(coverage_center > coverage_off);
    }

    #[test]
    fn test_sine_wave_coverage_respects_thickness() {
        let mut viz = SineWaveVisualizer::new(SineWaveConfig::default());
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
        let mut viz = SineWaveVisualizer::new(SineWaveConfig {
            smoothing_factor: 0.1,
            ..Default::default()
        });

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
