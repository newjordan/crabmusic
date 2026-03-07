use super::{lerp, BrailleGrid, Color, GridBuffer, Visualizer};
use crate::dsp::AudioParameters;
use crate::visualization::color_schemes::ColorScheme;
use std::f32::consts::TAU;

#[derive(Debug, Clone, Copy)]
struct BallState {
    x: f32,
    z: f32,
    vx: f32,
    vz: f32,
    plunge: f32,
    age: u32,
}

pub struct GravityWellVisualizer {
    color_scheme: ColorScheme,
    ball: BallState,
    spawn_index: u32,
    respawn_timer: u8,
    bass: f32,
    mid: f32,
    treble: f32,
    amplitude: f32,
    beat_flash: f32,
    grid_twist: f32,
}

impl GravityWellVisualizer {
    pub fn new(color_scheme: ColorScheme) -> Self {
        let mut visualizer = Self {
            color_scheme,
            ball: BallState {
                x: 0.0,
                z: 0.0,
                vx: 0.0,
                vz: 0.0,
                plunge: 0.0,
                age: 0,
            },
            spawn_index: 0,
            respawn_timer: 0,
            bass: 0.0,
            mid: 0.0,
            treble: 0.0,
            amplitude: 0.0,
            beat_flash: 0.0,
            grid_twist: 0.0,
        };
        visualizer.respawn_ball();
        visualizer
    }

    pub fn set_color_scheme(&mut self, color_scheme: ColorScheme) {
        self.color_scheme = color_scheme;
    }

    fn respawn_ball(&mut self) {
        let pattern = self.spawn_index % 4;
        let angle = (self.spawn_index as f32 * 2.399_963_1) % TAU;
        let radius = 5.6 + (self.spawn_index % 5) as f32 * 0.55;
        let x = radius * angle.cos();
        let z = radius * angle.sin();
        let inward = 0.010 + pattern as f32 * 0.002;
        let tangent = match pattern {
            0 => 0.022,
            1 => 0.056,
            2 => 0.040,
            _ => 0.032,
        };
        let swirl = if self.spawn_index.is_multiple_of(2) {
            1.0
        } else {
            -1.0
        };
        let radial_x = -x / radius.max(0.001);
        let radial_z = -z / radius.max(0.001);
        let tangential_x = -z / radius.max(0.001) * swirl;
        let tangential_z = x / radius.max(0.001) * swirl;

        self.ball = BallState {
            x,
            z,
            vx: radial_x * inward + tangential_x * tangent,
            vz: radial_z * inward + tangential_z * tangent,
            plunge: 0.0,
            age: 0,
        };
        self.spawn_index = self.spawn_index.wrapping_add(1);
    }

    fn horizon_radius(&self) -> f32 {
        0.72 + self.bass * 0.55 + self.beat_flash * 0.18
    }

    fn well_depth(&self) -> f32 {
        1.2 + self.bass * 2.4 + self.amplitude * 1.0 + self.beat_flash * 0.8
    }

    fn radial_distance(&self) -> f32 {
        (self.ball.x * self.ball.x + self.ball.z * self.ball.z).sqrt()
    }

    fn funnel_depth_at(&self, radius: f32) -> f32 {
        self.well_depth() / (radius * radius + 0.55)
    }

    fn ball_depth(&self) -> f32 {
        let radius = self.radial_distance();
        self.funnel_depth_at(radius) + self.ball.plunge * 1.9
    }

    fn warp_point(&self, x: f32, z: f32) -> Option<(f32, f32)> {
        let radius = (x * x + z * z).sqrt();
        let horizon = self.horizon_radius();
        if radius <= horizon * 0.78 {
            return None;
        }

        let sink = (self.well_depth() / (radius * radius + 1.0)).clamp(0.0, 0.55);
        let twist = self.grid_twist * sink * 1.9;
        let theta = z.atan2(x) + twist;
        let warped_radius = (radius * (1.0 - sink * 0.42)).max(horizon * 0.82);
        Some((warped_radius * theta.cos(), warped_radius * theta.sin()))
    }

