//! Scanline Effect
//!
//! Implements CRT-style horizontal scanlines that create an authentic retro aesthetic
//! by adding alternating dark lines across the display.

use crate::dsp::AudioParameters;
use crate::effects::Effect;
use crate::visualization::{Color, GridBuffer};

/// Scanline effect that simulates CRT monitor scan lines
///
/// This effect adds horizontal dark lines at regular intervals to simulate
/// the visible scan lines on vintage CRT monitors. The effect dims existing
/// content without replacing it, creating an authentic retro look.
///
/// # Examples
///
/// ```
/// use crabmusic::effects::scanline::ScanlineEffect;
///
/// // Create scanlines every 2 rows with 50% intensity
/// let mut effect = ScanlineEffect::new(2);
/// effect.set_intensity(0.5);
/// ```
#[derive(Debug, Clone)]
pub struct ScanlineEffect {
    /// Whether the effect is enabled
    enabled: bool,
    /// Effect intensity (0.0 = invisible, 1.0 = maximum darkness)
    intensity: f32,
    /// Draw scanline every N rows (e.g., 2 = every other row)
    spacing: usize,
}

impl ScanlineEffect {
    /// Create a new scanline effect with the specified spacing
    ///
    /// # Arguments
    /// * `spacing` - Draw scanline every N rows (minimum 1)
    ///
    /// # Returns
    /// A new ScanlineEffect with default intensity (0.5) and enabled state (true)
    ///
    /// # Examples
    ///
    /// ```
    /// use crabmusic::effects::scanline::ScanlineEffect;
    ///
    /// let effect = ScanlineEffect::new(2); // Scanline every 2 rows
    /// ```
    pub fn new(spacing: usize) -> Self {
        Self {
            enabled: true,
            intensity: 0.5, // Default to 50% intensity
            spacing: spacing.max(1), // Minimum spacing of 1
        }
    }

    /// Get the current scanline spacing
    ///
    /// # Returns
    /// The number of rows between scanlines
    pub fn spacing(&self) -> usize {
        self.spacing
    }

    /// Set the scanline spacing
    ///
    /// # Arguments
    /// * `spacing` - Draw scanline every N rows (minimum 1)
    pub fn set_spacing(&mut self, spacing: usize) {
        self.spacing = spacing.max(1);
    }
}

impl Effect for ScanlineEffect {
    /// Apply the scanline effect to the grid buffer
    ///
    /// This method dims every Nth row (where N is the spacing) by reducing
    /// the brightness of the foreground color. The dimming is proportional
    /// to the intensity setting, with a maximum of 50% darkness to prevent
    /// complete blackout.
    ///
    /// # Arguments
    /// * `grid` - The grid buffer to modify
    /// * `_params` - Audio parameters (unused for static scanlines)
    fn apply(&mut self, grid: &mut GridBuffer, _params: &AudioParameters) {
        if !self.enabled {
            return;
        }

        // Iterate through scanline rows
        for y in (0..grid.height()).step_by(self.spacing) {
            for x in 0..grid.width() {
                let cell = grid.get_cell_mut(x, y);
                
                // Only dim cells that have a foreground color
                if let Some(color) = cell.foreground_color {
                    // Reduce RGB values by intensity factor
                    // Max 50% dimming to prevent complete blackout
                    let factor = 1.0 - (self.intensity * 0.5);
                    cell.foreground_color = Some(Color::new(
                        (color.r as f32 * factor) as u8,
                        (color.g as f32 * factor) as u8,
                        (color.b as f32 * factor) as u8,
                    ));
                }
            }
        }
    }

    /// Get the effect name
    fn name(&self) -> &str {
        "Scanline"
    }

    /// Check if the effect is enabled
    fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enable or disable the effect
    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Get the effect intensity
    fn intensity(&self) -> f32 {
        self.intensity
    }

