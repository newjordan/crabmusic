//! Spectrogram Visualizer
//!
//! Displays frequency content over time as a scrolling waterfall display.
//! Each row represents a moment in time, each column represents a frequency bin,
//! and color intensity represents amplitude.

use crate::dsp::AudioParameters;
use crate::visualization::color_schemes::ColorScheme;
use crate::visualization::{GridBuffer, Visualizer};
use std::collections::VecDeque;

/// Scroll direction for spectrogram
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirection {
    /// New data appears at bottom, scrolls up (oldest at top)
    Up,
    /// New data appears at top, scrolls down (oldest at bottom)
    Down,
}

/// Spectrogram visualizer that displays frequency content over time
///
/// Creates a scrolling waterfall display where:
/// - Each row represents a moment in time
/// - Each column represents a frequency bin
/// - Color intensity represents amplitude/energy
/// - Display scrolls continuously (waterfall effect)
///
/// # Examples
///
/// ```
/// use crabmusic::visualization::spectrogram::{SpectrogramVisualizer, ScrollDirection};
/// use crabmusic::visualization::ColorScheme;
///
/// let color_scheme = ColorScheme::new_rainbow();
/// let visualizer = SpectrogramVisualizer::new(color_scheme, ScrollDirection::Up);
/// ```
pub struct SpectrogramVisualizer {
    /// Color scheme for mapping intensity to colors
    color_scheme: ColorScheme,
    /// Circular buffer of spectrum snapshots (history)
    history_buffer: VecDeque<Vec<f32>>,
    /// Maximum number of history rows to keep
    max_history: usize,
    /// Scroll direction (up or down)
    scroll_direction: ScrollDirection,
}

impl SpectrogramVisualizer {
    /// Create a new spectrogram visualizer
    ///
    /// # Arguments
    /// * `color_scheme` - Color scheme for mapping intensity to colors
    /// * `scroll_direction` - Direction to scroll (up or down)
    ///
    /// # Returns
    /// A new SpectrogramVisualizer
    ///
    /// # Examples
    ///
    /// ```
    /// use crabmusic::visualization::spectrogram::{SpectrogramVisualizer, ScrollDirection};
    /// use crabmusic::visualization::ColorScheme;
    ///
    /// let color_scheme = ColorScheme::new_rainbow();
    /// let visualizer = SpectrogramVisualizer::new(color_scheme, ScrollDirection::Up);
    /// ```
    pub fn new(color_scheme: ColorScheme, scroll_direction: ScrollDirection) -> Self {
        Self {
            color_scheme,
            history_buffer: VecDeque::new(),
            max_history: 100, // Will be adjusted based on grid height
            scroll_direction,
        }
    }

    /// Get the current scroll direction
    pub fn scroll_direction(&self) -> ScrollDirection {
        self.scroll_direction
    }

    /// Set the scroll direction
    pub fn set_scroll_direction(&mut self, direction: ScrollDirection) {
        self.scroll_direction = direction;
    }

    /// Toggle scroll direction
    pub fn toggle_scroll_direction(&mut self) {
        self.scroll_direction = match self.scroll_direction {
            ScrollDirection::Up => ScrollDirection::Down,
            ScrollDirection::Down => ScrollDirection::Up,
        };
    }
}

impl Visualizer for SpectrogramVisualizer {
    fn update(&mut self, params: &AudioParameters) {
        // Get frequency spectrum from audio parameters
        let spectrum = &params.spectrum;

        // Add new spectrum to history buffer
        self.history_buffer.push_back(spectrum.clone());

        // Remove oldest if we exceed max history
        while self.history_buffer.len() > self.max_history {
            self.history_buffer.pop_front();
        }
    }

