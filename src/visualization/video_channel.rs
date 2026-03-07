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
    display_name: String,
    idle_label: String,
    current_path: Option<String>,
}

impl VideoChannelVisualizer {
    pub fn new(color_scheme: ColorScheme) -> Self {
        Self::new_named(
            color_scheme,
            "White Noise (Video)",
            "White noise (video channel disabled)",
        )
    }

    pub fn new_named(
        color_scheme: ColorScheme,
        display_name: impl Into<String>,
        idle_label: impl Into<String>,
    ) -> Self {
        Self {
            color_scheme,
            pulse: 0.0,
            noise_seed: 0xC2B2_AE35_87B9_3A15,
            display_name: display_name.into(),
            idle_label: idle_label.into(),
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

    fn truncate_for_width(text: &str, width: usize) -> String {
        let char_count = text.chars().count();
        if char_count <= width {
            return text.to_string();
        }

        if width <= 1 {
            return "…".to_string();
        }

        let mut out = text.chars().take(width - 1).collect::<String>();
        out.push('…');
        out
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

        let title = Self::truncate_for_width(&self.display_name, grid.width().saturating_sub(2));
        Self::draw_centered(grid, 0, &title);

        let subtitle = self.current_path.as_deref().unwrap_or(&self.idle_label);
        let subtitle = Self::truncate_for_width(subtitle, grid.width().saturating_sub(4));
        Self::draw_centered(grid, 2, &subtitle);

        let _ = &self.color_scheme; // Reserved for future colorization
    }

    fn name(&self) -> &str {
        &self.display_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn archive_named_video_channel_uses_custom_name() {
        let viz = VideoChannelVisualizer::new_named(
            ColorScheme::default(),
            "Archive TV",
            "Tuning random old television...",
        );

        assert_eq!(viz.name(), "Archive TV");
    }
}