    /// Set the effect intensity
    ///
    /// # Arguments
    /// * `intensity` - Intensity value (clamped to 0.0-1.0)
    fn set_intensity(&mut self, intensity: f32) {
        self.intensity = intensity.clamp(0.0, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanline_new() {
        let effect = ScanlineEffect::new(2);
        assert_eq!(effect.name(), "Scanline");
        assert!(effect.is_enabled());
        assert_eq!(effect.intensity(), 0.5);
        assert_eq!(effect.spacing(), 2);
    }

    #[test]
    fn test_scanline_spacing_minimum() {
        let effect = ScanlineEffect::new(0);
        assert_eq!(effect.spacing(), 1); // Should clamp to minimum 1
    }

    #[test]
    fn test_scanline_enable_disable() {
        let mut effect = ScanlineEffect::new(2);
        assert!(effect.is_enabled());

        effect.set_enabled(false);
        assert!(!effect.is_enabled());

        effect.set_enabled(true);
        assert!(effect.is_enabled());
    }

    #[test]
    fn test_scanline_intensity() {
        let mut effect = ScanlineEffect::new(2);
        
        effect.set_intensity(0.0);
        assert_eq!(effect.intensity(), 0.0);

        effect.set_intensity(1.0);
        assert_eq!(effect.intensity(), 1.0);

        effect.set_intensity(0.5);
        assert_eq!(effect.intensity(), 0.5);

        // Test clamping
        effect.set_intensity(-0.5);
        assert_eq!(effect.intensity(), 0.0);

        effect.set_intensity(1.5);
        assert_eq!(effect.intensity(), 1.0);
    }

    #[test]
    fn test_scanline_set_spacing() {
        let mut effect = ScanlineEffect::new(2);
        
        effect.set_spacing(4);
        assert_eq!(effect.spacing(), 4);

        effect.set_spacing(0);
        assert_eq!(effect.spacing(), 1); // Should clamp to minimum 1
    }

    #[test]
    fn test_scanline_apply() {
        use crate::visualization::GridBuffer;

        let mut effect = ScanlineEffect::new(2);
        let mut grid = GridBuffer::new(10, 10);
        let params = AudioParameters::default();

        // Fill grid with colored cells
        for y in 0..10 {
            for x in 0..10 {
                grid.set_cell_with_color(x, y, 'X', Color::new(255, 255, 255));
            }
        }

        // Apply scanline effect
        effect.apply(&mut grid, &params);

        // Check that scanline rows (0, 2, 4, 6, 8) are dimmed
        for y in (0..10).step_by(2) {
            let cell = grid.get_cell(0, y);
            let color = cell.foreground_color.unwrap();
            // Should be dimmed (less than 255)
            assert!(color.r < 255, "Row {} should be dimmed", y);
            assert!(color.g < 255, "Row {} should be dimmed", y);
            assert!(color.b < 255, "Row {} should be dimmed", y);
        }

        // Check that non-scanline rows (1, 3, 5, 7, 9) are NOT dimmed
        for y in (1..10).step_by(2) {
            let cell = grid.get_cell(0, y);
            let color = cell.foreground_color.unwrap();
            // Should be original brightness
            assert_eq!(color.r, 255, "Row {} should NOT be dimmed", y);
            assert_eq!(color.g, 255, "Row {} should NOT be dimmed", y);
            assert_eq!(color.b, 255, "Row {} should NOT be dimmed", y);
        }
    }

    #[test]
    fn test_scanline_disabled() {
        use crate::visualization::GridBuffer;

        let mut effect = ScanlineEffect::new(2);
        effect.set_enabled(false);
        
        let mut grid = GridBuffer::new(10, 10);
        let params = AudioParameters::default();

        // Fill grid with colored cells
        for y in 0..10 {
            for x in 0..10 {
                grid.set_cell_with_color(x, y, 'X', Color::new(255, 255, 255));
            }
        }

        // Apply disabled effect
        effect.apply(&mut grid, &params);

        // All cells should remain unchanged
        for y in 0..10 {
            for x in 0..10 {
                let cell = grid.get_cell(x, y);
                let color = cell.foreground_color.unwrap();
                assert_eq!(color.r, 255);
                assert_eq!(color.g, 255);
                assert_eq!(color.b, 255);
            }
        }
    }
}