    fn render(&self, grid: &mut GridBuffer) {
        let width = grid.width();
        let height = grid.height();

        // Adjust max_history based on grid height (use Braille 4x vertical resolution)
        let effective_height = height * 4;

        // Clear grid
        grid.clear();

        // If no history, nothing to render
        if self.history_buffer.is_empty() {
            return;
        }

        // Determine how many history rows to display
        let num_rows = self.history_buffer.len().min(effective_height);

        // Render each row of history
        for row_idx in 0..num_rows {
            // Get spectrum for this row based on scroll direction
            let spectrum = match self.scroll_direction {
                ScrollDirection::Up => {
                    // Oldest at top, newest at bottom
                    let history_idx = self.history_buffer.len().saturating_sub(num_rows) + row_idx;
                    &self.history_buffer[history_idx]
                }
                ScrollDirection::Down => {
                    // Newest at top, oldest at bottom
                    let history_idx = self.history_buffer.len().saturating_sub(row_idx + 1);
                    &self.history_buffer[history_idx]
                }
            };

            // Guard: skip rendering if this spectrum row is empty to avoid index errors
            if spectrum.is_empty() {
                continue;
            }

            // Calculate y position
            let y = row_idx;

            // Skip if out of bounds
            if y >= height {
                continue;
            }

            // Render this row across the width
            for x in 0..width {
                // Map x position to frequency bin
                let bin_idx = (x * spectrum.len()) / width;
                let bin_idx = bin_idx.min(spectrum.len().saturating_sub(1));

                // Get amplitude for this bin
                let amplitude = spectrum[bin_idx];

                // Map amplitude to color using color scheme
                let color = self.color_scheme.get_color(amplitude);

                // Use full block character for solid appearance
                // Choose character based on amplitude for better visual density
                let character = if amplitude > 0.7 {
                    '█' // Full block for high amplitude
                } else if amplitude > 0.4 {
                    '▓' // Dark shade for medium amplitude
                } else if amplitude > 0.2 {
                    '▒' // Medium shade for low amplitude
                } else if amplitude > 0.05 {
                    '░' // Light shade for very low amplitude
                } else {
                    ' ' // Empty for silence
                };

                // Set cell with color and character
                let cell = grid.get_cell_mut(x, y);
                cell.character = character;
                cell.foreground_color = color;
            }
        }
    }

    fn name(&self) -> &str {
        "Spectrogram"
    }
}

impl SpectrogramVisualizer {
    /// Update the color scheme for rendering
    pub fn set_color_scheme(&mut self, color_scheme: ColorScheme) {
        self.color_scheme = color_scheme;
    }

    /// Get the current color scheme
    pub fn color_scheme(&self) -> &ColorScheme {
        &self.color_scheme
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visualization::color_schemes::ColorSchemeType;

    #[test]
    fn test_spectrogram_new() {
        let color_scheme = ColorScheme::new(ColorSchemeType::Monochrome);
        let visualizer = SpectrogramVisualizer::new(color_scheme, ScrollDirection::Up);
        assert_eq!(visualizer.scroll_direction(), ScrollDirection::Up);
        assert_eq!(visualizer.name(), "Spectrogram");
    }

    #[test]
    fn test_scroll_direction_toggle() {
        let color_scheme = ColorScheme::new(ColorSchemeType::Monochrome);
        let mut visualizer = SpectrogramVisualizer::new(color_scheme, ScrollDirection::Up);

        visualizer.toggle_scroll_direction();
        assert_eq!(visualizer.scroll_direction(), ScrollDirection::Down);

        visualizer.toggle_scroll_direction();
        assert_eq!(visualizer.scroll_direction(), ScrollDirection::Up);
    }

    #[test]
    fn test_history_buffer_limit() {
        let color_scheme = ColorScheme::new(ColorSchemeType::Monochrome);
        let mut visualizer = SpectrogramVisualizer::new(color_scheme, ScrollDirection::Up);
        visualizer.max_history = 10;

        let params = AudioParameters::default();

        // Add more than max_history frames
        for _ in 0..20 {
            visualizer.update(&params);
        }

        // Should be limited to max_history
        assert_eq!(visualizer.history_buffer.len(), 10);
    }

    #[test]
    fn test_set_color_scheme() {
        let color_scheme = ColorScheme::new(ColorSchemeType::Monochrome);
        let mut visualizer = SpectrogramVisualizer::new(color_scheme, ScrollDirection::Up);

        let new_scheme = ColorScheme::new(ColorSchemeType::Rainbow);
        visualizer.set_color_scheme(new_scheme);

        // Verify color scheme was updated (check scheme type)
        assert_eq!(visualizer.color_scheme().scheme_type().name(), "Rainbow");
    }
}
