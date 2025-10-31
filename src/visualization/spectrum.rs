// Spectrum analyzer visualizer
// Displays frequency spectrum as vertical bars

use super::{character_sets::CharacterSet, lerp, GridBuffer, Visualizer};
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
    /// Bar spacing (0 = no gap, 1 = one char gap, 2 = two char gap)
    pub bar_spacing: usize,
    /// Enable peak hold indicators
    pub peak_hold_enabled: bool,
    /// Peak hold decay rate (units per frame)
    pub peak_decay_rate: f32,
    /// Show frequency labels for debugging/calibration
    pub show_labels: bool,
}

impl Default for SpectrumConfig {
    fn default() -> Self {
        Self {
            bar_count: 32,
            freq_min: 20.0,
            freq_max: 20000.0,
            smoothing_factor: 0.0, // NO SMOOTHING for instant response to drums/beats
            amplitude_sensitivity: 2.5, // Boosted slightly to compensate
            bar_spacing: 0,
            peak_hold_enabled: true,
            peak_decay_rate: 0.05, // Faster decay to match no smoothing
            show_labels: false,
        }
    }
}

impl SpectrumConfig {
    /// Validate configuration parameters
    ///
    /// # Returns
    /// true if configuration is valid, false otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use crabmusic::visualization::SpectrumConfig;
    ///
    /// let config = SpectrumConfig::default();
    /// assert!(config.is_valid());
    ///
    /// let invalid = SpectrumConfig {
    ///     bar_count: 0,
    ///     ..Default::default()
    /// };
    /// assert!(!invalid.is_valid());
    /// ```
    pub fn is_valid(&self) -> bool {
        self.bar_count > 0
            && self.freq_min > 0.0
            && self.freq_max > self.freq_min
            && (0.0..=1.0).contains(&self.smoothing_factor)
            && self.amplitude_sensitivity > 0.0
            && (0.0..=1.0).contains(&self.peak_decay_rate)
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
/// let mut viz = SpectrumVisualizer::new(SpectrumConfig::default(), 44100);
/// let mut grid = GridBuffer::new(80, 24);
/// let params = AudioParameters::default();
///
/// viz.update(&params);
/// viz.render(&mut grid);
/// ```
pub struct SpectrumVisualizer {
    /// Current bar heights (smoothed, 0.0-1.0)
    bar_heights: Vec<f32>,
    /// Peak hold values for each bar
    peak_heights: Vec<f32>,
    /// Configuration
    config: SpectrumConfig,
    /// Sample rate (needed for frequency mapping)
    sample_rate: u32,
    /// Beat flash effect (0.0-1.0, decays over time)
    beat_flash: f32,
    /// Character set for rendering (smooth gradients)
    charset: CharacterSet,
    /// Color scheme for rendering
    color_scheme: super::color_schemes::ColorScheme,
}

impl SpectrumVisualizer {
    /// Create a new spectrum analyzer visualizer
    ///
    /// # Arguments
    /// * `config` - Configuration for the visualizer
    /// * `sample_rate` - Audio sample rate in Hz
    ///
    /// # Returns
    /// A new SpectrumVisualizer instance
    ///
    /// # Examples
    ///
    /// ```
    /// use crabmusic::visualization::{SpectrumVisualizer, SpectrumConfig, Visualizer};
    ///
    /// let viz = SpectrumVisualizer::new(SpectrumConfig::default(), 44100);
    /// assert_eq!(viz.name(), "Spectrum Analyzer");
    /// ```
    pub fn new(config: SpectrumConfig, sample_rate: u32, charset: CharacterSet) -> Self {
        // Validate configuration
        assert!(
            config.is_valid(),
            "Invalid SpectrumConfig: bar_count must be > 0, frequencies must be positive, freq_max > freq_min, smoothing/decay must be 0.0-1.0"
        );
        assert!(sample_rate > 0, "Sample rate must be > 0");

        let bar_heights = vec![0.0; config.bar_count];
        let peak_heights = vec![0.0; config.bar_count];
        Self {
            bar_heights,
            peak_heights,
            config,
            sample_rate,
            beat_flash: 0.0,
            charset,
            color_scheme: super::color_schemes::ColorScheme::new(super::color_schemes::ColorSchemeType::Monochrome),
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

    /// Map bar index to frequency range using logarithmic scaling
    ///
    /// Human hearing is logarithmic - an octave from 100-200 Hz sounds the same
    /// "distance" as 1000-2000 Hz. This function uses logarithmic scaling to map
    /// bars to perceptually equal frequency ranges.
    ///
    /// # Arguments
    /// * `bar_index` - Visual bar index (0 to bar_count-1)
    ///
    /// # Returns
    /// Tuple of (freq_min, freq_max) in Hz for this bar
    ///
    /// # Examples
    /// For 32 bars spanning 20-20000 Hz:
    /// - Bar 0: 20-25 Hz (bass)
    /// - Bar 16: ~400-600 Hz (mid)
    /// - Bar 31: ~15000-20000 Hz (treble)
    fn bar_to_frequency_range(&self, bar_index: usize) -> (f32, f32) {
        let f_min = self.config.freq_min;
        let f_max = self.config.freq_max;
        let n = self.config.bar_count as f32;
        let i = bar_index as f32;

        // Logarithmic scaling: f(i) = f_min * (f_max/f_min)^(i/n)
        let ratio = f_max / f_min;
        let freq_start = f_min * ratio.powf(i / n);
        let freq_end = f_min * ratio.powf((i + 1.0) / n);

        (freq_start, freq_end)
    }

    /// Extract bar height from FFT spectrum
    ///
    /// Aggregates FFT bins within the bar's frequency range to compute
    /// the bar's visual height. Uses averaging to prevent single bins
    /// from dominating the display.
    ///
    /// # Arguments
    /// * `spectrum` - FFT magnitude spectrum from AudioParameters
    /// * `bar_index` - Visual bar index
    ///
    /// # Returns
    /// Normalized bar height (0.0-1.0)
    ///
    /// # Edge Cases
    /// - Empty spectrum returns 0.0
    /// - Invalid bar_index returns 0.0
    /// - Empty frequency range returns 0.0
    fn extract_bar_from_spectrum(&self, spectrum: &[f32], bar_index: usize) -> f32 {
        // Handle empty spectrum (silence or initialization)
        if spectrum.is_empty() {
            return 0.0;
        }

        // Validate bar index
        if bar_index >= self.config.bar_count {
            return 0.0;
        }

        let (freq_min, freq_max) = self.bar_to_frequency_range(bar_index);

        // Convert frequency range to FFT bin range
        let window_size = spectrum.len() * 2; // Spectrum is half of window
        let bin_min = (freq_min * window_size as f32 / self.sample_rate as f32).ceil() as usize;
        let bin_max = (freq_max * window_size as f32 / self.sample_rate as f32).floor() as usize;

        // Clamp to valid range
        let bin_min = bin_min.min(spectrum.len());
        let bin_max = bin_max.min(spectrum.len());

        // Handle edge cases
        if bin_min >= bin_max {
            return 0.0;
        }

        // Average magnitude in this frequency range
        let sum: f32 = spectrum[bin_min..bin_max].iter().sum();
        let count = (bin_max - bin_min) as f32;

        if count > 0.0 {
            // Apply sensitivity and clamp to reasonable range
            let raw_height = (sum / count) * self.config.amplitude_sensitivity;
            raw_height.min(2.0) // Clamp to prevent extreme values
        } else {
            0.0
        }
    }

    /// Render frequency range labels for debugging/calibration
    ///
    /// Displays frequency ranges and band names at the bottom of the visualization
    /// to help understand which bars correspond to which frequencies.
    fn render_labels(&self, grid: &mut GridBuffer) {
        let width = grid.width();
        let height = grid.height();

        if height < 3 {
            return; // Need at least 3 lines for labels
        }

        // Define frequency bands (professional audio ranges)
        let bands = [
            (20.0, 60.0, "SUB", super::Color::new(80, 20, 20)),      // Sub-bass
            (60.0, 250.0, "BASS", super::Color::new(255, 50, 0)),    // Bass
            (250.0, 500.0, "LMID", super::Color::new(255, 150, 0)),  // Low-mid
            (500.0, 2000.0, "MID", super::Color::new(50, 255, 50)),  // Mid
            (2000.0, 4000.0, "HMID", super::Color::new(0, 200, 100)), // High-mid
            (4000.0, 6000.0, "PRES", super::Color::new(0, 150, 255)), // Presence
            (6000.0, 20000.0, "TREB", super::Color::new(0, 100, 255)), // Treble
        ];

        // Draw band labels at bottom
        let label_y = height - 2;

        for (freq_min, freq_max, label, color) in &bands {
            // Find which bar(s) fall into this frequency range
            let mut start_bar = None;
            let mut end_bar = None;

            for bar_idx in 0..self.config.bar_count {
                let (bar_f_min, bar_f_max) = self.bar_to_frequency_range(bar_idx);

                // Check if this bar overlaps with the frequency band
                if bar_f_max >= *freq_min && bar_f_min <= *freq_max {
                    if start_bar.is_none() {
                        start_bar = Some(bar_idx);
                    }
                    end_bar = Some(bar_idx);
                }
            }

            if let (Some(start), Some(end)) = (start_bar, end_bar) {
                // Calculate pixel position for label
                let bar_width_with_spacing = width / self.config.bar_count;
                let start_x = start * bar_width_with_spacing;
                let end_x = (end + 1) * bar_width_with_spacing;
                let center_x = (start_x + end_x) / 2;

                // Place label at center of band
                let label_start = center_x.saturating_sub(label.len() / 2);

                for (i, ch) in label.chars().enumerate() {
                    let x = label_start + i;
                    if x < width {
                        grid.set_cell_with_color(x, label_y, ch, *color);
                    }
                }
            }
        }

        // Draw frequency values at key points (bottom-most line)
        let freq_y = height - 1;
        let freq_points = [
            (0, self.config.freq_min),
            (self.config.bar_count / 4, 100.0),
            (self.config.bar_count / 2, 1000.0),
            (3 * self.config.bar_count / 4, 5000.0),
            (self.config.bar_count - 1, self.config.freq_max),
        ];

        for (bar_idx, _target_freq) in &freq_points {
            let bar_idx = (*bar_idx).min(self.config.bar_count - 1);
            let (f_min, f_max) = self.bar_to_frequency_range(bar_idx);
            let freq = (f_min + f_max) / 2.0;

            // Format frequency
            let freq_str = if freq >= 1000.0 {
                format!("{}k", (freq / 1000.0) as u32)
            } else {
                format!("{}Hz", freq as u32)
            };

            // Calculate position
            let bar_width_with_spacing = width / self.config.bar_count;
            let x = bar_idx * bar_width_with_spacing;

            for (i, ch) in freq_str.chars().enumerate() {
                let char_x = x + i;
                if char_x < width {
                    grid.set_cell(char_x, freq_y, ch);
                }
            }
        }
    }
}

impl Visualizer for SpectrumVisualizer {
    fn update(&mut self, params: &AudioParameters) {
        // Extract bar heights from real FFT spectrum
        for i in 0..self.config.bar_count {
            let target_height = self.extract_bar_from_spectrum(&params.spectrum, i);

            // Apply smoothing to prevent jitter
            self.bar_heights[i] = lerp(
                self.bar_heights[i],
                target_height.min(1.0),
                self.config.smoothing_factor,
            );

            // Update peak hold
            if self.config.peak_hold_enabled {
                if self.bar_heights[i] > self.peak_heights[i] {
                    // New peak detected
                    self.peak_heights[i] = self.bar_heights[i];
                } else {
                    // Decay peak slowly
                    self.peak_heights[i] =
                        (self.peak_heights[i] - self.config.peak_decay_rate).max(0.0);
                }
            }
        }

        // Handle beat flash effect
        if params.beat {
            self.beat_flash = 1.0; // Trigger flash
        } else {
            self.beat_flash *= 0.85; // Decay flash over time
        }
    }

    fn render(&self, grid: &mut GridBuffer) {
        // Clear grid first
        grid.clear();

        let width = grid.width();
        let height = grid.height();

        // Use HIGH-RESOLUTION Braille rendering (same as oscilloscope!)
        let mut braille = super::BrailleGrid::new(width, height);
        let dot_width = braille.dot_width();   // 2× width in dots
        let dot_height = braille.dot_height(); // 4× height in dots

        // Calculate bar width in dot space (including spacing)
        let bar_width_with_spacing = dot_width / self.config.bar_count;
        let bar_width = bar_width_with_spacing.saturating_sub(self.config.bar_spacing * 2).max(1);

        // Render each bar in high-resolution dot space
        for bar_idx in 0..self.config.bar_count {
            let x_start = bar_idx * bar_width_with_spacing;
            let x_end = (x_start + bar_width).min(dot_width);

            // Calculate bar height in DOTS (with beat flash boost)
            let boosted_height = (self.bar_heights[bar_idx] + self.beat_flash * 0.2).min(1.0);
            let bar_height_dots = (boosted_height * dot_height as f32) as usize;

            // Calculate peak position in dots
            let peak_dot_y = dot_height.saturating_sub((self.peak_heights[bar_idx] * dot_height as f32) as usize);

            // Determine bar color based on color scheme or frequency band
            let intensity = boosted_height;

            let bar_color = if let Some(scheme_color) = self.color_scheme.get_color(intensity) {
                // Use color scheme if it provides a color (not monochrome)
                scheme_color
            } else {
                // Otherwise use frequency-based RGB coloring!
                // Divide spectrum into 3 bands: Bass (Red), Mid (Green), Treble (Blue)
                let bass_bars = self.config.bar_count / 3;
                let mid_bars = bass_bars * 2;

                if bar_idx < bass_bars {
                    // BASS (0-250 Hz) = RED
                    super::Color::new(
                        (intensity * 255.0) as u8,
                        (intensity * 50.0) as u8,
                        0
                    )
                } else if bar_idx < mid_bars {
                    // MID (250-4000 Hz) = GREEN
                    super::Color::new(
                        (intensity * 50.0) as u8,
                        (intensity * 255.0) as u8,
                        (intensity * 50.0) as u8,
                    )
                } else {
                    // TREBLE (4000+ Hz) = BLUE
                    super::Color::new(
                        0,
                        (intensity * 100.0) as u8,
                        (intensity * 255.0) as u8,
                    )
                }
            };

            // Draw bar from bottom up (with smooth top!)
            for dot_x in x_start..x_end {
                let dots_from_bottom = bar_height_dots;

                for dot_y_from_bottom in 0..dots_from_bottom {
                    let dot_y = dot_height.saturating_sub(1 + dot_y_from_bottom);

                    // Gradient: brighter at top, dimmer at bottom
                    let y_ratio = dot_y_from_bottom as f32 / dots_from_bottom.max(1) as f32;
                    let brightness = 0.3 + y_ratio * 0.7; // 30% to 100% brightness

                    let gradient_color = super::Color::new(
                        (bar_color.r as f32 * brightness) as u8,
                        (bar_color.g as f32 * brightness) as u8,
                        (bar_color.b as f32 * brightness) as u8,
                    );

                    braille.set_dot_with_color(dot_x, dot_y, gradient_color);
                }

                // Draw peak indicator (single dot line at peak height)
                if self.config.peak_hold_enabled && self.peak_heights[bar_idx] > 0.05 {
                    if peak_dot_y < dot_height {
                        let peak_color = super::Color::new(255, 255, 100); // Bright yellow peak
                        braille.set_dot_with_color(dot_x, peak_dot_y, peak_color);
                    }
                }
            }
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

        // Render frequency labels if enabled
        if self.config.show_labels {
            self.render_labels(grid);
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
        let viz = SpectrumVisualizer::new(
            SpectrumConfig::default(),
            44100,
            crate::visualization::character_sets::get_character_set(
                crate::visualization::character_sets::CharacterSetType::Blocks
            ),
        );
        assert_eq!(viz.name(), "Spectrum Analyzer");
        assert_eq!(viz.bar_heights.len(), 32);
        assert_eq!(viz.peak_heights.len(), 32);
    }

    #[test]
    fn test_logarithmic_frequency_mapping() {
        let config = SpectrumConfig::default();
        let viz = SpectrumVisualizer::new(
            config,
            44100,
            crate::visualization::character_sets::get_character_set(
                crate::visualization::character_sets::CharacterSetType::Blocks
            ),
        );

        // First bar should start at freq_min
        let (f_min, _f_max) = viz.bar_to_frequency_range(0);
        assert!((f_min - 20.0).abs() < 1.0);

        // Last bar should end at freq_max
        let (_f_min, f_max) = viz.bar_to_frequency_range(31);
        assert!((f_max - 20000.0).abs() < 100.0);

        // Middle bars should be logarithmically spaced
        // Each octave should have similar number of bars
        let (f1_min, f1_max) = viz.bar_to_frequency_range(10);
        let (f2_min, f2_max) = viz.bar_to_frequency_range(20);

        // Ratio should be consistent (logarithmic property)
        let ratio1 = f1_max / f1_min;
        let ratio2 = f2_max / f2_min;
        assert!((ratio1 - ratio2).abs() < 0.1);
    }

    #[test]
    fn test_extract_bar_from_spectrum() {
        let config = SpectrumConfig::default();
        let viz = SpectrumVisualizer::new(
            config,
            44100,
            crate::visualization::character_sets::get_character_set(
                crate::visualization::character_sets::CharacterSetType::Blocks
            ),
        );

        // Create synthetic spectrum with peak at 1000 Hz
        let mut spectrum = vec![0.0; 1024];
        let peak_bin = (1000.0 * 2048.0 / 44100.0) as usize;
        spectrum[peak_bin] = 1.0;

        // Bar containing 1000 Hz should have highest value
        let mut max_bar = 0;
        let mut max_height = 0.0;

        for i in 0..32 {
            let height = viz.extract_bar_from_spectrum(&spectrum, i);
            if height > max_height {
                max_height = height;
                max_bar = i;
            }
        }

        // Verify the correct bar has the peak
        let (f_min, f_max) = viz.bar_to_frequency_range(max_bar);
        assert!(f_min <= 1000.0 && 1000.0 <= f_max);
    }

    #[test]
    fn test_peak_hold_behavior() {
        let config = SpectrumConfig {
            peak_hold_enabled: true,
            peak_decay_rate: 0.1,
            ..Default::default()
        };
        let mut viz = SpectrumVisualizer::new(
            config,
            44100,
            crate::visualization::character_sets::get_character_set(
                crate::visualization::character_sets::CharacterSetType::Blocks
            ),
        );

        // Create params with high energy
        let mut params = AudioParameters::default();
        params.spectrum = vec![1.0; 1024];

        viz.update(&params);
        let peak_after_high = viz.peak_heights[0];

        // Update with low energy
        params.spectrum = vec![0.1; 1024];
        viz.update(&params);
        let peak_after_low = viz.peak_heights[0];

        // Peak should decay but stay higher than current
        assert!(peak_after_low < peak_after_high);
        assert!(peak_after_low > viz.bar_heights[0]);
    }

    #[test]
    fn test_smoothing_prevents_jitter() {
        let config = SpectrumConfig {
            smoothing_factor: 0.3,
            ..Default::default()
        };
        let mut viz = SpectrumVisualizer::new(
            config,
            44100,
            crate::visualization::character_sets::get_character_set(
                crate::visualization::character_sets::CharacterSetType::Blocks
            ),
        );

        // Sudden change in spectrum
        let mut params = AudioParameters::default();
        params.spectrum = vec![0.0; 1024];
        viz.update(&params);

        params.spectrum = vec![1.0; 1024];
        viz.update(&params);

        // Height should move toward target but not instantly
        assert!(viz.bar_heights[0] > 0.0);
        assert!(viz.bar_heights[0] < 1.0);
    }

    #[test]
    fn test_spectrum_update() {
        let mut viz = SpectrumVisualizer::new(
            SpectrumConfig::default(),
            44100,
            crate::visualization::character_sets::get_character_set(
                crate::visualization::character_sets::CharacterSetType::Blocks
            ),
        );
        let mut params = AudioParameters::default();
        params.spectrum = vec![0.5; 1024];

        viz.update(&params);

        // Bar heights should be updated (smoothed, not instant)
        assert!(viz.bar_heights.iter().any(|&h| h > 0.0));
    }

    #[test]
    fn test_spectrum_render() {
        let mut viz = SpectrumVisualizer::new(
            SpectrumConfig {
                bar_count: 10,
                ..Default::default()
            },
            44100,
            crate::visualization::character_sets::get_character_set(
                crate::visualization::character_sets::CharacterSetType::Blocks
            ),
        );
        let mut grid = GridBuffer::new(40, 20);

        let mut params = AudioParameters::default();
        params.spectrum = vec![0.5; 1024];

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
    fn test_empty_spectrum_handling() {
        let viz = SpectrumVisualizer::new(
            SpectrumConfig::default(),
            44100,
            crate::visualization::character_sets::get_character_set(
                crate::visualization::character_sets::CharacterSetType::Blocks
            ),
        );
        let empty_spectrum: Vec<f32> = vec![];

        // Should not panic and should return 0.0
        let height = viz.extract_bar_from_spectrum(&empty_spectrum, 0);
        assert_eq!(height, 0.0);
    }

    #[test]
    fn test_config_validation() {
        // Valid config
        let valid = SpectrumConfig::default();
        assert!(valid.is_valid());

        // Invalid bar count
        let invalid_bars = SpectrumConfig {
            bar_count: 0,
            ..Default::default()
        };
        assert!(!invalid_bars.is_valid());

        // Invalid frequency range
        let invalid_freq = SpectrumConfig {
            freq_min: 0.0,
            ..Default::default()
        };
        assert!(!invalid_freq.is_valid());

        let invalid_freq_order = SpectrumConfig {
            freq_min: 20000.0,
            freq_max: 20.0,
            ..Default::default()
        };
        assert!(!invalid_freq_order.is_valid());

        // Invalid smoothing factor
        let invalid_smoothing = SpectrumConfig {
            smoothing_factor: 1.5,
            ..Default::default()
        };
        assert!(!invalid_smoothing.is_valid());

        // Invalid peak decay rate
        let invalid_decay = SpectrumConfig {
            peak_decay_rate: 2.0,
            ..Default::default()
        };
        assert!(!invalid_decay.is_valid());
    }

    #[test]
    #[should_panic(expected = "Invalid SpectrumConfig")]
    fn test_new_with_invalid_config_panics() {
        let invalid_config = SpectrumConfig {
            bar_count: 0,
            ..Default::default()
        };
        SpectrumVisualizer::new(
            invalid_config,
            44100,
            crate::visualization::character_sets::get_character_set(
                crate::visualization::character_sets::CharacterSetType::Blocks
            ),
        );
    }

    #[test]
    #[should_panic(expected = "Sample rate must be > 0")]
    fn test_new_with_zero_sample_rate_panics() {
        SpectrumVisualizer::new(
            SpectrumConfig::default(),
            0,
            crate::visualization::character_sets::get_character_set(
                crate::visualization::character_sets::CharacterSetType::Blocks
            ),
        );
    }

    #[test]
    fn test_extract_bar_with_invalid_index() {
        let viz = SpectrumVisualizer::new(
            SpectrumConfig::default(),
            44100,
            crate::visualization::character_sets::get_character_set(
                crate::visualization::character_sets::CharacterSetType::Blocks
            ),
        );
        let spectrum = vec![0.5; 1024];

        // Index beyond bar count should return 0.0
        let height = viz.extract_bar_from_spectrum(&spectrum, 999);
        assert_eq!(height, 0.0);
    }

    #[test]
    fn test_extreme_amplitude_clamping() {
        let config = SpectrumConfig {
            amplitude_sensitivity: 100.0, // Extreme sensitivity
            ..Default::default()
        };
        let viz = SpectrumVisualizer::new(
            config,
            44100,
            crate::visualization::character_sets::get_character_set(
                crate::visualization::character_sets::CharacterSetType::Blocks
            ),
        );
        let spectrum = vec![1.0; 1024]; // All bins at max

        // Should be clamped to 2.0 max
        let height = viz.extract_bar_from_spectrum(&spectrum, 0);
        assert!(height <= 2.0, "Height should be clamped to 2.0, got {}", height);
    }
}

