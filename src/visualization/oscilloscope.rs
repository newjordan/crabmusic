// Oscilloscope visualizer
// Displays waveform over time like a classic oscilloscope

use super::{lerp, BrailleGrid, Color, GridBuffer, Visualizer};
use crate::dsp::AudioParameters;

/// Trigger slope for oscilloscope display
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TriggerSlope {
    /// Trigger on rising edge
    Positive,
    /// Trigger on falling edge
    Negative,
    /// Trigger on either edge
    Both,
}

/// Waveform display mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WaveformMode {
    /// Draw line only
    Line,
    /// Fill area under waveform
    Filled,
    /// Draw line with fill
    LineAndFill,
}

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
    /// Enable trigger for stable display
    pub trigger_enabled: bool,
    /// Trigger level (-1.0 to 1.0)
    pub trigger_level: f32,
    /// Trigger slope
    pub trigger_slope: TriggerSlope,
    /// Show reference grid
    pub show_grid: bool,
    /// Waveform display mode
    pub waveform_mode: WaveformMode,
    /// Enable color gradient
    pub use_color: bool,
}

impl Default for OscilloscopeConfig {
    fn default() -> Self {
        Self {
            sample_count: 512,
            amplitude_sensitivity: 1.5,
            smoothing_factor: 0.1, // Less smoothing for accurate waveform
            line_thickness: 3.0,   // Thicker for better visibility
            trigger_enabled: true,
            trigger_level: 0.0, // Zero-crossing trigger
            trigger_slope: TriggerSlope::Positive,
            show_grid: true,
            waveform_mode: WaveformMode::LineAndFill, // Best visibility
            use_color: true,                          // Enable color by default
        }
    }
}

impl OscilloscopeConfig {
    /// Validate configuration parameters
    ///
    /// # Returns
    /// true if configuration is valid, false otherwise
    pub fn is_valid(&self) -> bool {
        self.sample_count > 0
            && self.amplitude_sensitivity > 0.0
            && (0.0..=1.0).contains(&self.smoothing_factor)
            && self.line_thickness > 0.0
            && (-1.0..=1.0).contains(&self.trigger_level)
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
    /// Current waveform (real audio samples, not synthetic)
    waveform: Vec<f32>,
    /// Configuration
    config: OscilloscopeConfig,
    /// Beat flash effect (0.0-1.0, decays over time)
    beat_flash: f32,
    /// Color scheme for rendering
    color_scheme: super::color_schemes::ColorScheme,
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
        // Validate configuration
        assert!(
            config.is_valid(),
            "Invalid OscilloscopeConfig: sample_count must be > 0, amplitude_sensitivity > 0, smoothing 0.0-1.0, trigger_level -1.0 to 1.0"
        );

        let waveform = vec![0.0; config.sample_count];
        Self {
            waveform,
            config,
            beat_flash: 0.0,
            color_scheme: super::color_schemes::ColorScheme::new(
                super::color_schemes::ColorSchemeType::Monochrome,
            ),
        }
    }

    /// Get the current waveform (for testing/debugging)
    pub fn waveform(&self) -> &[f32] {
        &self.waveform
    }

    /// Toggle reference grid on/off
    pub fn toggle_grid(&mut self) {
        self.config.show_grid = !self.config.show_grid;
    }

    /// Toggle through waveform fill modes
    pub fn toggle_fill_mode(&mut self) {
        self.config.waveform_mode = match self.config.waveform_mode {
            WaveformMode::Line => WaveformMode::Filled,
            WaveformMode::Filled => WaveformMode::LineAndFill,
            WaveformMode::LineAndFill => WaveformMode::Line,
        };
    }

    /// Toggle through trigger slope modes
    pub fn toggle_trigger_mode(&mut self) {
        self.config.trigger_slope = match self.config.trigger_slope {
            TriggerSlope::Positive => TriggerSlope::Negative,
            TriggerSlope::Negative => TriggerSlope::Both,
            TriggerSlope::Both => TriggerSlope::Positive,
        };
    }

