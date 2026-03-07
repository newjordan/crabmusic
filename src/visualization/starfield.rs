use crate::dsp::AudioParameters;
use crate::visualization::{lerp, Color, GridBuffer, Visualizer};
use rand::Rng;

/// A single star in the 3D field
struct Star {
    x: f32,
    y: f32,
    z: f32,
    prev_z: f32,
    color: Color,
}

impl Star {
    fn new(rng: &mut impl Rng) -> Self {
        Self {
            x: rng.gen_range(-1.0..1.0),
            y: rng.gen_range(-1.0..1.0),
            z: rng.gen_range(1.0..2.0), // Start further away
            prev_z: 2.0,
            color: Color::new(255, 255, 255),
        }
    }
}

/// Starfield visualizer
///
/// A 3D starfield that reacts to music beats.
/// - Bass: "Warp speed" effect (stars move faster)
/// - Mid: Rotates the camera/field
/// - Treble: Spawns twinkling stars or changes colors
pub struct StarfieldVisualizer {
    stars: Vec<Star>,
    speed: f32,
    speed_bias: f32,
    rotation: f32,
    rotation_velocity: f32,
    warp_flash: f32,
    projection_scale: f32,
    trail_gain: f32,
    auto_rotate: bool,
    color_scheme: crate::visualization::color_schemes::ColorScheme,
}

impl StarfieldVisualizer {
    pub fn new(color_scheme: crate::visualization::color_schemes::ColorScheme) -> Self {
        let mut rng = rand::thread_rng();
        let stars = (0..400).map(|_| Star::new(&mut rng)).collect();

        Self {
            stars,
            speed: 0.02,
            speed_bias: 0.0,
            rotation: 0.0,
            rotation_velocity: 0.005,
            warp_flash: 0.0,
            projection_scale: 0.9,
            trail_gain: 1.0,
            auto_rotate: true,
            color_scheme,
        }
    }

    pub fn set_color_scheme(&mut self, scheme: crate::visualization::color_schemes::ColorScheme) {
        self.color_scheme = scheme;
    }

    pub fn toggle_auto_rotate(&mut self) -> bool {
        self.auto_rotate = !self.auto_rotate;
        self.auto_rotate
    }

    pub fn rotate_left(&mut self, step: f32) {
        self.rotation -= step;
    }

    pub fn rotate_right(&mut self, step: f32) {
        self.rotation += step;
    }

    pub fn speed_down(&mut self) -> f32 {
        self.speed_bias = (self.speed_bias - 0.012).max(-0.02);
        self.speed_bias
    }

    pub fn speed_up(&mut self) -> f32 {
        self.speed_bias = (self.speed_bias + 0.012).min(0.14);
        self.speed_bias
    }

    pub fn zoom_in(&mut self) -> f32 {
        self.projection_scale = (self.projection_scale * 1.1).min(1.8);
        self.projection_scale
    }

    pub fn zoom_out(&mut self) -> f32 {
        self.projection_scale = (self.projection_scale / 1.1).max(0.45);
        self.projection_scale
    }

    pub fn trails_down(&mut self) -> f32 {
        self.trail_gain = (self.trail_gain - 0.15).max(0.25);
        self.trail_gain
    }

    pub fn trails_up(&mut self) -> f32 {
        self.trail_gain = (self.trail_gain + 0.15).min(2.0);
        self.trail_gain
    }

    fn project_star(
        x: f32,
        y: f32,
        z: f32,
        rotation: f32,
        width: f32,
        height: f32,
        projection_scale: f32,
    ) -> Option<(i32, i32)> {
        if z <= 0.05 {
            return None;
        }

        let rx = x * rotation.cos() - y * rotation.sin();
        let ry = x * rotation.sin() + y * rotation.cos();

        let cx = width / 2.0;
        let cy = height / 2.0;
        let scale = width.min(height) * projection_scale;
        let sx = cx + (rx / z) * scale;
        let sy = cy + (ry / z) * scale * 0.55;

        if sx >= 0.0 && sx < width && sy >= 0.0 && sy < height {
            Some((sx as i32, sy as i32))
        } else {
            None
        }
    }

    fn dim_color(color: Color, factor: f32) -> Color {
        let factor = factor.clamp(0.0, 1.0);
        Color::new(
            (color.r as f32 * factor) as u8,
            (color.g as f32 * factor) as u8,
            (color.b as f32 * factor) as u8,
        )
    }

