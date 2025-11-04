// XY Oscilloscope visualizer (Lissajous Curves)
// Plots left channel vs right channel for stereo visualization

use super::{BrailleGrid, Color, GridBuffer, Visualizer};
use crate::dsp::AudioParameters;

/// Display mode for XY oscilloscope
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum XYDisplayMode {
    /// Draw dots only
    Dots,
    /// Draw connected lines
    Lines,
    /// Draw lines with persistence trails
    LinesWithTrails,
}

/// Configuration for XY oscilloscope visualizer
#[derive(Debug, Clone)]
pub struct XYOscilloscopeConfig {
    /// Number of samples to display
    pub sample_count: usize,
    /// Amplitude sensitivity multiplier
    pub sensitivity: f32,
    /// Persistence/trail decay rate (0.0-1.0, higher = longer trails)
    pub persistence: f32,
    /// Rotation angle in radians
    pub rotation: f32,
    /// Zoom factor (1.0 = normal, >1.0 = zoomed in)
    pub zoom: f32,
    /// Display mode
    pub display_mode: XYDisplayMode,
    /// Show reference grid
    pub show_grid: bool,
    /// Show center crosshair
    pub show_crosshair: bool,
    /// Enable color gradient based on amplitude
    pub use_color: bool,
    /// Dot size for rendering
    pub dot_size: f32,
}

impl Default for XYOscilloscopeConfig {
    fn default() -> Self {
        Self {
            sample_count: 512,
            sensitivity: 1.0,
            persistence: 0.85,  // 85% persistence = nice trails
            rotation: 0.0,
            zoom: 1.0,
            display_mode: XYDisplayMode::Lines,  // Temporarily use simple lines for debugging
            show_grid: false,        // Disabled by default - cleaner look
            show_crosshair: false,   // Disabled by default - cleaner look
            use_color: true,
            dot_size: 2.0,
        }
    }
}

impl XYOscilloscopeConfig {
    /// Validate configuration parameters
    pub fn is_valid(&self) -> bool {
        self.sample_count > 0
            && self.sensitivity > 0.0
            && (0.0..=1.0).contains(&self.persistence)
            && self.zoom > 0.0
            && self.dot_size > 0.0
    }
}

/// XY Oscilloscope visualizer for Lissajous curves
///
/// Plots left channel (X axis) vs right channel (Y axis) to create
/// beautiful geometric patterns that reveal phase relationships and
/// stereo imaging in audio.
///
/// # Lissajous Patterns
/// - **Mono audio**: Diagonal line (L = R)
/// - **Stereo with phase shift**: Ellipses, circles
/// - **Complex stereo**: Intricate geometric patterns
/// - **Out of phase**: Figure-8 patterns
///
/// # Examples
///
/// ```
/// use crabmusic::visualization::{XYOscilloscopeVisualizer, XYOscilloscopeConfig, Visualizer, GridBuffer};
/// use crabmusic::dsp::AudioParameters;
///
/// let mut viz = XYOscilloscopeVisualizer::new(XYOscilloscopeConfig::default());
/// let mut grid = GridBuffer::new(80, 24);
/// let params = AudioParameters::default();
///
/// viz.update(&params);
/// viz.render(&mut grid);
/// ```
pub struct XYOscilloscopeVisualizer {
    /// Left channel samples (X axis)
    left_channel: Vec<f32>,
    /// Right channel samples (Y axis)
    right_channel: Vec<f32>,
    /// Persistence buffer for trail effect (stores previous frames)
    persistence_buffer: Vec<Vec<(f32, f32)>>,
    /// Configuration
    config: XYOscilloscopeConfig,
    /// Beat flash effect (0.0-1.0, decays over time)
    beat_flash: f32,
    /// Color scheme for rendering
    color_scheme: super::color_schemes::ColorScheme,
}

