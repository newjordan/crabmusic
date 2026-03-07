use crate::dsp::AudioParameters;
use crate::visualization::{lerp, Color, GridBuffer, Visualizer};

/// 3D Grid Tunnel Visualizer
///
/// A retro-style 3D wireframe tunnel that pulses with the music.
/// - Bass: Expands the tunnel radius (pulse effect).
/// - Mid: Rotates the tunnel (roll).
/// - Treble: Changes the grid color brightness.
pub struct GridTunnelVisualizer {
    /// Z-positions of tunnel segments
    segments: Vec<f32>,
    /// Movement speed
    speed: f32,
    /// Current rotation (roll) in radians
    rotation: f32,
    /// Smoothed roll velocity
    rotation_velocity: f32,
    /// Current pulse scale (1.0 = normal)
    pulse: f32,
    /// Treble-driven glow/brightness
    glow: f32,
    /// User-controlled brightness bias
    glow_bias: f32,
    /// User-controlled zoom factor
    zoom: f32,
    /// User-controlled speed bias
    speed_bias: f32,
    /// Whether audio should keep auto-rolling the tunnel
    auto_roll: bool,
    /// Color scheme
    color_scheme: crate::visualization::color_schemes::ColorScheme,
}

impl GridTunnelVisualizer {
    pub fn new(color_scheme: crate::visualization::color_schemes::ColorScheme) -> Self {
        // Initialize segments spaced out in Z
        let mut segments = Vec::new();
        for i in 0..20 {
            segments.push(1.0 + i as f32 * 0.5);
        }

        Self {
            segments,
            speed: 0.05,
            rotation: 0.0,
            rotation_velocity: 0.02,
            pulse: 1.0,
            glow: 0.4,
            glow_bias: 0.0,
            zoom: 1.0,
            speed_bias: 0.0,
            auto_roll: true,
            color_scheme,
        }
    }

    pub fn set_color_scheme(&mut self, scheme: crate::visualization::color_schemes::ColorScheme) {
        self.color_scheme = scheme;
    }

    pub fn toggle_auto_roll(&mut self) -> bool {
        self.auto_roll = !self.auto_roll;
        self.auto_roll
    }

    pub fn roll_left(&mut self, step: f32) {
        self.rotation -= step;
    }

    pub fn roll_right(&mut self, step: f32) {
        self.rotation += step;
    }

    pub fn speed_down(&mut self) -> f32 {
        self.speed_bias = (self.speed_bias - 0.015).max(-0.03);
        self.speed_bias
    }

    pub fn speed_up(&mut self) -> f32 {
        self.speed_bias = (self.speed_bias + 0.015).min(0.16);
        self.speed_bias
    }

    pub fn zoom_in(&mut self) -> f32 {
        self.zoom = (self.zoom * 1.12).min(1.8);
        self.zoom
    }

    pub fn zoom_out(&mut self) -> f32 {
        self.zoom = (self.zoom / 1.12).max(0.6);
        self.zoom
    }

    pub fn glow_down(&mut self) -> f32 {
        self.glow_bias = (self.glow_bias - 0.12).max(-0.2);
        self.glow_bias
    }

    pub fn glow_up(&mut self) -> f32 {
        self.glow_bias = (self.glow_bias + 0.12).min(0.8);
        self.glow_bias
    }

    /// Draw a line between two points using Bresenham's algorithm
    fn draw_line(
        &self,
        grid: &mut GridBuffer,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        char: char,
        color: Color,
    ) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        let mut x = x0;
        let mut y = y0;

        loop {
            if x >= 0 && x < grid.width() as i32 && y >= 0 && y < grid.height() as i32 {
                grid.set_cell_with_color(x as usize, y as usize, char, color);
            }

            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    fn project_square(
        &self,
        z: f32,
        width: f32,
        height: f32,
        cx: f32,
        cy: f32,
    ) -> Option<[(i32, i32); 4]> {
        if z <= 0.1 {
            return None;
        } // Too close/behind

        let scale = 1.0 / z;
        let scaled_w = width * scale * self.pulse;
        let scaled_h = height * scale * self.pulse;

        // Apply rotation
        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();

        // Corners relative to center
        let corners = [
            (-scaled_w, -scaled_h),
            (scaled_w, -scaled_h),
            (scaled_w, scaled_h),
            (-scaled_w, scaled_h),
        ];

        // Rotate and translate corners
        Some(corners.map(|(x, y)| {
            let rx = x * cos_r - y * sin_r;
            let ry = x * sin_r + y * cos_r;
            ((cx + rx) as i32, (cy + ry) as i32)
        }))
    }

    fn depth_char(z: f32) -> char {
        if z < 1.0 {
            '#'
        } else if z < 2.5 {
            '*'
        } else if z < 4.0 {
            '+'
        } else {
            '.'
        }
    }

    fn brighten(color: Color, factor: f32) -> Color {
        let factor = factor.max(0.0);
        Color::new(
            (color.r as f32 * factor).min(255.0) as u8,
            (color.g as f32 * factor).min(255.0) as u8,
            (color.b as f32 * factor).min(255.0) as u8,
        )
    }
}

impl Visualizer for GridTunnelVisualizer {
    fn update(&mut self, params: &AudioParameters) {
        // Bass controls pulse
        let target_pulse = 1.0 + params.bass * 0.5;
        self.pulse = lerp(self.pulse, target_pulse, 0.12);

        // Speed increases with amplitude
        let target_speed = (0.04 + params.amplitude * 0.08 + params.bass * 0.05 + self.speed_bias)
            .clamp(0.015, 0.24);
        self.speed = lerp(self.speed, target_speed, 0.08);

        // Mid controls rotation
        let target_rotation_velocity = if self.auto_roll {
            0.01 + params.mid * 0.09
        } else {
            0.0
        };
        self.rotation_velocity = lerp(self.rotation_velocity, target_rotation_velocity, 0.08);
        if self.auto_roll && params.beat_mid {
            self.rotation_velocity += 0.03;
        }
        self.rotation += self.rotation_velocity;

        // Treble controls the tunnel glow
        let glow_target = (0.35 + params.treble * 0.9 + self.glow_bias).clamp(0.1, 1.3);
        self.glow = lerp(self.glow, glow_target, 0.08);

        // Move segments
        for z in &mut self.segments {
            *z -= self.speed;
            if *z < 0.5 {
                *z += 10.0; // Recycle to back
            }
        }
        // Keep sorted for correct rendering order (back to front)
        self.segments.sort_by(|a, b| b.partial_cmp(a).unwrap());
    }

