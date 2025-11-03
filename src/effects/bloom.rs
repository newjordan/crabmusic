//! Bloom Effect
//!
//! Implements a glow/bloom effect that makes bright elements "glow" by blurring
//! and adding luminosity to pixels above a brightness threshold.

use crate::dsp::AudioParameters;
use crate::effects::Effect;
use crate::visualization::{Color, GridBuffer};

/// Generate a normalized Gaussian blur kernel
///
/// # Arguments
/// * `radius` - Blur radius (kernel size will be radius*2+1)
///
/// # Returns
/// Normalized Gaussian kernel weights
///
/// # Examples
///
/// ```
/// use crabmusic::effects::bloom::gaussian_kernel;
///
/// let kernel = gaussian_kernel(2);
/// assert_eq!(kernel.len(), 5); // radius*2+1
/// ```
pub fn gaussian_kernel(radius: usize) -> Vec<f32> {
    let sigma = radius as f32 / 2.0;
    let size = radius * 2 + 1;
    let mut kernel = vec![0.0; size];
    let mut sum = 0.0;
    
    for i in 0..size {
        let x = i as f32 - radius as f32;
        kernel[i] = (-x * x / (2.0 * sigma * sigma)).exp();
        sum += kernel[i];
    }
    
    // Normalize so weights sum to 1.0
    for i in 0..size {
        kernel[i] /= sum;
    }
    
    kernel
}

/// Calculate perceived brightness of a color using ITU-R BT.709 luminance formula
///
/// # Arguments
/// * `color` - Color to calculate brightness for
///
/// # Returns
/// Brightness value in range 0.0-1.0
///
/// # Examples
///
/// ```
/// use crabmusic::effects::bloom::color_brightness;
/// use crabmusic::visualization::Color;
///
/// let white = Color::new(255, 255, 255);
/// assert_eq!(color_brightness(white), 1.0);
///
/// let black = Color::new(0, 0, 0);
/// assert_eq!(color_brightness(black), 0.0);
/// ```
pub fn color_brightness(color: Color) -> f32 {
    // ITU-R BT.709 perceived luminance formula
    (0.2126 * color.r as f32 + 0.7152 * color.g as f32 + 0.0722 * color.b as f32) / 255.0
}

/// Glow effect that makes bright elements glow by brightening neighbors
///
/// This is a simplified "bloom" that works better in terminals.
/// Instead of complex Gaussian blur, it just brightens cells around bright spots.
///
/// This effect works by:
/// 1. Finding bright pixels above a threshold
/// 2. Brightening neighboring cells in a radius around each bright pixel
/// 3. Using distance-based falloff for smooth glow
///
/// # Examples
///
/// ```
/// use crabmusic::effects::bloom::BloomEffect;
///
/// // Create glow with 0.3 threshold and radius 3
/// let mut effect = BloomEffect::new(0.3, 3);
/// effect.set_intensity(0.8);
/// ```
#[derive(Debug, Clone)]
pub struct BloomEffect {
    /// Whether the effect is enabled
    enabled: bool,
    /// Effect intensity (0.0-1.0, strength of glow)
    intensity: f32,
    /// Brightness threshold (0.0-1.0, pixels above this glow)
    threshold: f32,
    /// Glow radius (1-5, how far the glow spreads)
    blur_radius: usize,
}

impl BloomEffect {
    /// Create a new glow effect
    ///
    /// # Arguments
    /// * `threshold` - Brightness threshold (0.0-1.0, pixels above this glow)
    /// * `blur_radius` - Glow radius (1-5, how far the glow spreads)
    ///
    /// # Returns
    /// A new BloomEffect with default intensity (0.8) and enabled state (true)
    ///
    /// # Examples
    ///
    /// ```
    /// use crabmusic::effects::bloom::BloomEffect;
    ///
    /// let effect = BloomEffect::new(0.3, 3);
    /// ```
    pub fn new(threshold: f32, blur_radius: usize) -> Self {
        let threshold = threshold.clamp(0.0, 1.0);
        let blur_radius = blur_radius.clamp(1, 5);

        Self {
            enabled: true,
            intensity: 0.8, // Higher default for visibility
            threshold,
            blur_radius,
        }
    }

    /// Get the current brightness threshold
    pub fn threshold(&self) -> f32 {
        self.threshold
    }

    /// Set the brightness threshold
    ///
    /// # Arguments
    /// * `threshold` - Brightness threshold (clamped to 0.0-1.0)
    pub fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold.clamp(0.0, 1.0);
    }

    /// Get the current blur radius
    pub fn blur_radius(&self) -> usize {
        self.blur_radius
    }

    /// Set the glow radius
    ///
    /// # Arguments
    /// * `radius` - Glow radius (clamped to 1-5)
    pub fn set_blur_radius(&mut self, radius: usize) {
        self.blur_radius = radius.clamp(1, 5);
    }
}

