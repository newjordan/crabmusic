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

/// Bloom effect that makes bright elements glow
///
/// This effect works by:
/// 1. Extracting bright pixels above a threshold
/// 2. Applying Gaussian blur to the bright pixels
/// 3. Compositing the blurred result back onto the original (additive blending)
///
/// # Examples
///
/// ```
/// use crabmusic::effects::bloom::BloomEffect;
///
/// // Create bloom with 0.7 threshold and radius 2
/// let mut effect = BloomEffect::new(0.7, 2);
/// effect.set_intensity(0.5);
/// ```
#[derive(Debug, Clone)]
pub struct BloomEffect {
    /// Whether the effect is enabled
    enabled: bool,
    /// Effect intensity (0.0-1.0, strength of bloom)
    intensity: f32,
    /// Brightness threshold (0.0-1.0, pixels above this bloom)
    threshold: f32,
    /// Blur radius (1-5, size of blur kernel)
    blur_radius: usize,
    /// Temporary buffer for bright pixels
    bright_buffer: Vec<Option<Color>>,
    /// Temporary buffer for blurred result
    blur_buffer: Vec<Option<Color>>,
    /// Cached Gaussian kernel
    kernel: Vec<f32>,
}

impl BloomEffect {
    /// Create a new bloom effect
    ///
    /// # Arguments
    /// * `threshold` - Brightness threshold (0.0-1.0, pixels above this bloom)
    /// * `blur_radius` - Blur radius (1-5, size of blur kernel)
    ///
    /// # Returns
    /// A new BloomEffect with default intensity (0.5) and enabled state (true)
    ///
    /// # Examples
    ///
    /// ```
    /// use crabmusic::effects::bloom::BloomEffect;
    ///
    /// let effect = BloomEffect::new(0.7, 2);
    /// ```
    pub fn new(threshold: f32, blur_radius: usize) -> Self {
        let threshold = threshold.clamp(0.0, 1.0);
        let blur_radius = blur_radius.clamp(1, 5);
        let kernel = gaussian_kernel(blur_radius);

        Self {
            enabled: true,
            intensity: 0.5,
            threshold,
            blur_radius,
            bright_buffer: Vec::new(),
            blur_buffer: Vec::new(),
            kernel,
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

    /// Set the blur radius
    ///
    /// # Arguments
    /// * `radius` - Blur radius (clamped to 1-5)
    pub fn set_blur_radius(&mut self, radius: usize) {
        self.blur_radius = radius.clamp(1, 5);
        self.kernel = gaussian_kernel(self.blur_radius);
    }

    /// Resize internal buffers to match grid dimensions
    fn resize_buffers(&mut self, width: usize, height: usize) {
        let size = width * height;
        if self.bright_buffer.len() != size {
            self.bright_buffer.resize(size, None);
            self.blur_buffer.resize(size, None);
        }
    }

    /// Extract bright pixels above threshold
    fn extract_bright_pixels(&mut self, grid: &GridBuffer) {
        let width = grid.width();

        for y in 0..grid.height() {
            for x in 0..width {
                let cell = grid.get_cell(x, y);
                let idx = y * width + x;

                if let Some(color) = cell.foreground_color {
                    let brightness = color_brightness(color);

                    if brightness >= self.threshold {
                        self.bright_buffer[idx] = Some(color);
                    } else {
                        self.bright_buffer[idx] = None;
                    }
                } else {
                    self.bright_buffer[idx] = None;
                }
            }
        }
    }

    /// Apply horizontal Gaussian blur pass
    fn blur_horizontal(&mut self, width: usize, height: usize) {
        for y in 0..height {
            for x in 0..width {
                let mut r_sum = 0.0;
                let mut g_sum = 0.0;
                let mut b_sum = 0.0;
                let mut weight_sum = 0.0;

                for (i, &weight) in self.kernel.iter().enumerate() {
                    let offset = i as isize - self.blur_radius as isize;
                    let sample_x = (x as isize + offset).clamp(0, width as isize - 1) as usize;

                    if let Some(color) = self.bright_buffer[y * width + sample_x] {
                        r_sum += color.r as f32 * weight;
                        g_sum += color.g as f32 * weight;
                        b_sum += color.b as f32 * weight;
                        weight_sum += weight;
                    }
                }

                if weight_sum > 0.0 {
                    self.blur_buffer[y * width + x] = Some(Color::new(
                        (r_sum / weight_sum) as u8,
                        (g_sum / weight_sum) as u8,
                        (b_sum / weight_sum) as u8,
                    ));
                } else {
                    self.blur_buffer[y * width + x] = None;
                }
            }
        }
    }

    /// Apply vertical Gaussian blur pass
    fn blur_vertical(&mut self, width: usize, height: usize) {
        // Copy blur_buffer to bright_buffer for vertical pass
        self.bright_buffer.copy_from_slice(&self.blur_buffer);

        for y in 0..height {
            for x in 0..width {
                let mut r_sum = 0.0;
                let mut g_sum = 0.0;
                let mut b_sum = 0.0;
                let mut weight_sum = 0.0;

                for (i, &weight) in self.kernel.iter().enumerate() {
                    let offset = i as isize - self.blur_radius as isize;
                    let sample_y = (y as isize + offset).clamp(0, height as isize - 1) as usize;

                    if let Some(color) = self.bright_buffer[sample_y * width + x] {
                        r_sum += color.r as f32 * weight;
                        g_sum += color.g as f32 * weight;
                        b_sum += color.b as f32 * weight;
                        weight_sum += weight;
                    }
                }

                if weight_sum > 0.0 {
                    self.blur_buffer[y * width + x] = Some(Color::new(
                        (r_sum / weight_sum) as u8,
                        (g_sum / weight_sum) as u8,
                        (b_sum / weight_sum) as u8,
                    ));
                } else {
                    self.blur_buffer[y * width + x] = None;
                }
            }
        }
    }

    /// Composite blurred bloom back onto original grid (additive blending)
    fn composite_bloom(&self, grid: &mut GridBuffer) {
        let width = grid.width();

        for y in 0..grid.height() {
            for x in 0..width {
                let idx = y * width + x;

                if let Some(bloom_color) = self.blur_buffer[idx] {
                    let cell = grid.get_cell_mut(x, y);

                    if let Some(original_color) = cell.foreground_color {
                        // Additive blend with intensity
                        cell.foreground_color = Some(Color::new(
                            (original_color.r as f32 + bloom_color.r as f32 * self.intensity)
                                .min(255.0) as u8,
                            (original_color.g as f32 + bloom_color.g as f32 * self.intensity)
                                .min(255.0) as u8,
                            (original_color.b as f32 + bloom_color.b as f32 * self.intensity)
                                .min(255.0) as u8,
                        ));
                    }
                }
            }
        }
    }
}

impl Effect for BloomEffect {
    /// Apply the bloom effect to the grid buffer
    ///
    /// This method performs a multi-pass bloom:
    /// 1. Extract bright pixels above threshold
    /// 2. Apply horizontal Gaussian blur
    /// 3. Apply vertical Gaussian blur
    /// 4. Composite blurred result back onto original (additive)
    fn apply(&mut self, grid: &mut GridBuffer, _params: &AudioParameters) {
        if !self.enabled {
            return;
        }

        let width = grid.width();
        let height = grid.height();

        // Resize buffers if needed
        self.resize_buffers(width, height);

        // Pass 1: Extract bright pixels above threshold
        self.extract_bright_pixels(grid);

        // Pass 2: Apply separable Gaussian blur (horizontal)
        self.blur_horizontal(width, height);

        // Pass 3: Apply separable Gaussian blur (vertical)
        self.blur_vertical(width, height);

        // Pass 4: Composite blurred bloom back onto original (additive)
        self.composite_bloom(grid);
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
        assert!(
            (sum - 1.0).abs() < 0.001,
            "Kernel sum should be 1.0, got {}",
            sum
        );

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
        assert!(
            brightness > 0.4 && brightness < 0.6,
            "Gray brightness should be ~0.5, got {}",
            brightness
        );

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
