// Video channel as a Visualizer (interactive path input)
// Lets you type/paste a file path. Playback requires building with --features video.

use crate::dsp::AudioParameters;
use crate::visualization::color_schemes::ColorScheme;
use crate::visualization::{GridBuffer, Visualizer};

pub struct VideoChannelVisualizer {
    color_scheme: ColorScheme,
    pulse: f32,
    // Temporary: disable actual video; render white noise instead
    noise_seed: u64,
    current_path: Option<String>,
}

impl VideoChannelVisualizer {
    pub fn new(color_scheme: ColorScheme) -> Self {
        Self {
            color_scheme,
            pulse: 0.0,
            noise_seed: 0xC2B2_AE35_87B9_3A15,
            current_path: None,
        }
    }

    pub fn set_color_scheme(&mut self, scheme: ColorScheme) {
        self.color_scheme = scheme;
    }

    /// Accept a path (playback occurs when built with feature "video")
    pub fn try_load(&mut self, path: &str) -> Result<(), String> {
        self.current_path = Some(path.to_string());
        tracing::info!("Selected video: {}", path);
        Ok(())
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

impl Visualizer for VideoChannelVisualizer {
    fn update(&mut self, params: &AudioParameters) {
        // Gentle pulse based on amplitude to give it some life
        let target = params.amplitude.clamp(0.0, 1.0);
        self.pulse = self.pulse + (target - self.pulse) * 0.15;
        // Advance noise seed
        self.noise_seed = self.noise_seed.wrapping_add(0xC2B2_AE35_87B9_3A15);
    }

    fn render(&self, grid: &mut GridBuffer) {
        // White noise: fill screen with pseudo-random characters
        const CHARS: &[u8] = b" .:-=+*#%@";
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                let mut v = self.noise_seed ^ ((x as u64) << 33) ^ (y as u64);
                v ^= v >> 30;
                v = v.wrapping_mul(0xBF58_476D_1CE4_E5B9);
                v ^= v >> 27;
                v = v.wrapping_mul(0x94D0_49BB_1331_11EB);
                v ^= v >> 31;
                let idx = (v as usize) % CHARS.len();
                grid.set_cell(x, y, CHARS[idx] as char);
            }
        }
        let label = "White noise (video channel disabled)";
        for (i, ch) in label.chars().enumerate() {
            if i < grid.width() {
                grid.set_cell(i, 0, ch);
            }
        }
        let _ = &self.color_scheme; // Reserved for future colorization
    }

    fn name(&self) -> &str {
        "White Noise (Video)"
    }
}