    fn render(&self, grid: &mut GridBuffer) {
        grid.clear();

        let w = grid.width() as f32;
        let h = grid.height() as f32;
        let cx = w / 2.0;
        let cy = h / 2.0;

        // Base square size
        let base_w = w * 0.5 * self.zoom;
        let base_h = h * 0.5 * self.zoom;

        // Draw diagonals (infinite tunnel effect)
        // We can just draw lines from center to corners, rotated
        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();
        let max_dist = w.max(h);

        // 4 corners of the "infinite" end
        let diagonals = [
            (-max_dist, -max_dist),
            (max_dist, -max_dist),
            (max_dist, max_dist),
            (-max_dist, max_dist),
        ];

        let diag_color = self
            .color_scheme
            .get_color((0.2 + self.glow * 0.25).clamp(0.0, 1.0))
            .map(|c| Self::brighten(c, 0.6))
            .unwrap_or(Color::new(70, 70, 70));

        for (dx, dy) in diagonals {
            let rx = dx * cos_r - dy * sin_r;
            let ry = dx * sin_r + dy * cos_r;
            self.draw_line(
                grid,
                cx as i32,
                cy as i32,
                (cx + rx) as i32,
                (cy + ry) as i32,
                '.',
                diag_color,
            );
        }

        // Draw segments and connect them into a proper tunnel mesh.
        let mut previous_corners: Option<[(i32, i32); 4]> = None;
        for z in &self.segments {
            // Color based on depth and treble
            let depth_intensity = (1.0 - (*z / 10.0)).clamp(0.15, 1.0);
            let color = self
                .color_scheme
                .get_color((depth_intensity * 0.85 + self.glow * 0.15).clamp(0.0, 1.0))
                .map(|c| Self::brighten(c, 0.8 + self.glow * 0.35))
                .unwrap_or(Color::new(220, 220, 220));

            if let Some(corners) = self.project_square(*z, base_w, base_h, cx, cy) {
                let edge_char = Self::depth_char(*z);
                for i in 0..4 {
                    let (x0, y0) = corners[i];
                    let (x1, y1) = corners[(i + 1) % 4];
                    self.draw_line(grid, x0, y0, x1, y1, edge_char, color);
                }

                if let Some(prev) = previous_corners {
                    let connector_color = Self::brighten(color, 0.7);
                    for i in 0..4 {
                        let (x0, y0) = prev[i];
                        let (x1, y1) = corners[i];
                        self.draw_line(grid, x0, y0, x1, y1, '+', connector_color);
                    }
                }

                previous_corners = Some(corners);
            }
        }
    }

    fn name(&self) -> &str {
        "Grid Tunnel"
    }
}

#[cfg(test)]
mod tests {
    use super::GridTunnelVisualizer;
    use crate::visualization::color_schemes::{ColorScheme, ColorSchemeType};

    #[test]
    fn project_square_rejects_near_plane() {
        let viz = GridTunnelVisualizer::new(ColorScheme::new(ColorSchemeType::Monochrome));
        assert!(viz.project_square(0.05, 10.0, 8.0, 40.0, 12.0).is_none());
    }

    #[test]
    fn project_square_returns_four_corners() {
        let viz = GridTunnelVisualizer::new(ColorScheme::new(ColorSchemeType::Monochrome));
        let corners = viz.project_square(2.0, 10.0, 8.0, 40.0, 12.0).unwrap();
        assert_eq!(corners.len(), 4);
        assert_ne!(corners[0], corners[2]);
    }

    #[test]
    fn grid_tunnel_zoom_controls_clamp() {
        let mut viz = GridTunnelVisualizer::new(ColorScheme::new(ColorSchemeType::Monochrome));
        for _ in 0..20 {
            viz.zoom_in();
        }
        assert!(viz.zoom <= 1.8);
        for _ in 0..40 {
            viz.zoom_out();
        }
        assert!(viz.zoom >= 0.6);
    }

    #[test]
    fn grid_tunnel_auto_roll_toggle_flips_state() {
        let mut viz = GridTunnelVisualizer::new(ColorScheme::new(ColorSchemeType::Monochrome));
        assert!(viz.auto_roll);
        assert!(!viz.toggle_auto_roll());
        assert!(viz.toggle_auto_roll());
    }
}