    fn draw_line(
        grid: &mut GridBuffer,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        ch: char,
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
                grid.set_cell_with_color(x as usize, y as usize, ch, color);
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

    fn head_char(z: f32, warp_flash: f32) -> char {
        if z < 0.35 || warp_flash > 0.75 {
            '@'
        } else if z < 0.65 {
            'O'
        } else if z < 1.0 {
            'o'
        } else if z < 1.45 {
            '*'
        } else {
            '.'
        }
    }
}

impl Visualizer for StarfieldVisualizer {
    fn update(&mut self, params: &AudioParameters) {
        let mut rng = rand::thread_rng();

        // Base speed from amplitude
        let target_speed =
            (0.015 + (params.amplitude * 0.07) + (params.bass * 0.03) + self.speed_bias)
                .clamp(0.008, 0.18);

        // Bass beat triggers "warp speed"
        if params.beat_bass {
            self.speed = (self.speed + 0.18).min(0.28);
            self.warp_flash = 1.0;
        } else {
            // Decay speed back to target
            self.speed = lerp(self.speed, target_speed, 0.12);
            self.warp_flash = lerp(self.warp_flash, 0.0, 0.1);
        }

        // Mid beat affects rotation
        let target_rotation_velocity = if self.auto_rotate {
            0.004 + params.mid * 0.03
        } else {
            0.0
        };
        self.rotation_velocity = lerp(self.rotation_velocity, target_rotation_velocity, 0.08);
        if self.auto_rotate && params.beat_mid {
            self.rotation_velocity += 0.012;
        }
        self.rotation += self.rotation_velocity;

        // Treble beat changes star colors or brightness
        let treble_boost = if params.beat_treble { 0.25 } else { 0.0 };

        for star in &mut self.stars {
            // Move star towards camera (decrease Z)
            star.prev_z = star.z;
            star.z -= self.speed;

            // Reset if behind camera
            if star.z <= 0.0 {
                star.z = rng.gen_range(1.6..2.8);
                star.prev_z = star.z;
                star.x = rng.gen_range(-1.0..1.0);
                star.y = rng.gen_range(-1.0..1.0);
            }

            // Update color based on position or beat
            // Use color scheme based on Z depth (closer = brighter/different color)
            let intensity = (1.0 - (star.z / 2.0)).clamp(0.0, 1.0);
            // Boost intensity with treble
            let final_intensity = (intensity + treble_boost + self.warp_flash * 0.2).min(1.0);

            if let Some(c) = self.color_scheme.get_color(final_intensity) {
                star.color = c;
            } else {
                let brightness = (140.0 + final_intensity * 115.0) as u8;
                star.color = Color::new(brightness, brightness, brightness);
            }
        }
    }

    fn render(&self, grid: &mut GridBuffer) {
        grid.clear();

        let width = grid.width() as f32;
        let height = grid.height() as f32;

        if self.warp_flash > 0.05 {
            let cx = grid.width() / 2;
            let cy = grid.height() / 2;
            let flash_color = self
                .color_scheme
                .get_color((0.7 + self.warp_flash * 0.3).min(1.0))
                .unwrap_or(Color::new(220, 220, 220));
            let spokes = [
                (cx, cy, '@'),
                (cx.saturating_sub(1), cy, '+'),
                ((cx + 1).min(grid.width().saturating_sub(1)), cy, '+'),
                (cx, cy.saturating_sub(1), '+'),
                (cx, (cy + 1).min(grid.height().saturating_sub(1)), '+'),
            ];
            for (x, y, ch) in spokes {
                grid.set_cell_with_color(x, y, ch, flash_color);
            }
        }

        for star in &self.stars {
            let current = Self::project_star(
                star.x,
                star.y,
                star.z,
                self.rotation,
                width,
                height,
                self.projection_scale,
            );
            let previous = Self::project_star(
                star.x,
                star.y,
                star.prev_z,
                self.rotation - self.rotation_velocity,
                width,
                height,
                self.projection_scale,
            );

            if let (Some((px, py)), Some((x, y))) = (previous, current) {
                let trail_strength = (((star.prev_z - star.z) * 8.0 + self.warp_flash * 0.5)
                    * self.trail_gain)
                    .clamp(0.12, 0.95);
                Self::draw_line(
                    grid,
                    px,
                    py,
                    x,
                    y,
                    ':',
                    Self::dim_color(star.color, trail_strength),
                );
            }

            if let Some((x, y)) = current {
                grid.set_cell_with_color(
                    x as usize,
                    y as usize,
                    Self::head_char(star.z, self.warp_flash),
                    star.color,
                );
            }
        }
    }

    fn name(&self) -> &str {
        "Starfield"
    }
}

#[cfg(test)]
mod tests {
    use super::StarfieldVisualizer;

    #[test]
    fn project_star_rejects_near_plane() {
        assert_eq!(
            StarfieldVisualizer::project_star(0.0, 0.0, 0.0, 0.0, 80.0, 24.0, 0.9),
            None
        );
    }

    #[test]
    fn project_star_centers_origin() {
        assert_eq!(
            StarfieldVisualizer::project_star(0.0, 0.0, 1.0, 0.0, 80.0, 24.0, 0.9),
            Some((40, 12))
        );
    }

    #[test]
    fn starfield_zoom_controls_clamp() {
        let mut viz =
            StarfieldVisualizer::new(crate::visualization::color_schemes::ColorScheme::new(
                crate::visualization::color_schemes::ColorSchemeType::Monochrome,
            ));
        for _ in 0..20 {
            viz.zoom_in();
        }
        assert!(viz.projection_scale <= 1.8);
        for _ in 0..30 {
            viz.zoom_out();
        }
        assert!(viz.projection_scale >= 0.45);
    }

    #[test]
    fn starfield_trail_controls_clamp() {
        let mut viz =
            StarfieldVisualizer::new(crate::visualization::color_schemes::ColorScheme::new(
                crate::visualization::color_schemes::ColorSchemeType::Monochrome,
            ));
        for _ in 0..20 {
            viz.trails_down();
        }
        assert!(viz.trail_gain >= 0.25);
        for _ in 0..20 {
            viz.trails_up();
        }
        assert!(viz.trail_gain <= 2.0);
    }
}