impl XYOscilloscopeVisualizer {
    /// Create a new XY oscilloscope visualizer
    pub fn new(config: XYOscilloscopeConfig) -> Self {
        assert!(
            config.is_valid(),
            "Invalid XYOscilloscopeConfig: sample_count must be > 0, sensitivity > 0, persistence 0.0-1.0, zoom > 0, dot_size > 0"
        );

        let left_channel = vec![0.0; config.sample_count];
        let right_channel = vec![0.0; config.sample_count];
        let persistence_buffer = Vec::new();

        Self {
            left_channel,
            right_channel,
            persistence_buffer,
            config,
            beat_flash: 0.0,
            color_scheme: super::color_schemes::ColorScheme::new(
                super::color_schemes::ColorSchemeType::CyanMagenta,
            ),
        }
    }

    /// Toggle reference grid on/off
    pub fn toggle_grid(&mut self) {
        self.config.show_grid = !self.config.show_grid;
    }

    /// Toggle crosshair on/off
    pub fn toggle_crosshair(&mut self) {
        self.config.show_crosshair = !self.config.show_crosshair;
    }

    /// Cycle through display modes
    pub fn cycle_display_mode(&mut self) {
        self.config.display_mode = match self.config.display_mode {
            XYDisplayMode::Dots => XYDisplayMode::Lines,
            XYDisplayMode::Lines => XYDisplayMode::LinesWithTrails,
            XYDisplayMode::LinesWithTrails => XYDisplayMode::Dots,
        };
    }

    /// Increase persistence (longer trails)
    pub fn increase_persistence(&mut self) {
        self.config.persistence = (self.config.persistence + 0.05).min(0.99);
    }

    /// Decrease persistence (shorter trails)
    pub fn decrease_persistence(&mut self) {
        self.config.persistence = (self.config.persistence - 0.05).max(0.0);
    }

    /// Increase zoom
    pub fn increase_zoom(&mut self) {
        self.config.zoom = (self.config.zoom * 1.1).min(5.0);
    }

    /// Decrease zoom
    pub fn decrease_zoom(&mut self) {
        self.config.zoom = (self.config.zoom / 1.1).max(0.2);
    }

    /// Rotate clockwise
    pub fn rotate_clockwise(&mut self) {
        self.config.rotation += std::f32::consts::PI / 12.0; // 15 degrees
        if self.config.rotation > std::f32::consts::PI * 2.0 {
            self.config.rotation -= std::f32::consts::PI * 2.0;
        }
    }

    /// Rotate counter-clockwise
    pub fn rotate_counterclockwise(&mut self) {
        self.config.rotation -= std::f32::consts::PI / 12.0; // 15 degrees
        if self.config.rotation < 0.0 {
            self.config.rotation += std::f32::consts::PI * 2.0;
        }
    }

    /// Update the color scheme for rendering
    pub fn set_color_scheme(&mut self, color_scheme: super::color_schemes::ColorScheme) {
        self.color_scheme = color_scheme;
    }

    /// Transform XY coordinates with rotation and zoom
    fn transform_point(&self, x: f32, y: f32) -> (f32, f32) {
        // Apply zoom
        let x = x * self.config.zoom;
        let y = y * self.config.zoom;

        // Apply rotation
        let cos_r = self.config.rotation.cos();
        let sin_r = self.config.rotation.sin();
        let x_rot = x * cos_r - y * sin_r;
        let y_rot = x * sin_r + y * cos_r;

        (x_rot, y_rot)
    }