    fn world_to_top_dot(
        &self,
        x: f32,
        z: f32,
        left_width: usize,
        height: usize,
    ) -> Option<(usize, usize)> {
        let scale = left_width.min(height) as f32 * 0.075;
        let sx = left_width as f32 * 0.5 + x * scale;
        let sy = height as f32 * 0.5 + z * scale * 0.78;
        let ix = sx.round() as i32;
        let iy = sy.round() as i32;
        if ix >= 0 && iy >= 0 && (ix as usize) < left_width && (iy as usize) < height {
            Some((ix as usize, iy as usize))
        } else {
            None
        }
    }

    fn side_to_dot(
        &self,
        offset_x: usize,
        width: usize,
        height: usize,
        x: f32,
        depth: f32,
    ) -> Option<(usize, usize)> {
        if width == 0 || height == 0 {
            return None;
        }
        let half = width as f32 * 0.5;
        let sx = offset_x as f32 + half + x * (width as f32 * 0.13);
        let sy = height as f32 * 0.16 + depth * (height as f32 * 0.22);
        let ix = sx.round() as i32;
        let iy = sy.round() as i32;
        if ix >= offset_x as i32
            && iy >= 0
            && (ix as usize) < offset_x + width
            && (iy as usize) < height
        {
            Some((ix as usize, iy as usize))
        } else {
            None
        }
    }

    fn color_at(&self, intensity: f32) -> Color {
        self.color_scheme
            .get_color(intensity)
            .unwrap_or_else(|| Color::new(180, 180, 180))
    }

    fn scale_color(color: Color, factor: f32) -> Color {
        let factor = factor.max(0.0);
        Color::new(
            (color.r as f32 * factor).clamp(0.0, 255.0) as u8,
            (color.g as f32 * factor).clamp(0.0, 255.0) as u8,
            (color.b as f32 * factor).clamp(0.0, 255.0) as u8,
        )
    }

    fn draw_top_grid(&self, braille: &mut BrailleGrid, left_width: usize, height: usize) {
        let base_color =
            self.color_at((0.25 + self.treble * 0.45 + self.beat_flash * 0.25).min(1.0));
        let grid_extent = 8.5;
        let step = 1.0;
        let samples = 40;

        for line in -8..=8 {
            let grid_value = line as f32 * step;
            for axis in 0..2 {
                let mut previous = None;
                for sample in 0..=samples {
                    let t = -grid_extent + 2.0 * grid_extent * sample as f32 / samples as f32;
                    let (x, z) = if axis == 0 {
                        (grid_value, t)
                    } else {
                        (t, grid_value)
                    };
                    let warped = self.warp_point(x, z);
                    let current = warped
                        .and_then(|(wx, wz)| self.world_to_top_dot(wx, wz, left_width, height));
                    if let (Some((px, py)), Some((cx, cy))) = (previous, current) {
                        braille.draw_line_with_color(
                            px,
                            py,
                            cx,
                            cy,
                            Self::scale_color(base_color, 0.78),
                        );
                    }
                    previous = current;
                }
            }
        }

        let ring_color =
            self.color_at((0.55 + self.treble * 0.35 + self.beat_flash * 0.4).min(1.0));
        let ring_radius = self.horizon_radius() * 0.95;
        let mut previous = None;
        for step_idx in 0..=72 {
            let theta = TAU * step_idx as f32 / 72.0;
            let point = self.world_to_top_dot(
                ring_radius * theta.cos(),
                ring_radius * theta.sin(),
                left_width,
                height,
            );
            if let (Some((px, py)), Some((cx, cy))) = (previous, point) {
                braille.draw_line_with_color(px, py, cx, cy, ring_color);
            }
            previous = point;
        }
    }

    fn draw_ball_top(&self, braille: &mut BrailleGrid, left_width: usize, height: usize) {
        if let Some((wx, wz)) = self.warp_point(self.ball.x, self.ball.z) {
            if let Some((x, y)) = self.world_to_top_dot(wx, wz, left_width, height) {
                let color = self.color_at((0.7 + self.treble * 0.3).min(1.0));
                for oy in y.saturating_sub(1)..=(y + 1).min(braille.dot_height().saturating_sub(1))
                {
                    for ox in
                        x.saturating_sub(1)..=(x + 1).min(braille.dot_width().saturating_sub(1))
                    {
                        if ox < left_width {
                            braille.set_dot_with_color(ox, oy, color);
                        }
                    }
                }
            }
        }
    }