impl Effect for BloomEffect {
    /// Apply the glow effect to the grid buffer
    ///
    /// This is a simplified "bloom" that works better in terminals:
    /// 1. Find all bright pixels above threshold
    /// 2. For each bright pixel, brighten neighboring cells in a radius
    /// 3. Use distance-based falloff for smooth glow
    fn apply(&mut self, grid: &mut GridBuffer, _params: &AudioParameters) {
        if !self.enabled {
            return;
        }

        let width = grid.width();
        let height = grid.height();

        // First pass: collect all bright pixels
        let mut bright_pixels = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let cell = grid.get_cell(x, y);
                if let Some(color) = cell.foreground_color {
                    let brightness = color_brightness(color);
                    if brightness >= self.threshold {
                        bright_pixels.push((x, y, color));
                    }
                }
            }
        }

        // Second pass: apply glow around each bright pixel
        let radius = self.blur_radius as isize;
        for (bx, by, bright_color) in bright_pixels {
            // Apply glow in a square radius around the bright pixel
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let x = bx as isize + dx;
                    let y = by as isize + dy;

                    // Skip out of bounds
                    if x < 0 || y < 0 || x >= width as isize || y >= height as isize {
                        continue;
                    }

                    let x = x as usize;
                    let y = y as usize;

                    // Calculate distance-based falloff
                    let distance = ((dx * dx + dy * dy) as f32).sqrt();
                    let max_distance = (radius * radius * 2) as f32;
                    let falloff = (1.0 - (distance / max_distance)).max(0.0);
                    let glow_strength = self.intensity * falloff;

                    // Apply glow (additive blending)
                    let cell = grid.get_cell_mut(x, y);
                    if let Some(original_color) = cell.foreground_color {
                        cell.foreground_color = Some(Color::new(
                            (original_color.r as f32 + bright_color.r as f32 * glow_strength).min(255.0) as u8,
                            (original_color.g as f32 + bright_color.g as f32 * glow_strength).min(255.0) as u8,
                            (original_color.b as f32 + bright_color.b as f32 * glow_strength).min(255.0) as u8,
                        ));
                    } else {
                        // If cell is empty, add glow directly
                        cell.foreground_color = Some(Color::new(
                            (bright_color.r as f32 * glow_strength).min(255.0) as u8,
                            (bright_color.g as f32 * glow_strength).min(255.0) as u8,
                            (bright_color.b as f32 * glow_strength).min(255.0) as u8,
                        ));
                    }
                }
            }
        }
    }

    fn name(&self) -> &str {
        "Bloom"
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
    fn test_gaussian_kernel() {
        let kernel = gaussian_kernel(2);
        assert_eq!(kernel.len(), 5); // radius*2+1

        // Kernel should sum to approximately 1.0
        let sum: f32 = kernel.iter().sum();
        assert!((sum - 1.0).abs() < 0.001, "Kernel sum should be 1.0, got {}", sum);

        // Center should have highest weight
        assert!(kernel[2] > kernel[1]);
        assert!(kernel[2] > kernel[3]);
    }

    #[test]
    fn test_color_brightness() {
        // White should be 1.0
        let white = Color::new(255, 255, 255);
        assert_eq!(color_brightness(white), 1.0);

        // Black should be 0.0
        let black = Color::new(0, 0, 0);
        assert_eq!(color_brightness(black), 0.0);

        // Gray should be around 0.5
        let gray = Color::new(128, 128, 128);
        let brightness = color_brightness(gray);
        assert!(brightness > 0.4 && brightness < 0.6, "Gray brightness should be ~0.5, got {}", brightness);

        // Green should be brighter than red (ITU-R BT.709)
        let red = Color::new(255, 0, 0);
        let green = Color::new(0, 255, 0);
        assert!(color_brightness(green) > color_brightness(red));
    }

    #[test]
    fn test_bloom_new() {
        let effect = BloomEffect::new(0.7, 2);
        assert_eq!(effect.name(), "Bloom");
        assert!(effect.is_enabled());
        assert_eq!(effect.intensity(), 0.5);
        assert_eq!(effect.threshold(), 0.7);
        assert_eq!(effect.blur_radius(), 2);
    }

    #[test]
    fn test_bloom_threshold_clamping() {
        let effect = BloomEffect::new(-0.5, 2);
        assert_eq!(effect.threshold(), 0.0);

        let effect = BloomEffect::new(1.5, 2);
        assert_eq!(effect.threshold(), 1.0);
    }

    #[test]
    fn test_bloom_radius_clamping() {
        let effect = BloomEffect::new(0.7, 0);
        assert_eq!(effect.blur_radius(), 1);

        let effect = BloomEffect::new(0.7, 10);
        assert_eq!(effect.blur_radius(), 5);
    }

    #[test]
    fn test_bloom_enable_disable() {
        let mut effect = BloomEffect::new(0.7, 2);
        assert!(effect.is_enabled());

        effect.set_enabled(false);
        assert!(!effect.is_enabled());

        effect.set_enabled(true);
        assert!(effect.is_enabled());
    }

    #[test]
    fn test_bloom_intensity() {
        let mut effect = BloomEffect::new(0.7, 2);

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
    fn test_bloom_set_threshold() {
        let mut effect = BloomEffect::new(0.7, 2);

        effect.set_threshold(0.5);
        assert_eq!(effect.threshold(), 0.5);

        effect.set_threshold(-0.5);
        assert_eq!(effect.threshold(), 0.0);

        effect.set_threshold(1.5);
        assert_eq!(effect.threshold(), 1.0);
    }

    #[test]
    fn test_bloom_set_blur_radius() {
        let mut effect = BloomEffect::new(0.7, 2);

        effect.set_blur_radius(3);
        assert_eq!(effect.blur_radius(), 3);

        effect.set_blur_radius(0);
        assert_eq!(effect.blur_radius(), 1);

        effect.set_blur_radius(10);
        assert_eq!(effect.blur_radius(), 5);
    }

    #[test]
    fn test_bloom_disabled() {
        use crate::visualization::GridBuffer;

        let mut effect = BloomEffect::new(0.7, 2);
        effect.set_enabled(false);

        let mut grid = GridBuffer::new(10, 10);
        let params = AudioParameters::default();

        // Fill grid with bright colored cells
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