    /// Update the color scheme for rendering
    ///
    /// Allows changing the color scheme at runtime for different visual styles
    pub fn set_color_scheme(&mut self, color_scheme: super::color_schemes::ColorScheme) {
        self.color_scheme = color_scheme;
    }

    /// Find trigger point in waveform for stable display
    ///
    /// Searches for a point where the waveform crosses the trigger level
    /// in the specified direction. This makes periodic signals appear stable
    /// instead of scrolling.
    ///
    /// # Arguments
    /// * `waveform` - Audio waveform samples
    /// * `trigger_level` - Level to trigger on (-1.0 to 1.0)
    /// * `trigger_slope` - Edge direction to trigger on
    ///
    /// # Returns
    /// Index in waveform where trigger occurs, or 0 if no trigger found
    ///
    /// # Examples
    ///
    /// ```
    /// use crabmusic::visualization::{OscilloscopeVisualizer, OscilloscopeConfig, TriggerSlope};
    ///
    /// let viz = OscilloscopeVisualizer::new(OscilloscopeConfig::default());
    /// // Internal method, tested via update()
    /// ```
    fn find_trigger_point(
        &self,
        waveform: &[f32],
        trigger_level: f32,
        trigger_slope: TriggerSlope,
    ) -> usize {
        if waveform.len() < 2 {
            return 0;
        }

        // Search for trigger crossing in first half of waveform
        // (keep second half for display after trigger)
        let search_len = waveform.len() / 2;

        for i in 1..search_len {
            let prev = waveform[i - 1];
            let curr = waveform[i];

            let is_rising = prev < trigger_level && curr >= trigger_level;
            let is_falling = prev > trigger_level && curr <= trigger_level;

            let triggered = match trigger_slope {
                TriggerSlope::Positive => is_rising,
                TriggerSlope::Negative => is_falling,
                TriggerSlope::Both => is_rising || is_falling,
            };

            if triggered {
                return i;
            }
        }

        // No trigger found - return 0 (freerun mode)
        0
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
        // Map x to waveform sample index
        let sample_idx = (x as f32 / width as f32 * self.waveform.len() as f32) as usize;
        if sample_idx >= self.waveform.len() {
            return 0.0;
        }

        // Get REAL waveform value at this x position (-1.0 to 1.0)
        let waveform_value = self.waveform[sample_idx];

        // Convert waveform value to y position (normalized 0.0 to 1.0)
        let waveform_y = 0.5 - (waveform_value * self.config.amplitude_sensitivity * 0.45);

        // Normalize y coordinate
        let norm_y = y as f32 / height as f32;

        // Calculate distance from waveform line
        let distance = (norm_y - waveform_y).abs();

        // Convert distance to coverage based on thickness
        let half_thickness = self.config.line_thickness / height as f32 / 2.0;

        let base_coverage = if distance < half_thickness {
            1.0 // Inside the line
        } else if distance < half_thickness * 2.0 {
            // Edge anti-aliasing
            1.0 - (distance - half_thickness) / half_thickness
        } else {
            0.0 // Outside
        };

        // Apply beat flash
        let flash_boost = self.beat_flash * 0.3;
        (base_coverage + flash_boost).min(1.0)
    }
}

