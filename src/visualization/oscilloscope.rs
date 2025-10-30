// Oscilloscope visualizer
// Displays waveform over time like a classic oscilloscope

use super::{lerp, select_character_for_coverage, GridBuffer, Visualizer};
use crate::dsp::AudioParameters;
use std::collections::VecDeque;

/// Configuration for oscilloscope visualizer
#[derive(Debug, Clone)]
pub struct OscilloscopeConfig {
    /// Number of samples to display (time window)
    pub sample_count: usize,
    /// Amplitude sensitivity multiplier
    pub amplitude_sensitivity: f32,
    /// Smoothing factor (0.0-1.0, higher = smoother)
    pub smoothing_factor: f32,
    /// Line thickness in rows
    pub line_thickness: f32,
    /// Trigger level (0.0-1.0, for stable display)
    pub trigger_level: f32,
}

impl Default for OscilloscopeConfig {
    fn default() -> Self {
        Self {
            sample_count: 200,
            amplitude_sensitivity: 1.5,
            smoothing_factor: 0.3,
            line_thickness: 2.0,
            trigger_level: 0.0,
        }
    }
}

/// Oscilloscope visualizer
///
/// Renders audio waveform over time, scrolling from right to left.
/// Shows the actual waveform shape like a classic oscilloscope.
///
/// # Audio Parameter Mapping
/// - `amplitude` → waveform amplitude (vertical scale)
/// - `bass` → line thickness
/// - Waveform scrolls continuously
///
/// # Examples
///
/// ```
/// use crabmusic::visualization::{OscilloscopeVisualizer, OscilloscopeConfig, Visualizer, GridBuffer};
/// use crabmusic::dsp::AudioParameters;
///
/// let mut viz = OscilloscopeVisualizer::new(OscilloscopeConfig::default());
/// let mut grid = GridBuffer::new(80, 24);
/// let params = AudioParameters::default();
///
/// viz.update(&params);
/// viz.render(&mut grid);
/// ```
pub struct OscilloscopeVisualizer {
    /// Waveform sample buffer (circular buffer)
    samples: VecDeque<f32>,
    /// Current amplitude (smoothed)
    amplitude: f32,
    /// Current line thickness (smoothed)
    thickness: f32,
    /// Configuration
    config: OscilloscopeConfig,
    /// Beat flash effect (0.0-1.0, decays over time)
    beat_flash: f32,
}

impl OscilloscopeVisualizer {
    /// Create a new oscilloscope visualizer
    ///
    /// # Arguments
    /// * `config` - Configuration for the visualizer
    ///
    /// # Returns
    /// A new OscilloscopeVisualizer instance
    ///
    /// # Examples
    ///
    /// ```
    /// use crabmusic::visualization::{OscilloscopeVisualizer, OscilloscopeConfig, Visualizer};
    ///
    /// let viz = OscilloscopeVisualizer::new(OscilloscopeConfig::default());
    /// assert_eq!(viz.name(), "Oscilloscope");
    /// ```
    pub fn new(config: OscilloscopeConfig) -> Self {
        let mut samples = VecDeque::with_capacity(config.sample_count);
        // Initialize with zeros
        for _ in 0..config.sample_count {
            samples.push_back(0.0);
        }
        
        Self {
            samples,
            amplitude: 0.0,
            thickness: config.line_thickness,
            config,
            beat_flash: 0.0,
        }
    }

    /// Generate synthetic waveform from audio parameters
    ///
    /// Creates a waveform that represents the current audio state.
    /// This is a simplified approach - ideally we'd have access to raw audio samples.
    ///
    /// # Arguments
    /// * `params` - Audio parameters
    ///
    /// # Returns
    /// Waveform value (-1.0 to 1.0)
    fn generate_waveform_sample(&self, params: &AudioParameters, phase: f32) -> f32 {
        // Combine bass, mid, and treble into a composite waveform
        let bass_wave = (phase * 2.0 * std::f32::consts::PI).sin() * params.bass;
        let mid_wave = (phase * 8.0 * std::f32::consts::PI).sin() * params.mid * 0.5;
        let treble_wave = (phase * 20.0 * std::f32::consts::PI).sin() * params.treble * 0.3;
        
        // Combine and normalize
        let combined = bass_wave + mid_wave + treble_wave;
        combined.max(-1.0).min(1.0)
    }

    /// Calculate coverage for a grid cell
    ///
    /// Determines what percentage of the cell is covered by the waveform line.
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
        // Map x to sample index
        let sample_idx = (x as f32 / width as f32 * self.samples.len() as f32) as usize;
        if sample_idx >= self.samples.len() {
            return 0.0;
        }
        