    /// Render reference grid
    fn render_grid(&self, braille: &mut BrailleGrid) {
        let dot_width = braille.dot_width();
        let dot_height = braille.dot_height();
        let center_x = dot_width as f32 / 2.0;
        let center_y = dot_height as f32 / 2.0;

        // Very subtle grid - only draw sparse dots, not solid lines
        let grid_color = Color::new(20, 20, 25); // Very dark blue-gray
        let spacing = 8; // Draw a dot every 8 pixels

        // Vertical grid lines (sparse dots)
        for i in 0..=4 {
            let x = (i as f32 / 4.0) * dot_width as f32;
            let x_pos = x as usize;
            let mut y = 0;
            while y < dot_height {
                if x_pos < dot_width {
                    braille.set_dot_with_color(x_pos, y, grid_color);
                }
                y += spacing;
            }
        }

        // Horizontal grid lines (sparse dots)
        for i in 0..=4 {
            let y = (i as f32 / 4.0) * dot_height as f32;
            let y_pos = y as usize;
            let mut x = 0;
            while x < dot_width {
                if y_pos < dot_height {
                    braille.set_dot_with_color(x, y_pos, grid_color);
                }
                x += spacing;
            }
        }

        // Center crosshair (slightly brighter, also sparse)
        if self.config.show_crosshair {
            let crosshair_color = Color::new(40, 40, 50); // Subtle
            let crosshair_spacing = 4; // Denser than grid but still sparse

            // Vertical crosshair
            let mut y = 0;
            while y < dot_height {
                braille.set_dot_with_color(center_x as usize, y, crosshair_color);
                y += crosshair_spacing;
            }

            // Horizontal crosshair
            let mut x = 0;
            while x < dot_width {
                braille.set_dot_with_color(x, center_y as usize, crosshair_color);
                x += crosshair_spacing;
            }
        }
    }

    /// Render XY plot with current samples
    fn render_xy_plot(&self, braille: &mut BrailleGrid) {
        let dot_width = braille.dot_width();
        let dot_height = braille.dot_height();
        let center_x = dot_width as f32 / 2.0;
        let center_y = dot_height as f32 / 2.0;

        static mut PLOT_LOG: u32 = 0;
        unsafe {
            PLOT_LOG += 1;
            if PLOT_LOG == 1 {
                eprintln!("XY: render_xy_plot - left_channel.len()={}, right_channel.len()={}",
                    self.left_channel.len(), self.right_channel.len());
                if self.left_channel.len() > 0 {
                    eprintln!("  left[0]={:.3}, right[0]={:.3}", self.left_channel[0], self.right_channel[0]);
                }
            }
        }

        // Collect XY points
        let mut points: Vec<(f32, f32)> = Vec::new();
        let mut clipped_count = 0;
        let mut total_count = 0;

        for i in 0..self.left_channel.len().min(self.right_channel.len()) {
            let x = self.left_channel[i] * self.config.sensitivity;
            let y = self.right_channel[i] * self.config.sensitivity;

            // Transform (rotate, zoom)
            let (x_transformed, y_transformed) = self.transform_point(x, y);

            // Convert to screen coordinates
            let screen_x = center_x + x_transformed * center_x * 0.9;
            let screen_y = center_y - y_transformed * center_y * 0.9; // Flip Y

            total_count += 1;

            // Check if point is within bounds
            if screen_x >= 0.0 && screen_x < dot_width as f32 && screen_y >= 0.0 && screen_y < dot_height as f32 {
                points.push((screen_x, screen_y));
            } else {
                clipped_count += 1;
            }
        }

        // Render based on display mode
        static mut RENDER_LOG: u32 = 0;
        unsafe {
            RENDER_LOG += 1;
            if RENDER_LOG == 1 {
                eprintln!("XY: render called, total={}, clipped={}, visible_points={}, mode={:?}",
                    total_count, clipped_count, points.len(), self.config.display_mode);
                if points.len() > 0 {
                    eprintln!("  First point: ({:.1}, {:.1})", points[0].0, points[0].1);
                }
            }
        }

        match self.config.display_mode {
            XYDisplayMode::Dots => self.render_dots(braille, &points),
            XYDisplayMode::Lines => self.render_lines(braille, &points),
            XYDisplayMode::LinesWithTrails => self.render_lines_with_trails(braille, &points),
        }
    }

    /// Render as individual dots
    fn render_dots(&self, braille: &mut BrailleGrid, points: &[(f32, f32)]) {
        for (i, &(x, y)) in points.iter().enumerate() {
            if x >= 0.0 && y >= 0.0 && x < braille.dot_width() as f32 && y < braille.dot_height() as f32 {
                let intensity = i as f32 / points.len() as f32;
                let color = if self.config.use_color {
                    self.color_scheme.get_color(intensity).unwrap_or(Color::new(255, 255, 255))
                } else {
                    Color::new(255, 255, 255)
                };

                braille.set_dot_with_color(x as usize, y as usize, color);
            }
        }
    }

