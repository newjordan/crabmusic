// Illuminati Eye visualizer
// Simplified, fast, non-AA braille drawing. Audio-reactive triangle + eye + pupil.

use super::{BrailleGrid, Color, GridBuffer, Visualizer};
use super::color_schemes::ColorScheme;
use crate::dsp::AudioParameters;

#[derive(Debug, Clone)]
pub struct IlluminatiEyeConfig {
    pub base_radius: f32,      // base eye radius in DOT space fraction of min(dot_w,dot_h)
    pub triangle_margin: f32,  // margin from edges as fraction of dot dims
}

impl Default for IlluminatiEyeConfig {
    fn default() -> Self {
        Self {
            base_radius: 0.14,
            triangle_margin: 0.08,
        }
    }
}

pub struct IlluminatiEyeVisualizer {
    config: IlluminatiEyeConfig,
    color_scheme: ColorScheme,
    // smoothed bands
    bass: f32,
    mid: f32,
    treble: f32,
    amplitude: f32,
    // animation state
    time: f32,
    beat_flash: f32,
}

impl IlluminatiEyeVisualizer {
    pub fn new(color_scheme: ColorScheme) -> Self {
        Self {
            config: IlluminatiEyeConfig::default(),
            color_scheme,
            bass: 0.0,
            mid: 0.0,
            treble: 0.0,
            amplitude: 0.0,
            time: 0.0,
            beat_flash: 0.0,
        }
    }

    #[inline]
    fn lerp(a: f32, b: f32, t: f32) -> f32 { a + (b - a) * t }
}

impl Visualizer for IlluminatiEyeVisualizer {
    fn update(&mut self, params: &AudioParameters) {
        let s = 0.18; // smoothing
        self.bass = Self::lerp(self.bass, params.bass, s);
        self.mid = Self::lerp(self.mid, params.mid, s);
        self.treble = Self::lerp(self.treble, params.treble, s);
        self.amplitude = Self::lerp(self.amplitude, params.amplitude, s);

        // Drive time by mid (melodic content)
        self.time += 0.06 + self.mid * 0.12;
        if self.time > 1000.0 { self.time = 0.0; }

        // Beat flash for highlights
        if params.beat { self.beat_flash = 1.0; } else { self.beat_flash *= 0.88; }
    }

    fn render(&self, grid: &mut GridBuffer) {
        grid.clear();
        let width = grid.width();
        let height = grid.height();
        let mut braille = BrailleGrid::new(width, height);

        let dot_w = braille.dot_width() as f32;
        let dot_h = braille.dot_height() as f32;
        let cx = dot_w * 0.5;
        let cy = dot_h * 0.52; // slightly low for aesthetics

        // Triangle vertices (equilateral-ish) with margin
        let m = self.config.triangle_margin;
        let left = (dot_w * m, dot_h * (1.0 - m));
        let right = (dot_w * (1.0 - m), dot_h * (1.0 - m));
        let top = (dot_w * 0.5, dot_h * m);

        // Color base from scheme by amplitude + beat flash
        let intensity = (self.amplitude * 0.8 + self.beat_flash * 0.6).clamp(0.0, 1.0);
        let edge_color = self
            .color_scheme
            .get_color(intensity)
            .unwrap_or(Color::new(255, 215, 0)); // gold-ish fallback

        // Draw triangle edges
        let (x1, y1) = (left.0.round() as usize, left.1.round() as usize);
        let (x2, y2) = (right.0.round() as usize, right.1.round() as usize);
        let (x3, y3) = (top.0.round() as usize, top.1.round() as usize);
        braille.draw_line_with_color(x1, y1, x2, y2, edge_color);
        braille.draw_line_with_color(x2, y2, x3, y3, edge_color);
        braille.draw_line_with_color(x3, y3, x1, y1, edge_color);

        // Eye radius driven by amplitude, bass swells the outline, treble wiggles pupil
        let min_dim = dot_w.min(dot_h);
        let base_r = self.config.base_radius * min_dim;
        let radius = (base_r * (1.0 + self.amplitude * 0.6 + self.bass * 0.4)).clamp(4.0, min_dim * 0.45);
        let pupil_r = (radius * (0.25 + 0.12 * (1.0 + (self.time * (1.0 + self.treble * 1.5)).sin()))).max(2.0);

        // Eye color sweeps with scheme using treble
        let eye_color = self
            .color_scheme
            .get_color(((self.treble * 0.7) + 0.2).clamp(0.0, 1.0))
            .unwrap_or(Color::new(200, 240, 255));

        // Draw iris (outer circle)
        braille.draw_circle(cx.round() as usize, cy.round() as usize, radius.round() as usize, eye_color);

        // Draw pupil (concentric circles to look filled)
        let pupil_color = Color::new(10, 10, 10);
        let pr = pupil_r.round() as usize;
        for r in (0..=pr).rev().step_by(2) {
            braille.draw_circle(cx.round() as usize, cy.round() as usize, r.max(1), pupil_color);
        }

        // Optional "rays" on beat
        if self.beat_flash > 0.05 {
            let rays = 8usize;
            let ray_len = (radius * (0.35 + self.beat_flash * 0.25)) as usize;
            let rc = self
                .color_scheme
                .get_color((0.6 + 0.4 * intensity).min(1.0))
                .unwrap_or(edge_color);
            for i in 0..rays {
                let a = (i as f32 / rays as f32) * std::f32::consts::TAU + self.time * 0.25;
                let ex = (cx + (radius + ray_len as f32) * a.cos()).round().clamp(0.0, dot_w - 1.0) as usize;
                let ey = (cy + (radius + ray_len as f32) * a.sin()).round().clamp(0.0, dot_h - 1.0) as usize;
                braille.draw_line_with_color(cx as usize, cy as usize, ex, ey, rc);
            }
        }

        // Transfer braille grid to main grid
        for y in 0..height {
            for x in 0..width {
                let ch = braille.get_char(x, y);
                if ch != '⠀' {
                    if let Some(color) = braille.get_color(x, y) {
                        grid.set_cell_with_color(x, y, ch, color);
                    } else {
                        grid.set_cell(x, y, ch);
                    }
                }
            }
        }
    }

    fn name(&self) -> &str { "Illuminati Eye" }
}