impl OscilloscopeVisualizer {
    /// Render just the waveform line using Braille for smooth curves!
    fn render_line(&self, grid: &mut GridBuffer, width: usize, height: usize, _center_y: usize) {
        // Create high-res Braille grid (2× width, 4× height in dots)
        let mut braille = BrailleGrid::new(width, height);
        let dot_width = braille.dot_width();
        let dot_height = braille.dot_height();
        let dot_center_y = dot_height / 2;

        // Draw the waveform using smooth lines between points
        let mut prev_x = 0;
        let mut prev_y = dot_center_y;

        for x in 0..dot_width {
            // Map to waveform sample
            let sample_idx = (x as f32 / dot_width as f32 * self.waveform.len() as f32) as usize;
            if sample_idx >= self.waveform.len() {
                break;
            }

            let waveform_value = self.waveform[sample_idx];

            // Convert to dot coordinates (with better scaling)
            let y_offset =
                waveform_value * self.config.amplitude_sensitivity * (dot_height as f32 / 2.2);
            let y = (dot_center_y as f32 - y_offset).clamp(0.0, (dot_height - 1) as f32) as usize;

            // Draw line from previous point to current point (smooth!)
            if x > 0 {
                if self.config.use_color {
                    let amplitude = waveform_value.abs();
                    let intensity = (amplitude * 0.8 + self.beat_flash * 0.2).clamp(0.0, 1.0);

                    // Use color scheme for consistent coloring across visualizers
                    let color = match self.color_scheme.get_color(intensity) {
                        Some(c) => c,
                        None => {
                            // Fallback to cyan/blue if monochrome
                            let color_val = (intensity * 200.0).min(255.0) as u8;
                            Color::new(0, color_val.saturating_add(50), color_val)
                        }
                    };
                    braille.draw_line_with_color(prev_x, prev_y, x, y, color);
                } else {
                    braille.draw_line(prev_x, prev_y, x, y);
                }
            }

            prev_x = x;
            prev_y = y;
        }

        // Convert Braille grid to regular grid
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

    /// Render filled area under waveform using Braille gradients
    fn render_filled(&self, grid: &mut GridBuffer, width: usize, height: usize, _center_y: usize) {
        // Use Braille for high-resolution fill too!
        let mut braille = BrailleGrid::new(width, height);
        let dot_width = braille.dot_width();
        let dot_height = braille.dot_height();
        let dot_center_y = dot_height / 2;

        for dot_x in 0..dot_width {
            let sample_idx =
                (dot_x as f32 / dot_width as f32 * self.waveform.len() as f32) as usize;
            if sample_idx >= self.waveform.len() {
                break;
            }

            let waveform_value = self.waveform[sample_idx];
            let y_offset =
                waveform_value * self.config.amplitude_sensitivity * (dot_height as f32 / 2.2);
            let waveform_dot_y =
                (dot_center_y as f32 - y_offset).clamp(0.0, (dot_height - 1) as f32) as usize;

            // Fill from center to waveform
            let (start_y, end_y) = if waveform_dot_y < dot_center_y {
                (waveform_dot_y, dot_center_y)
            } else {
                (dot_center_y, waveform_dot_y.min(dot_height - 1))
            };

            // Fill every other dot for gradient effect
            for dot_y in start_y..=end_y {
                // Distance-based gradient
                let distance =
                    ((dot_y as i32 - dot_center_y as i32).abs() as f32) / (dot_height as f32 / 2.0);

                // Probabilistic fill for gradient
                if (dot_y + dot_x) % 3 != 0 || distance < 0.3 {
                    if self.config.use_color {
                        let amplitude = waveform_value.abs();
                        let intensity = (amplitude * 0.4 + self.beat_flash * 0.1).clamp(0.0, 1.0);

                        // Use color scheme for fill, but dimmer than the line
                        let color = match self.color_scheme.get_color(intensity) {
                            Some(mut c) => {
                                // Dim the fill color to 30% for subtle effect
                                c.r = (c.r as f32 * 0.3) as u8;
                                c.g = (c.g as f32 * 0.3) as u8;
                                c.b = (c.b as f32 * 0.3) as u8;
                                c
                            }
                            None => {
                                // Fallback to dim cyan if monochrome
                                let color_val = (intensity * 60.0) as u8;
                                Color::new(0, color_val, color_val)
                            }
                        };
                        braille.set_dot_with_color(dot_x, dot_y, color);
                    } else {
                        braille.set_dot(dot_x, dot_y);
                    }
                }
            }
        }

        // Convert to grid
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
}

impl Visualizer for OscilloscopeVisualizer {
    fn update(&mut self, params: &AudioParameters) {
        // Use real waveform from audio parameters
        if !params.waveform.is_empty() {
            // Find trigger point for stable display
            let trigger_offset = if self.config.trigger_enabled {
                self.find_trigger_point(
                    &params.waveform,
                    self.config.trigger_level,
                    self.config.trigger_slope,
                )
            } else {
                0
            };

            // Extract waveform starting from trigger point
            let waveform_len = params.waveform.len();
            let display_len = self.config.sample_count.min(waveform_len - trigger_offset);

            // Apply gentle smoothing to reduce noise (but preserve shape)
            for i in 0..display_len {
                let source_idx = trigger_offset + i;
                let new_value = params.waveform[source_idx];

                // Smooth with previous value to reduce jitter
                let smoothing = self.config.smoothing_factor;
                if i < self.waveform.len() {
                    self.waveform[i] = lerp(self.waveform[i], new_value, 1.0 - smoothing);
                }
            }

            // Fill remaining with zeros if waveform is shorter than display
            for i in display_len..self.waveform.len() {
                self.waveform[i] = lerp(self.waveform[i], 0.0, 1.0 - self.config.smoothing_factor);
            }
        } else {
            // No waveform data - fade to zero
            for sample in &mut self.waveform {
                *sample *= 0.9;
            }
        }

        // Handle beat flash effect
        if params.beat {
            self.beat_flash = 1.0;
        } else {
            self.beat_flash *= 0.85;
        }
    }

    fn render(&self, grid: &mut GridBuffer) {
        // Clear grid first
        grid.clear();

        let width = grid.width();
        let height = grid.height();
        let center_y = height / 2;

        // Draw simplified reference grid if enabled
        if self.config.show_grid {
            // Just center line with minimal markers
            for x in (0..width).step_by(10) {
                let color = Color::new(60, 60, 60); // Dim gray
                grid.set_cell_with_color(x, center_y, '·', color);
            }
        }

        // Render waveform with different modes
        match self.config.waveform_mode {
            WaveformMode::Line => self.render_line(grid, width, height, center_y),
            WaveformMode::Filled => self.render_filled(grid, width, height, center_y),
            WaveformMode::LineAndFill => {
                self.render_filled(grid, width, height, center_y);
                self.render_line(grid, width, height, center_y);
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
        assert_eq!(viz.waveform.len(), 512);
    }

    #[test]
    fn test_config_validation() {
        // Valid config
        let valid = OscilloscopeConfig::default();
        assert!(valid.is_valid());

        // Invalid sample count
        let invalid_samples = OscilloscopeConfig {
            sample_count: 0,
            ..Default::default()
        };
        assert!(!invalid_samples.is_valid());

        // Invalid amplitude sensitivity
        let invalid_amp = OscilloscopeConfig {
            amplitude_sensitivity: 0.0,
            ..Default::default()
        };
        assert!(!invalid_amp.is_valid());

        // Invalid smoothing factor
        let invalid_smoothing = OscilloscopeConfig {
            smoothing_factor: 1.5,
            ..Default::default()
        };
        assert!(!invalid_smoothing.is_valid());

        // Invalid trigger level
        let invalid_trigger = OscilloscopeConfig {
            trigger_level: 2.0,
            ..Default::default()
        };
        assert!(!invalid_trigger.is_valid());
    }

    #[test]
    #[should_panic(expected = "Invalid OscilloscopeConfig")]
    fn test_new_with_invalid_config_panics() {
        let invalid_config = OscilloscopeConfig {
            sample_count: 0,
            ..Default::default()
        };
        OscilloscopeVisualizer::new(invalid_config);
    }

    #[test]
    fn test_trigger_detection_positive_slope() {
        let config = OscilloscopeConfig {
            trigger_enabled: true,
            trigger_level: 0.0,
            trigger_slope: TriggerSlope::Positive,
            ..Default::default()
        };
        let viz = OscilloscopeVisualizer::new(config);

        // Waveform that crosses zero upward at index 10
        let mut waveform = vec![-0.5; 512];
        for item in waveform.iter_mut().take(512).skip(10) {
            *item = 0.5;
        }

        let trigger = viz.find_trigger_point(&waveform, 0.0, TriggerSlope::Positive);
        assert_eq!(trigger, 10);
    }

    #[test]
    fn test_trigger_detection_negative_slope() {
        let config = OscilloscopeConfig {
            trigger_enabled: true,
            trigger_level: 0.0,
            trigger_slope: TriggerSlope::Negative,
            ..Default::default()
        };
        let viz = OscilloscopeVisualizer::new(config);

        // Waveform that crosses zero downward at index 20
        let mut waveform = vec![0.5; 512];
        for item in waveform.iter_mut().take(512).skip(20) {
            *item = -0.5;
        }

        let trigger = viz.find_trigger_point(&waveform, 0.0, TriggerSlope::Negative);
        assert_eq!(trigger, 20);
    }

    #[test]
    fn test_no_trigger_freerun() {
        let config = OscilloscopeConfig::default();
        let viz = OscilloscopeVisualizer::new(config);

        // Waveform that never crosses trigger level
        let waveform = vec![0.8; 512];

        let trigger = viz.find_trigger_point(&waveform, 0.0, TriggerSlope::Positive);
        assert_eq!(trigger, 0); // Freerun mode
    }

    #[test]
    fn test_real_waveform_update() {
        let mut viz = OscilloscopeVisualizer::new(OscilloscopeConfig::default());

        // Create sine wave as real waveform
        let mut waveform = Vec::new();
        for i in 0..512 {
            let t = i as f32 / 512.0;
            waveform.push((t * 2.0 * std::f32::consts::PI * 4.0).sin());
        }

        let params = AudioParameters {
            waveform,
            amplitude: 0.5,
            beat: false,
            ..Default::default()
        };

        viz.update(&params);

        // Waveform should now contain real sine wave data
        assert!(viz.waveform.iter().any(|&s| s > 0.3));
        assert!(viz.waveform.iter().any(|&s| s < -0.3));
    }

    #[test]
    fn test_oscilloscope_update_with_empty_waveform() {
        let mut viz = OscilloscopeVisualizer::new(OscilloscopeConfig::default());

        // Initial waveform has some content
        viz.waveform[0] = 0.5;

        let params = AudioParameters {
            waveform: vec![], // Empty waveform
            amplitude: 0.5,
            beat: false,
            ..Default::default()
        };

        viz.update(&params);

        // Should fade toward zero
        assert!(viz.waveform[0] < 0.5);
    }

    #[test]
    fn test_oscilloscope_render() {
        let mut viz = OscilloscopeVisualizer::new(OscilloscopeConfig::default());
        let mut grid = GridBuffer::new(80, 24);

        // Create sine wave
        let mut waveform = Vec::new();
        for i in 0..512 {
            let t = i as f32 / 512.0;
            waveform.push((t * 2.0 * std::f32::consts::PI * 4.0).sin());
        }

        let params = AudioParameters {
            waveform,
            amplitude: 0.5,
            beat: false,
            ..Default::default()
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
    fn test_beat_flash_effect() {
        let mut viz = OscilloscopeVisualizer::new(OscilloscopeConfig::default());

        // No beat
        let params_no_beat = AudioParameters {
            waveform: vec![0.5; 512],
            beat: false,
            ..Default::default()
        };
        viz.update(&params_no_beat);
        assert_eq!(viz.beat_flash, 0.0);

        // Beat detected
        let params_beat = AudioParameters {
            waveform: vec![0.5; 512],
            beat: true,
            ..Default::default()
        };
        viz.update(&params_beat);
        assert_eq!(viz.beat_flash, 1.0);

        // Flash decays
        viz.update(&params_no_beat);
        assert!(viz.beat_flash < 1.0 && viz.beat_flash > 0.0);
    }
}