    /// Render as connected lines
    fn render_lines(&self, braille: &mut BrailleGrid, points: &[(f32, f32)]) {
        static mut LINE_LOG: u32 = 0;
        unsafe {
            LINE_LOG += 1;
            if LINE_LOG == 1 {
                eprintln!("XY: render_lines called with {} points", points.len());
                if points.len() >= 2 {
                    eprintln!("  First: ({:.1}, {:.1}) -> ({:.1}, {:.1})",
                        points[0].0, points[0].1, points[1].0, points[1].1);
                }
            }
        }

        for i in 1..points.len() {
            let (x0, y0) = points[i - 1];
            let (x1, y1) = points[i];

            let intensity = i as f32 / points.len() as f32;
            let color = if self.config.use_color {
                self.color_scheme.get_color(intensity).unwrap_or(Color::new(255, 255, 255))
            } else {
                Color::new(255, 255, 255)
            };

            // Draw with integer pixel lines (non-AA)
            let dot_w = braille.dot_width() as f32;
            let dot_h = braille.dot_height() as f32;
            let xi0 = x0.round().clamp(0.0, dot_w - 1.0) as usize;
            let yi0 = y0.round().clamp(0.0, dot_h - 1.0) as usize;
            let xi1 = x1.round().clamp(0.0, dot_w - 1.0) as usize;
            let yi1 = y1.round().clamp(0.0, dot_h - 1.0) as usize;
            braille.draw_line_with_color(xi0, yi0, xi1, yi1, color);
        }
    }

    /// Render lines with persistence trails
    fn render_lines_with_trails(&self, braille: &mut BrailleGrid, points: &[(f32, f32)]) {
        let dot_width = braille.dot_width();
        let dot_height = braille.dot_height();
        let center_x = dot_width as f32 / 2.0;
        let center_y = dot_height as f32 / 2.0;

        // Render old frames with fading
        for (frame_age, old_points) in self.persistence_buffer.iter().enumerate() {
            let fade = self.config.persistence.powi(frame_age as i32 + 1);

            for i in 1..old_points.len() {
                // Old points are in transformed space (-1 to 1), need to convert to screen coords
                let (x0_t, y0_t) = old_points[i - 1];
                let (x1_t, y1_t) = old_points[i];

                // Convert to screen coordinates
                let x0 = center_x + x0_t * center_x * 0.9;
                let y0 = center_y - y0_t * center_y * 0.9;
                let x1 = center_x + x1_t * center_x * 0.9;
                let y1 = center_y - y1_t * center_y * 0.9;

                let intensity = (i as f32 / old_points.len() as f32) * fade;
                let base_color = if self.config.use_color {
                    self.color_scheme.get_color(intensity).unwrap_or(Color::new(255, 255, 255))
                } else {
                    Color::new(255, 255, 255)
                };

                // Fade color
                let color = Color::new(
                    (base_color.r as f32 * fade) as u8,
                    (base_color.g as f32 * fade) as u8,
                    (base_color.b as f32 * fade) as u8,
                );

                // Draw with integer pixel lines (non-AA)
                let xi0 = x0.round().clamp(0.0, dot_width as f32 - 1.0) as usize;
                let yi0 = y0.round().clamp(0.0, dot_height as f32 - 1.0) as usize;
                let xi1 = x1.round().clamp(0.0, dot_width as f32 - 1.0) as usize;
                let yi1 = y1.round().clamp(0.0, dot_height as f32 - 1.0) as usize;
                braille.draw_line_with_color(xi0, yi0, xi1, yi1, color);
            }
        }

        // Render current frame (brightest)
        self.render_lines(braille, points);
    }
}

impl Visualizer for XYOscilloscopeVisualizer {
    fn name(&self) -> &str {
        "XY Oscilloscope (Lissajous)"
    }

