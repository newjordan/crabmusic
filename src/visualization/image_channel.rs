// Image channel as a Visualizer (interactive)
// Allows typing/pasting a file path and renders the image inline using Braille.

use crate::dsp::AudioParameters;
use crate::visualization::color_schemes::ColorScheme;
use crate::visualization::{BrailleGrid, GridBuffer, Visualizer};
// Local luma -> Braille blit to avoid cross-module import from lib builds
fn blit_luma_to_braille(
    luma: &[u8],
    img_w: usize,
    img_h: usize,
    threshold: u8,
    braille: &mut BrailleGrid,
) {
    if img_w == 0 || img_h == 0 {
        return;
    }
    let dot_w = braille.dot_width();
    let dot_h = braille.dot_height();
    for dy in 0..dot_h {
        let sy = (dy * img_h) / dot_h;
        let sy_off = sy * img_w;
        for dx in 0..dot_w {
            let sx = (dx * img_w) / dot_w;
            let v = luma[sy_off + sx];
            if v >= threshold {
                braille.set_dot(dx, dy);
            }
        }
    }
}

use image::DynamicImage;

pub struct ImageChannelVisualizer {
    color_scheme: ColorScheme,
    pulse: f32,
    // Temporary: disable actual image loading; render white noise instead
    noise_seed: u64,
    current_image: Option<DynamicImage>,
    current_path: Option<String>,
}

impl ImageChannelVisualizer {
    pub fn new(color_scheme: ColorScheme) -> Self {
        Self {
            color_scheme,
            pulse: 0.0,
            noise_seed: 0x9E37_79B9_7F4A_7C15,
            current_image: None,
            current_path: None,
        }
    }

    pub fn set_color_scheme(&mut self, scheme: ColorScheme) {
        self.color_scheme = scheme;
    }

    /// Try to load an image from path
    pub fn try_load(&mut self, path: &str) -> Result<(), String> {
        match image::open(path) {
            Ok(img) => {
                self.current_path = Some(path.to_string());
                self.current_image = Some(img);
                tracing::info!("Loaded image: {}", path);
                Ok(())
            }
            Err(e) => Err(format!("Failed to open image '{}': {}", path, e)),
        }
    }

    fn draw_centered(grid: &mut GridBuffer, row: usize, text: &str) {
        if row >= grid.height() {
            return;
        }
        let start_x = (grid.width().saturating_sub(text.len())) / 2;
        for (i, ch) in text.chars().enumerate() {
            let x = start_x + i;
            if x < grid.width() {
                grid.set_cell(x, row, ch);
            }
        }
    }
}

impl Visualizer for ImageChannelVisualizer {
    fn update(&mut self, params: &AudioParameters) {
        // Gentle pulse based on amplitude
        let target = params.amplitude.clamp(0.0, 1.0);
        self.pulse = self.pulse + (target - self.pulse) * 0.15;
        // Advance noise seed
        self.noise_seed = self.noise_seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    }

    fn render(&self, grid: &mut GridBuffer) {
        // White noise: fill screen with pseudo-random characters
        // Deterministic per-frame using noise_seed, stable across cells using a simple mix
        const CHARS: &[u8] = b" .:-=+*#%@"; // light -> dense
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                let mut v = self.noise_seed ^ ((x as u64) << 32) ^ (y as u64);
                // SplitMix64 mix
                v ^= v >> 30;
                v = v.wrapping_mul(0xBF58_476D_1CE4_E5B9);
                v ^= v >> 27;
                v = v.wrapping_mul(0x94D0_49BB_1331_11EB);
                v ^= v >> 31;
                let idx = (v as usize) % CHARS.len();
                grid.set_cell(x, y, CHARS[idx] as char);
            }
        }
        // Top-left label to indicate temporary mode
        let label = "White noise (image channel disabled)";
        for (i, ch) in label.chars().enumerate() {
            if i < grid.width() {
                grid.set_cell(i, 0, ch);
            }
        }
    }

    fn name(&self) -> &str {
        "White Noise (Image)"
    }
}