    fn draw_side_view(
        &self,
        braille: &mut BrailleGrid,
        offset_x: usize,
        width: usize,
        height: usize,
    ) {
        if width < 6 {
            return;
        }

        let grid_color =
            Self::scale_color(self.color_at((0.18 + self.treble * 0.25).min(1.0)), 0.85);
        for line in 0..=4 {
            let x = offset_x + (line * width.saturating_sub(1) / 4);
            braille.draw_line_with_color(x, 0, x, height.saturating_sub(1), grid_color);
        }
        for line in 1..=5 {
            let y = line * height.saturating_sub(1) / 6;
            braille.draw_line_with_color(
                offset_x,
                y,
                offset_x + width.saturating_sub(1),
                y,
                grid_color,
            );
        }

        let funnel_color = self.color_at((0.45 + self.mid * 0.25 + self.beat_flash * 0.2).min(1.0));
        let mut prev_left = None;
        let mut prev_right = None;
        for step in 0..=48 {
            let x = 4.4 * (1.0 - step as f32 / 48.0).powf(0.82);
            let depth = self.funnel_depth_at(x) * 1.15;
            let left = self.side_to_dot(offset_x, width, height, -x, depth);
            let right = self.side_to_dot(offset_x, width, height, x, depth);
            if let (Some((px, py)), Some((cx, cy))) = (prev_left, left) {
                braille.draw_line_with_color(px, py, cx, cy, funnel_color);
            }
            if let (Some((px, py)), Some((cx, cy))) = (prev_right, right) {
                braille.draw_line_with_color(px, py, cx, cy, funnel_color);
            }
            prev_left = left;
            prev_right = right;
        }

        let shaft_color = Self::scale_color(funnel_color, 0.72);
        if let (Some((top_x, top_y)), Some((bottom_x, bottom_y))) = (
            self.side_to_dot(
                offset_x,
                width,
                height,
                0.0,
                self.funnel_depth_at(0.18) * 1.05,
            ),
            self.side_to_dot(offset_x, width, height, 0.0, self.ball_depth() + 2.4),
        ) {
            braille.draw_line_with_color(top_x, top_y, bottom_x, bottom_y, shaft_color);
        }

        if let Some((ball_x, ball_y)) = self.side_to_dot(
            offset_x,
            width,
            height,
            self.ball.x * 0.42,
            self.ball_depth(),
        ) {
            let color = self.color_at((0.74 + self.treble * 0.26).min(1.0));
            for oy in
                ball_y.saturating_sub(1)..=(ball_y + 1).min(braille.dot_height().saturating_sub(1))
            {
                for ox in ball_x.saturating_sub(1)
                    ..=(ball_x + 1).min((offset_x + width).saturating_sub(1))
                {
                    if ox >= offset_x {
                        braille.set_dot_with_color(ox, oy, color);
                    }
                }
            }
        }
    }

    fn blit_braille(&self, braille: &BrailleGrid, grid: &mut GridBuffer) {
        for cell_y in 0..grid.height() {
            for cell_x in 0..grid.width() {
                let character = braille.get_char(cell_x, cell_y);
                let color = braille.get_color(cell_x, cell_y);
                if character != ' ' && character != '⠀' {
                    let cell = grid.get_cell_mut(cell_x, cell_y);
                    cell.character = character;
                    cell.foreground_color = color;
                }
            }
        }
    }
}