    fn update(&mut self, params: &AudioParameters) {
        // Update left and right channels from stereo waveforms
        let left_len = params.waveform_left.len().min(self.config.sample_count);
        let right_len = params.waveform_right.len().min(self.config.sample_count);
        let len = left_len.min(right_len);

        // Copy samples
        for i in 0..len {
            self.left_channel[i] = params.waveform_left[i];
            self.right_channel[i] = params.waveform_right[i];
        }

        // Fill remaining with zeros if needed
        for i in len..self.config.sample_count {
            self.left_channel[i] = 0.0;
            self.right_channel[i] = 0.0;
        }

        // Update beat flash
        if params.beat {
            self.beat_flash = 1.0;
        } else {
            self.beat_flash *= 0.9; // Decay
        }

        // Update persistence buffer for trails
        if self.config.display_mode == XYDisplayMode::LinesWithTrails {
            // Collect current points
            let mut current_points = Vec::new();
            for i in 0..len {
                let x = self.left_channel[i] * self.config.sensitivity;
                let y = self.right_channel[i] * self.config.sensitivity;
                let (x_t, y_t) = self.transform_point(x, y);
                current_points.push((x_t, y_t));
            }

            // Add to persistence buffer
            self.persistence_buffer.insert(0, current_points);

            // Limit buffer size (keep last N frames)
            let max_frames = 10;
            if self.persistence_buffer.len() > max_frames {
                self.persistence_buffer.truncate(max_frames);
            }
        } else {
            // Clear persistence buffer if not using trails
            self.persistence_buffer.clear();
        }
    }

    fn render(&self, grid: &mut GridBuffer) {
        let width = grid.width();
        let height = grid.height();

        // Create high-res Braille grid
        let mut braille = BrailleGrid::new(width, height);

        // Render grid if enabled
        if self.config.show_grid {
            self.render_grid(&mut braille);
        }

        // Render XY plot
        self.render_xy_plot(&mut braille);

        // Convert Braille to grid
        for y in 0..height {
            for x in 0..width {
                let ch = braille.get_char(x, y);
                let color = braille.get_color(x, y).unwrap_or(Color::new(255, 255, 255));
                grid.set_cell_with_color(x, y, ch, color);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xy_oscilloscope_creation() {
        let viz = XYOscilloscopeVisualizer::new(XYOscilloscopeConfig::default());
        assert_eq!(viz.name(), "XY Oscilloscope (Lissajous)");
    }

    #[test]
    fn test_config_validation() {
        let config = XYOscilloscopeConfig::default();
        assert!(config.is_valid());

        let mut invalid_config = config.clone();
        invalid_config.sample_count = 0;
        assert!(!invalid_config.is_valid());
    }

    #[test]
    fn test_transform_point() {
        let viz = XYOscilloscopeVisualizer::new(XYOscilloscopeConfig::default());

        // No rotation, no zoom
        let (x, y) = viz.transform_point(1.0, 0.0);
        assert!((x - 1.0).abs() < 0.001);
        assert!(y.abs() < 0.001);
    }

    #[test]
    fn test_zoom() {
        let mut viz = XYOscilloscopeVisualizer::new(XYOscilloscopeConfig::default());

        viz.increase_zoom();
        assert!(viz.config.zoom > 1.0);

        viz.decrease_zoom();
        viz.decrease_zoom();
        assert!(viz.config.zoom < 1.0);
    }

    #[test]
    fn test_persistence() {
        let mut viz = XYOscilloscopeVisualizer::new(XYOscilloscopeConfig::default());
        let initial = viz.config.persistence;

        viz.increase_persistence();
        assert!(viz.config.persistence > initial);

        viz.decrease_persistence();
        viz.decrease_persistence();
        assert!(viz.config.persistence < initial);
    }

    #[test]
    fn test_display_mode_cycling() {
        let mut viz = XYOscilloscopeVisualizer::new(XYOscilloscopeConfig::default());

        let initial = viz.config.display_mode;
        viz.cycle_display_mode();
        assert_ne!(viz.config.display_mode, initial);
    }
}

