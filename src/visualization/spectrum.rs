// Spectrum analyzer visualizer
// Displays frequency spectrum as vertical bars

use super::{lerp, select_character_for_coverage, GridBuffer, Visualizer};
use crate::dsp::AudioParameters;

/// Configuration for spectrum analyzer visualizer
#[derive(Debug, Clone)]
pub struct SpectrumConfig {
    /// Number of frequency bars to display
    pub bar_count: usize,
    /// Minimum frequency to display (Hz)
    pub freq_min: f32,
    /// Maximum frequency to display (Hz)
    pub freq_max: f32,
    /// Smoothing factor (0.0-1.0, higher = smoother)
    pub smoothing_factor: f32,
    /// Amplitude sensitivity multiplier
    pub amplitude_sensitivity: f32,
    /// Bar spacing (0 = no gap, 1 = one char gap)
    pub bar_spacing: usize,
}

impl Default for SpectrumConfig {
    fn default() -> Self {
        Self {
            bar_count: 32,
            freq_min: 20.0,
            freq_max: 20000.0,
            smoothing_factor: 0.7,
            amplitude_sensitivity: 1.5,
            bar_spacing: 0,
        }
    }
}

/// Spectrum analyzer visualizer
///
/// Renders frequency spectrum as vertical bars across the terminal width.
/// Each bar represents a frequency range, with height proportional to amplitude.
///
/// # Audio Parameter Mapping
/// - Frequency spectrum → bar heights
/// - Bass/mid/treble → emphasized frequency ranges
///
/// # Examples
///
/// ```
/// use crabmusic::visualization::{SpectrumVisualizer, SpectrumConfig, Visualizer, GridBuffer};
/// use crabmusic::dsp::AudioParameters;
///
/// let mut viz = SpectrumVisualizer::new(SpectrumConfig::default());
/// let mut grid = GridBuffer::new(80, 24);
/// let params = AudioParameters::default();
///
/// viz.update(&params);
/// viz.render(&mut grid);
/// ```
pub struct SpectrumVisualizer {
    /// Current bar heights (smoothed, 0.0-1.0)
    bar_heights: Vec<f32>,
    /// Configuration
    config: SpectrumConfig,
}

impl SpectrumVisualizer {
    /// Create a new spectrum analyzer visualizer
    ///
    /// # Arguments
    /// * `config` - Configuration for the visualizer
    ///
    /// # Returns
    /// A new SpectrumVisualizer instance
    ///
    /// # Examples
    ///
    /// ```
    /// use crabmusic::visualization::{SpectrumVisualizer, SpectrumConfig, Visualizer};
    ///
    /// let viz = SpectrumVisualizer::new(SpectrumConfig::default());
    /// assert_eq!(viz.name(), "Spectrum Analyzer");
    /// ```
    pub fn new(config: SpectrumConfig) -> Self {
        let bar_heights = vec![0.0; config.bar_count];
        Self {
            bar_heights,
            config,
        }
    }

    /// Extract frequency bars from audio parameters
    ///
    /// Maps bass/mid/treble to frequency bars across the spectrum.
    /// This is a simplified approach - ideally we'd have access to the full FFT spectrum.
    ///
    /// # Arguments
    /// * `params` - Audio parameters
    ///
    /// # Returns
    /// Vector of bar heights (0.0-1.0)
    fn extract_bars(&self, params: &AudioParameters) -> Vec<f32> {
        let mut bars = vec![0.0; self.config.bar_count];
        
        // Divide bars into bass, mid, and treble regions
        let bass_bars = self.config.bar_count / 3;
        let mid_bars = self.config.bar_count / 3;
        let treble_bars = self.config.bar_count - bass_bars - mid_bars;
        
        // Fill bass region (first third)
        for i in 0..bass_bars {
            let falloff = 1.0 - (i as f32 / bass_bars as f32) * 0.3; // Slight falloff
            bars[i] = params.bass * falloff * self.config.amplitude_sensitivity;
        }
        
        // Fill mid region (middle third)
        for i in 0..mid_bars {
            let idx = bass_bars + i;
            let peak = (i as f32 / mid_bars as f32 * std::f32::consts::PI).sin(); // Peak in middle
            bars[idx] = params.mid * peak * self.config.amplitude_sensitivity;
        }
        
        // Fill treble region (last third)
        for i in 0..treble_bars {
            let idx = bass_bars + mid_bars + i;
            let falloff = (i as f32 / treble_bars as f32); // Gradual rise
            bars[idx] = params.treble * falloff * self.config.amplitude_sensitivity;
        }
        
        // Clamp all values to 0.0-1.0
        bars.iter().map(|&h| h.min(1.0).max(0.0)).collect()
    }
}

