// Night Night visualizer
// Minimal starfield + soft pulsing title, audio-reactive brightness and twinkle.

use super::{Color, GridBuffer, Visualizer};
use super::color_schemes::ColorScheme;
use crate::dsp::AudioParameters;

pub struct NightNightVisualizer {
    color_scheme: ColorScheme,
    time: f32,
    amp: f32,
    bass: f32,
    mid: f32,
    treble: f32,
}

impl NightNightVisualizer {
    pub fn new(color_scheme: ColorScheme) -> Self {
        Self { color_scheme, time: 0.0, amp: 0.0, bass: 0.0, mid: 0.0, treble: 0.0 }
    }

    #[inline]
    fn lerp(a: f32, b: f32, t: f32) -> f32 { a + (b - a) * t }

    // Simple integer hash for pseudo-random star placement
    #[inline]
    fn hash(x: i32, y: i32) -> u32 {
        let mut h = (x as u32).wrapping_mul(73856093) ^ (y as u32).wrapping_mul(19349663);
        h ^= h >> 13;
        h = h.wrapping_mul(1274126177);
        h ^ (h >> 16)
    }
}

impl Visualizer for NightNightVisualizer {
    fn update(&mut self, params: &AudioParameters) {
        let s = 0.18;
        self.amp = Self::lerp(self.amp, params.amplitude, s);
        self.bass = Self::lerp(self.bass, params.bass, s);
        self.mid = Self::lerp(self.mid, params.mid, s);
        self.treble = Self::lerp(self.treble, params.treble, s);

        // Drift time; speed up slightly with mid
        self.time += 0.04 + self.mid * 0.08;
        if self.time > 1000.0 { self.time = 0.0; }
    }

    fn render(&self, grid: &mut GridBuffer) {
        grid.clear();
        let w = grid.width();
        let h = grid.height();

        // Sky gradient rows: darker at top, lighter near horizon
        for y in 1..h { // leave row 0 for UI bar
            let ny = y as f32 / h as f32;
            let base_intensity = (0.15 + 0.5 * (1.0 - ny)).clamp(0.0, 1.0);
            let twinkle = (self.time * 0.6 + ny * 2.0).sin().abs() * 0.15 * (0.5 + self.treble);
            let intensity = (base_intensity + twinkle * 0.5).min(1.0);
            if let Some(color) = self.color_scheme.get_color(intensity) {
                for x in 0..w {
                    // Only color a subtle background (use thin dot chars)
                    grid.set_cell_with_color(x, y, ' ', color); // color used by renderer even for space
                }
            }
        }

        // Stars: sparse pseudo-random, twinkle with treble
        for y in 1..h.saturating_sub(2) {
            for x in 0..w {
                let hval = Self::hash(x as i32, y as i32) % 97; // density control
                if hval < 2 { // ~2% density
                    // Twinkle phase per-star
                    let phase = (Self::hash((x as i32)+13, (y as i32)-7) as f32).to_bits() as f32;
                    let sparkle = (self.time * 1.2 + (phase % 6.28)).sin().abs();
                    let intensity = (0.4 + 0.6 * sparkle * (0.5 + self.treble)).clamp(0.0, 1.0);
                    if let Some(color) = self.color_scheme.get_color(intensity) {
                        let ch = if sparkle > 0.66 { '✦' } else if sparkle > 0.33 { '•' } else { '·' };
                        grid.set_cell_with_color(x, y, ch, color);
                    } else {
                        grid.set_cell(x, y, '*');
                    }
                }
            }
        }

        // "Night Night" title, softly pulsing with bass
        let title = "Night Night";
        let start_x = if w > title.len() { (w - title.len()) / 2 } else { 0 };
        let y = h.saturating_sub(3);
        let pulse = (0.5 + 0.5 * (self.time * 0.8).sin()) * (0.4 + 0.6 * (0.4 + self.bass)).min(1.0);
        let color = self.color_scheme.get_color(pulse).unwrap_or(Color::new(180, 200, 255));
        for (i, ch) in title.chars().enumerate() {
            if start_x + i < w {
                grid.set_cell_with_color(start_x + i, y, ch, color);
            }
        }
    }

    fn name(&self) -> &str { "Night Night" }
}