        // Get waveform value at this x position
        let waveform_value = self.samples[sample_idx];
        
        // Convert waveform value (-1.0 to 1.0) to y position (0.0 to 1.0)
        let waveform_y = 0.5 - (waveform_value * self.amplitude * 0.4); // 0.4 keeps it on screen
        
        // Normalize y coordinate
        let norm_y = y as f32 / height as f32;
        
        // Calculate distance from waveform line
        let distance = (norm_y - waveform_y).abs();
        
        // Convert distance to coverage based on thickness
        let half_thickness = self.thickness / height as f32 / 2.0;

        let base_coverage = if distance < half_thickness {
            // Inside the line - full coverage
            1.0
        } else if distance < half_thickness * 2.0 {
            // Edge of line - partial coverage (anti-aliasing)
            1.0 - (distance - half_thickness) / half_thickness
        } else {
            // Outside the line
            0.0
        };

        // Apply beat flash effect (boost coverage on beat)
        let flash_boost = self.beat_flash * 0.3; // Add up to 30% more coverage on beat
        (base_coverage + flash_boost).min(1.0)
    }
}

impl Visualizer for OscilloscopeVisualizer {
    fn update(&mut self, params: &AudioParameters) {
        // Apply smoothing to amplitude and thickness
        let smoothing = self.config.smoothing_factor;

        self.amplitude = lerp(
            self.amplitude,
            params.amplitude * self.config.amplitude_sensitivity,
            smoothing,
        );

        self.thickness = lerp(
            self.thickness,
            self.config.line_thickness + params.bass * 2.0,
            smoothing,
        );

        // Handle beat flash effect
        if params.beat {
            self.beat_flash = 1.0; // Trigger flash
        } else {
            self.beat_flash *= 0.85; // Decay flash over time
        }

        // Add new samples to the buffer (simulate continuous waveform)
        // In a real implementation, we'd use actual audio samples
        for i in 0..5 {
            let phase = i as f32 / 5.0;
            let sample = self.generate_waveform_sample(params, phase);

            // Remove oldest sample and add new one
            self.samples.pop_front();
            self.samples.push_back(sample);
        }
    }

    fn render(&self, grid: &mut GridBuffer) {
        // Clear grid first
        grid.clear();
        
        let width = grid.width();
        let height = grid.height();
        
        // Draw center line (reference)
        let center_y = height / 2;
        for x in 0..width {
            if x % 4 == 0 {
                grid.set_cell(x, center_y, '·');
            }
        }
        
        // Render waveform
        for y in 0..height {
            for x in 0..width {
                let coverage = self.calculate_coverage(x, y, width, height);
                if coverage > 0.1 {
                    let character = select_character_for_coverage(coverage);
                    grid.set_cell(x, y, character);
                }
            }
        }
    }

    fn name(&self) -> &str {
        "Oscilloscope"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oscilloscope_creation() {
        let viz = OscilloscopeVisualizer::new(OscilloscopeConfig::default());
        assert_eq!(viz.name(), "Oscilloscope");
        assert_eq!(viz.samples.len(), 200);
    }

    #[test]
    fn test_oscilloscope_update() {
        let mut viz = OscilloscopeVisualizer::new(OscilloscopeConfig::default());
        let params = AudioParameters {
            bass: 0.8,
            mid: 0.5,
            treble: 0.3,
            amplitude: 0.6,
            beat: false,
        };

        let initial_amplitude = viz.amplitude;
        viz.update(&params);

        // Amplitude should change (smoothed)
        assert!(viz.amplitude != initial_amplitude);
        
        // Samples should be updated
        assert!(viz.samples.iter().any(|&s| s != 0.0));
    }

    #[test]
    fn test_oscilloscope_render() {
        let mut viz = OscilloscopeVisualizer::new(OscilloscopeConfig::default());
        let mut grid = GridBuffer::new(80, 24);
        
        let params = AudioParameters {
            bass: 0.5,
            mid: 0.5,
            treble: 0.5,
            amplitude: 0.5,
            beat: false,
        };
        
        viz.update(&params);
        viz.render(&mut grid);
        
        // Grid should have some non-space characters
        let mut has_content = false;
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                if grid.get_cell(x, y).character != ' ' {
                    has_content = true;
                    break;
                }
            }
        }
        assert!(has_content, "Grid should have some visualization content");
    }
}