impl Visualizer for SpectrumVisualizer {
    fn update(&mut self, params: &AudioParameters) {
        // Extract new bar heights from audio parameters
        let new_bars = self.extract_bars(params);
        
        // Apply smoothing to prevent jitter
        let smoothing = self.config.smoothing_factor;
        for (i, &new_height) in new_bars.iter().enumerate() {
            self.bar_heights[i] = lerp(self.bar_heights[i], new_height, smoothing);
        }
    }

    fn render(&self, grid: &mut GridBuffer) {
        // Clear grid first
        grid.clear();
        
        let width = grid.width();
        let height = grid.height();
        
        // Calculate bar width (including spacing)
        let bar_width_with_spacing = width / self.config.bar_count;
        let bar_width = bar_width_with_spacing.saturating_sub(self.config.bar_spacing).max(1);
        
        // Render each bar
        for (bar_idx, &bar_height) in self.bar_heights.iter().enumerate() {
            let x_start = bar_idx * bar_width_with_spacing;
            let x_end = (x_start + bar_width).min(width);
            
            // Calculate bar height in characters
            let bar_height_chars = (bar_height * height as f32) as usize;
            
            // Draw bar from bottom up
            for y in 0..height {
                let y_from_bottom = height - 1 - y;
                
                for x in x_start..x_end {
                    if y_from_bottom < bar_height_chars {
                        // Full character for filled portion
                        let coverage = if y_from_bottom == bar_height_chars - 1 {
                            // Top of bar - use fractional coverage
                            let fractional = (bar_height * height as f32) - bar_height_chars as f32;
                            fractional.max(0.5) // At least 50% for visibility
                        } else {
                            // Full bar
                            1.0
                        };
                        
                        let character = select_character_for_coverage(coverage);
                        grid.set_cell(x, y, character);
                    }
                }
            }
        }
    }

    fn name(&self) -> &str {
        "Spectrum Analyzer"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spectrum_creation() {
        let viz = SpectrumVisualizer::new(SpectrumConfig::default());
        assert_eq!(viz.name(), "Spectrum Analyzer");
        assert_eq!(viz.bar_heights.len(), 32);
    }

    #[test]
    fn test_spectrum_update() {
        let mut viz = SpectrumVisualizer::new(SpectrumConfig::default());
        let params = AudioParameters {
            bass: 0.8,
            mid: 0.5,
            treble: 0.3,
            amplitude: 0.6,
            beat: false,
        };

        viz.update(&params);

        // Bar heights should be updated (smoothed, not instant)
        assert!(viz.bar_heights.iter().any(|&h| h > 0.0));
        
        // Bass bars (first third) should be higher
        let bass_avg: f32 = viz.bar_heights[..10].iter().sum::<f32>() / 10.0;
        assert!(bass_avg > 0.0);
    }

    #[test]
    fn test_spectrum_render() {
        let mut viz = SpectrumVisualizer::new(SpectrumConfig {
            bar_count: 10,
            ..Default::default()
        });
        let mut grid = GridBuffer::new(40, 20);
        
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

    #[test]
    fn test_extract_bars() {
        let viz = SpectrumVisualizer::new(SpectrumConfig {
            bar_count: 30,
            amplitude_sensitivity: 1.0,
            ..Default::default()
        });
        
        let params = AudioParameters {
            bass: 1.0,
            mid: 0.5,
            treble: 0.2,
            amplitude: 0.6,
            beat: false,
        };
        
        let bars = viz.extract_bars(&params);
        
        assert_eq!(bars.len(), 30);
        // All bars should be in valid range
        assert!(bars.iter().all(|&h| h >= 0.0 && h <= 1.0));
    }
}

