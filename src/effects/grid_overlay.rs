//! Grid overlay effect for testing and debugging
//!
//! Draws a grid pattern over the visualization to help with alignment,
//! testing, and visual debugging of the effect pipeline.

use super::Effect;
use crate::dsp::AudioParameters;
use crate::visualization::{Color, GridBuffer};

/// Grid overlay effect that draws a test pattern
///
/// Overlays a grid of lines on top of the visualization.
/// Useful for:
/// - Verifying effects are being applied
/// - Testing effect pipeline integration
/// - Visual debugging and alignment
/// - Demonstrating effect composability
///
/// # Examples
///
/// ```
/// use crabmusic::effects::{Effect, grid_overlay::GridOverlayEffect};
/// use crabmusic::visualization::GridBuffer;
/// use crabmusic::dsp::AudioParameters;
///
/// let mut effect = GridOverlayEffect::new(10);  // Grid lines every 10 cells
/// let mut grid = GridBuffer::new(80, 24);
/// let params = AudioParameters::default();
///
/// effect.apply(&mut grid, &params);
/// // Grid now has overlay lines
/// ```
#[derive(Debug, Clone)]
pub struct GridOverlayEffect {
    enabled: bool,
    intensity: f32,
    spacing: usize,
}

impl GridOverlayEffect {
    /// Create a new grid overlay effect
    ///
    /// # Arguments
    /// * `spacing` - Distance between grid lines in cells
    ///
    /// # Examples
    ///
    /// ```
    /// use crabmusic::effects::grid_overlay::GridOverlayEffect;
    ///
    /// let effect = GridOverlayEffect::new(10);
    /// assert!(effect.is_enabled());
    /// assert_eq!(effect.intensity(), 1.0);
    /// assert_eq!(effect.spacing(), 10);
    /// ```
    pub fn new(spacing: usize) -> Self {
        Self {
            enabled: true,
            intensity: 1.0,
            spacing: spacing.max(1), // Ensure spacing is at least 1
        }
    }

    /// Get the grid spacing
    pub fn spacing(&self) -> usize {
        self.spacing
    }

    /// Set the grid spacing
    ///
    /// # Arguments
    /// * `spacing` - Distance between grid lines (minimum 1)
    pub fn set_spacing(&mut self, spacing: usize) {
        self.spacing = spacing.max(1);
    }
}

impl Default for GridOverlayEffect {
    fn default() -> Self {
        Self::new(10)
    }
}

impl Effect for GridOverlayEffect {
    fn apply(&mut self, grid: &mut GridBuffer, _params: &AudioParameters) {
        if !self.enabled {
            return;
        }

        // Calculate color based on intensity
        let color_val = (50.0 * self.intensity) as u8;
        let color = Color::new(color_val, color_val, color_val);

        let width = grid.width();
        let height = grid.height();

        // Draw vertical lines
        for x in (0..width).step_by(self.spacing) {
            for y in 0..height {
                grid.set_cell_with_color(x, y, '│', color);
            }
        }

        // Draw horizontal lines
        for y in (0..height).step_by(self.spacing) {
            for x in 0..width {
                grid.set_cell_with_color(x, y, '─', color);
            }
        }

        // Draw intersections
        for y in (0..height).step_by(self.spacing) {
            for x in (0..width).step_by(self.spacing) {
                grid.set_cell_with_color(x, y, '┼', color);
            }
        }
    }

    fn name(&self) -> &str {
        "GridOverlay"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn intensity(&self) -> f32 {
        self.intensity
    }

    fn set_intensity(&mut self, intensity: f32) {
        self.intensity = intensity.clamp(0.0, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_overlay_new() {
        let effect = GridOverlayEffect::new(10);
        assert!(effect.is_enabled());
        assert_eq!(effect.intensity(), 1.0);
        assert_eq!(effect.spacing(), 10);
        assert_eq!(effect.name(), "GridOverlay");
    }

    #[test]
    fn test_grid_overlay_spacing() {
        let mut effect = GridOverlayEffect::new(10);
        assert_eq!(effect.spacing(), 10);

        effect.set_spacing(5);
        assert_eq!(effect.spacing(), 5);

        // Test minimum spacing
        effect.set_spacing(0);
        assert_eq!(effect.spacing(), 1);
    }

    #[test]
    fn test_grid_overlay_enable_disable() {
        let mut effect = GridOverlayEffect::new(10);
        assert!(effect.is_enabled());

        effect.set_enabled(false);
        assert!(!effect.is_enabled());

        effect.set_enabled(true);
        assert!(effect.is_enabled());
    }

    #[test]
    fn test_grid_overlay_intensity() {
        let mut effect = GridOverlayEffect::new(10);
        assert_eq!(effect.intensity(), 1.0);

        effect.set_intensity(0.5);
        assert_eq!(effect.intensity(), 0.5);

        // Test clamping
        effect.set_intensity(1.5);
        assert_eq!(effect.intensity(), 1.0);

        effect.set_intensity(-0.5);
        assert_eq!(effect.intensity(), 0.0);
    }

    #[test]
    fn test_grid_overlay_apply() {
        let mut effect = GridOverlayEffect::new(5);
        let mut grid = GridBuffer::new(10, 10);
        let params = AudioParameters::default();

        // Apply effect
        effect.apply(&mut grid, &params);

        // Check that grid lines were drawn
        // Vertical line at x=0
        assert_eq!(grid.get_cell(0, 1).character, '│');
        // Horizontal line at y=0
        assert_eq!(grid.get_cell(1, 0).character, '─');
        // Intersection at (0, 0)
        assert_eq!(grid.get_cell(0, 0).character, '┼');
        // Intersection at (5, 5)
        assert_eq!(grid.get_cell(5, 5).character, '┼');
    }

    #[test]
    fn test_grid_overlay_disabled() {
        let mut effect = GridOverlayEffect::new(5);
        effect.set_enabled(false);

        let mut grid = GridBuffer::new(10, 10);
        let params = AudioParameters::default();

        // Apply effect (should do nothing when disabled)
        effect.apply(&mut grid, &params);

        // Grid should be empty
        assert_eq!(grid.get_cell(0, 0).character, ' ');
        assert_eq!(grid.get_cell(5, 5).character, ' ');
    }
}