impl Visualizer for GravityWellVisualizer {
    fn update(&mut self, params: &AudioParameters) {
        self.bass = lerp(self.bass, params.bass, 0.10);
        self.mid = lerp(self.mid, params.mid, 0.10);
        self.treble = lerp(self.treble, params.treble, 0.12);
        self.amplitude = lerp(self.amplitude, params.amplitude, 0.08);
        self.grid_twist = lerp(
            self.grid_twist,
            self.grid_twist + (self.mid - 0.5) * 0.015,
            0.08,
        );
        self.beat_flash = (self.beat_flash * 0.9).max(0.0);
        if params.beat_bass || params.beat {
            self.beat_flash = 1.0;
        }

        if self.respawn_timer > 0 {
            self.respawn_timer -= 1;
            if self.respawn_timer == 0 {
                self.respawn_ball();
            }
            return;
        }

        let radius = self.radial_distance();
        let gravity = 0.010 + self.bass * 0.022 + self.amplitude * 0.012;
        let soft = radius * radius + 0.75;
        let ax = -self.ball.x * gravity / soft;
        let az = -self.ball.z * gravity / soft;
        let swirl = 0.0012 + self.mid * 0.0045;
        let tangential_x = -self.ball.z * swirl / (radius + 1.0);
        let tangential_z = self.ball.x * swirl / (radius + 1.0);

        self.ball.vx = (self.ball.vx + ax + tangential_x) * 0.996;
        self.ball.vz = (self.ball.vz + az + tangential_z) * 0.996;
        self.ball.x += self.ball.vx;
        self.ball.z += self.ball.vz;
        self.ball.age = self.ball.age.saturating_add(1);

        let new_radius = self.radial_distance();
        let horizon = self.horizon_radius();
        if new_radius < horizon {
            self.ball.x *= 0.92;
            self.ball.z *= 0.92;
            self.ball.vx *= 0.88;
            self.ball.vz *= 0.88;
            self.ball.plunge += 0.12 + self.bass * 0.12;
            if self.ball.plunge > 1.35 {
                self.respawn_timer = 18;
            }
        } else {
            self.ball.plunge = lerp(self.ball.plunge, 0.0, 0.05);
        }

        if self.ball.age > 900 || new_radius > 10.0 {
            self.respawn_timer = 6;
        }
    }

    fn render(&self, grid: &mut GridBuffer) {
        grid.clear();

        let width = grid.width();
        let height = grid.height();
        if width < 24 || height < 10 {
            return;
        }

        let side_cells = ((width as f32) * 0.28).round() as usize;
        let side_cells = side_cells.clamp(12, width.saturating_sub(12));
        let divider_cell = width.saturating_sub(side_cells + 1);
        let left_cells = divider_cell;

        let mut braille = BrailleGrid::new(width, height);
        let left_dot_width = left_cells * 2;
        let side_dot_offset = (divider_cell + 1) * 2;
        let side_dot_width = side_cells * 2;
        let dot_height = braille.dot_height();

        self.draw_top_grid(&mut braille, left_dot_width, dot_height);
        self.draw_ball_top(&mut braille, left_dot_width, dot_height);
        self.draw_side_view(&mut braille, side_dot_offset, side_dot_width, dot_height);
        self.blit_braille(&braille, grid);

        let divider_color =
            Self::scale_color(self.color_at((0.32 + self.treble * 0.2).min(1.0)), 0.75);
        for y in 0..height {
            grid.set_cell_with_color(divider_cell, y, '│', divider_color);
        }
    }

    fn name(&self) -> &str {
        "Gravity Well"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visualization::color_schemes::ColorSchemeType;

    #[test]
    fn funnel_depth_increases_toward_center() {
        let viz = GravityWellVisualizer::new(ColorScheme::new(ColorSchemeType::GreenYellow));
        assert!(viz.funnel_depth_at(0.8) > viz.funnel_depth_at(2.4));
    }

    #[test]
    fn gravity_well_renders_non_empty_frame() {
        let mut viz = GravityWellVisualizer::new(ColorScheme::new(ColorSchemeType::GreenYellow));
        let mut params = AudioParameters::default();
        params.bass = 0.8;
        params.mid = 0.6;
        params.treble = 0.5;
        params.amplitude = 0.7;
        params.beat_bass = true;
        viz.update(&params);

        let mut grid = GridBuffer::new(80, 32);
        viz.render(&mut grid);

        let mut active_cells = 0;
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                if grid.get_cell(x, y).character != ' ' {
                    active_cells += 1;
                }
            }
        }
        assert!(
            active_cells > 40,
            "expected a visible frame, got {active_cells} active cells"
        );
    }

    #[test]
    fn gravity_well_name_is_stable() {
        let viz = GravityWellVisualizer::new(ColorScheme::new(ColorSchemeType::Monochrome));
        assert_eq!(viz.name(), "Gravity Well");
    }
}
