// Insult Fireworks Scene - braille-based light show with looping insults

use crabmusic::rendering::TerminalRenderer;
use crabmusic::visualization::{BrailleGrid, Color, GridBuffer};
use crossterm::event::{self, Event, KeyCode};
use std::f32::consts::{PI, TAU};
use std::thread;
use std::time::{Duration, Instant};

const LOOP_DURATION: f32 = 30.0;
const FIREWORK_DURATION: f32 = 5.0;
const LAUNCH_DURATION: f32 = 1.2;
const FRAME_TIME: Duration = Duration::from_millis(16);

#[derive(Clone, Copy)]
struct SceneDims {
    dot_width: usize,
    dot_height: usize,
    aspect: f32,
}

impl SceneDims {
    fn new(width: usize, height: usize) -> Self {
        let dot_width = width.saturating_mul(2);
        let dot_height = height.saturating_mul(4);
        let aspect = if dot_width == 0 {
            1.0
        } else {
            dot_height as f32 / dot_width as f32
        };
        Self {
            dot_width,
            dot_height,
            aspect,
        }
    }

    fn min_dot(&self) -> usize {
        self.dot_width.min(self.dot_height)
    }
}

#[derive(Clone, Copy)]
struct TextOverlay {
    text: &'static str,
    column: isize,
    row: isize,
    color: Color,
    intensity: f32,
}

#[derive(Clone, Copy)]
enum FireworkStyle {
    Spiral { arms: usize, swirl_turns: f32 },
    Ribbon { ribbons: usize, wave_count: f32 },
    Cascade { strands: usize, arch_height: f32 },
}

#[derive(Clone, Copy)]
struct FireworkEvent {
    start_time: f32,
    center: (f32, f32),
    launch_x: f32,
    primary_color: Color,
    accent_color: Color,
    insult: &'static str,
    style: FireworkStyle,
    radius_factor: f32,
    seed: u64,
}

impl FireworkEvent {
    fn render(
        &self,
        show_time: f32,
        dims: &SceneDims,
        braille: &mut BrailleGrid,
        overlays: &mut Vec<TextOverlay>,
    ) {
        let diff = (show_time - self.start_time).rem_euclid(LOOP_DURATION);
        if diff >= FIREWORK_DURATION {
            return;
        }

        let center = (
            self.center.0 * dims.dot_width as f32,
            self.center.1 * dims.dot_height as f32,
        );

        let launch_progress = (diff / LAUNCH_DURATION).clamp(0.0, 1.0);
        self.render_launch(launch_progress, dims, braille, center);

        if diff <= LAUNCH_DURATION {
            return;
        }

        let explosion_progress =
            ((diff - LAUNCH_DURATION) / (FIREWORK_DURATION - LAUNCH_DURATION)).clamp(0.0, 1.0);

        self.render_core_glow(explosion_progress, dims, braille, center);

        match self.style {
            FireworkStyle::Spiral { arms, swirl_turns } => {
                self.render_spiral(arms, swirl_turns, explosion_progress, dims, braille, center);
            }
            FireworkStyle::Ribbon {
                ribbons,
                wave_count,
            } => {
                self.render_ribbon(
                    ribbons,
                    wave_count,
                    explosion_progress,
                    dims,
                    braille,
                    center,
                );
            }
            FireworkStyle::Cascade {
                strands,
                arch_height,
            } => {
                self.render_cascade(
                    strands,
                    arch_height,
                    explosion_progress,
                    dims,
                    braille,
                    center,
                );
            }
        }

        self.render_starburst(explosion_progress, dims, braille, center);
        self.render_embers(explosion_progress, dims, braille, center);
        self.push_insult(explosion_progress, overlays, center);
    }

    fn base_angle(&self) -> f32 {
        (self.seed as f32 * 0.618_033_9).rem_euclid(TAU)
    }

    fn seed_phase(&self) -> f32 {
        (self.seed as f32 * 1.327_684).rem_euclid(TAU)
    }

    fn render_launch(
        &self,
        progress: f32,
        dims: &SceneDims,
        braille: &mut BrailleGrid,
        center: (f32, f32),
    ) {
        if progress <= 0.0 || dims.dot_width == 0 || dims.dot_height == 0 {
            return;
        }

        let start_x = self.launch_x * dims.dot_width as f32;
        let start_y = dims.dot_height as f32 + 10.0;
        let steps = 100;

        for i in 0..steps {
            let t = i as f32 / (steps - 1) as f32;
            if t > progress {
                break;
            }

            let eased = smoothstep(t);
            let arc = (t * PI * 2.0 + self.seed_phase()).sin() * (6.0 * (1.0 - eased));
            let x = lerp(start_x, center.0, eased) + arc;
            let y = lerp(start_y, center.1, eased);
            let tail_color = scale_color(self.primary_color, 0.8 + 0.6 * (1.0 - t));
            paint_dot(braille, dims, x, y, tail_color);

            let flicker_offset = (t * 18.0 + self.seed_phase()).cos() * 3.0;
            let flicker_y = y + 3.0;
            let flicker_color = scale_color(self.accent_color, 0.5 + 0.5 * (1.0 - t));
            paint_dot(braille, dims, x + flicker_offset, flicker_y, flicker_color);
        }
    }

    fn render_core_glow(
        &self,
        progress: f32,
        dims: &SceneDims,
        braille: &mut BrailleGrid,
        center: (f32, f32),
    ) {
        if progress <= 0.0 {
            return;
        }

        for dx in -3..=3 {
            for dy in -3..=3 {
                let distance = ((dx * dx + dy * dy) as f32).sqrt();
                if distance <= 3.4 {
                    let falloff = (1.0 - distance / 3.4).max(0.0);
                    let color =
                        scale_color(self.accent_color, 1.2 * falloff + 0.6 * (1.0 - progress));
                    paint_dot(
                        braille,
                        dims,
                        center.0 + dx as f32,
                        center.1 + dy as f32,
                        color,
                    );
                }
            }
        }

        let ring_radius = dims.min_dot() as f32 * self.radius_factor * (0.18 + progress * 0.35);
        let ring_points = 96;
        for i in 0..ring_points {
            let angle = i as f32 / ring_points as f32 * TAU;
            let x = center.0 + ring_radius * angle.cos();
            let y = center.1 + ring_radius * angle.sin();
            let color = scale_color(
                blend_color(self.primary_color, self.accent_color, 0.5),
                0.6 + 0.4 * (1.0 - progress),
            );
            paint_dot(braille, dims, x, y, color);
        }
    }

    fn render_spiral(
        &self,
        arms: usize,
        swirl_turns: f32,
        progress: f32,
        dims: &SceneDims,
        braille: &mut BrailleGrid,
        center: (f32, f32),
    ) {
        if progress <= 0.0 || arms == 0 {
            return;
        }

        let max_radius = dims.min_dot() as f32 * self.radius_factor;
        let base_angle = self.base_angle();
        let samples = 240;

        for arm in 0..arms {
            let arm_angle = base_angle + (arm as f32 / arms as f32) * TAU;
            let mut tip = center;

            for i in 0..samples {
                let t = i as f32 / (samples - 1) as f32;
                if t > progress {
                    break;
                }

                let eased = ease_out_cubic(t);
                let swirl = swirl_turns * eased * progress;
                let angle = arm_angle + swirl * TAU + self.seed_phase() * 0.25;
                let wobble = (t * 12.0 + progress * 8.0 + arm as f32).sin() * 8.0 * (1.0 - eased);
                let radius = eased * max_radius + wobble;
                let x = center.0 + radius * angle.cos();
                let y = center.1 + (radius * angle.sin()) / dims.aspect.max(0.35);
                let color_mix = blend_color(self.primary_color, self.accent_color, eased);
                let color = scale_color(color_mix, 0.7 + 0.6 * (1.0 - t));
                paint_dot(braille, dims, x, y, color);
                tip = (x, y);
            }

            draw_line(
                braille,
                dims,
                center.0,
                center.1,
                tip.0,
                tip.1,
                scale_color(self.primary_color, 0.4 + 0.4 * (1.0 - progress)),
            );
        }
    }

    fn render_ribbon(
        &self,
        ribbons: usize,
        wave_count: f32,
        progress: f32,
        dims: &SceneDims,
        braille: &mut BrailleGrid,
        center: (f32, f32),
    ) {
        if progress <= 0.0 || ribbons == 0 {
            return;
        }

        let max_radius = dims.min_dot() as f32 * self.radius_factor * (0.8 + 0.4 * progress);
        let base_angle = self.base_angle();
        let samples = 260;

        for ribbon in 0..ribbons {
            let ribbon_offset = ribbon as f32 / ribbons as f32;
            let offset_angle = ribbon_offset * TAU;
            let mut last_point = center;

            for i in 0..samples {
                let t = i as f32 / (samples - 1) as f32;
                if t > progress {
                    break;
                }

                let eased = ease_in_out_sine(t);
                let wave =
                    (t * wave_count * TAU + progress * TAU + offset_angle + self.seed_phase())
                        .sin();
                let radius = eased * max_radius + wave * 10.0;
                let angle = base_angle + offset_angle + progress * 1.3 + wave * 0.35;
                let x = center.0 + radius * angle.cos();
                let y = center.1 + radius * angle.sin();
                let color_mix =
                    blend_color(self.primary_color, self.accent_color, wave * 0.5 + 0.5);
                let color = scale_color(color_mix, 0.6 + 0.5 * (1.0 - t));
                paint_dot(braille, dims, x, y, color);

                if i % 8 == 0 {
                    draw_line(
                        braille,
                        dims,
                        last_point.0,
                        last_point.1,
                        x,
                        y,
                        scale_color(color_mix, 0.3 + 0.5 * (1.0 - t)),
                    );
                    last_point = (x, y);
                }
            }
        }
    }

    fn render_cascade(
        &self,
        strands: usize,
        arch_height: f32,
        progress: f32,
        dims: &SceneDims,
        braille: &mut BrailleGrid,
        center: (f32, f32),
    ) {
        if progress <= 0.0 || strands == 0 {
            return;
        }

        let max_radius = dims.min_dot() as f32 * self.radius_factor;
        let base_angle = self.base_angle();
        let samples = 200;

        for strand in 0..strands {
            let strand_norm = if strands == 1 {
                0.0
            } else {
                strand as f32 / (strands - 1) as f32
            };
            let spread = (strand_norm - 0.5) * 0.9;
            let mut tip = center;

            for i in 0..samples {
                let t = i as f32 / (samples - 1) as f32;
                if t > progress {
                    break;
                }

                let eased = smoothstep(t);
                let radius = eased * max_radius;
                let gravity = arch_height * dims.dot_height as f32 * eased.powf(1.4);
                let angle = base_angle + spread * 0.8 + eased * 0.9;
                let sway = (t * 6.0 + spread * 4.0 + self.seed_phase()).sin() * 6.0 * (1.0 - eased);
                let x = center.0 + radius * angle.cos() + sway;
                let y = center.1 + radius * angle.sin() + gravity;
                let color_mix = blend_color(self.primary_color, self.accent_color, eased.powf(0.8));
                let color = scale_color(color_mix, 0.7 + 0.3 * (1.0 - t));
                paint_dot(braille, dims, x, y, color);
                tip = (x, y);
            }

            draw_line(
                braille,
                dims,
                center.0,
                center.1,
                tip.0,
                tip.1,
                scale_color(self.primary_color, 0.35 + 0.4 * (1.0 - progress)),
            );
        }
    }

    fn render_starburst(
        &self,
        progress: f32,
        dims: &SceneDims,
        braille: &mut BrailleGrid,
        center: (f32, f32),
    ) {
        if progress <= 0.05 {
            return;
        }

        let rays = 10;
        let radius = dims.min_dot() as f32 * self.radius_factor * (0.25 + progress * 0.9);
        for i in 0..rays {
            let angle = self.base_angle() + (i as f32 / rays as f32) * TAU + progress * 0.8;
            let end_x = center.0 + radius * angle.cos();
            let end_y = center.1 + radius * angle.sin();
            let color = scale_color(self.accent_color, 0.3 + 0.5 * (1.0 - progress));
            draw_line(braille, dims, center.0, center.1, end_x, end_y, color);
        }
    }

    fn render_embers(
        &self,
        progress: f32,
        dims: &SceneDims,
        braille: &mut BrailleGrid,
        center: (f32, f32),
    ) {
        if progress <= 0.05 {
            return;
        }

        let ember_count = 12;
        let drift = progress.powf(1.5) * dims.dot_height as f32 * 0.18;
        let radius = dims.min_dot() as f32 * self.radius_factor * progress.powf(0.8);

        for i in 0..ember_count {
            let offset = i as f32 / ember_count as f32;
            let angle = self.seed_phase() + offset * TAU + progress * 1.6;
            let sway = (offset * 12.0 + progress * 14.0).sin() * 6.0;
            let x = center.0 + radius * angle.cos() + sway;
            let y = center.1 + radius * angle.sin() + drift;
            let color = scale_color(self.primary_color, 0.25 + 0.55 * (1.0 - progress));
            paint_dot(braille, dims, x, y, color);
        }
    }

    fn push_insult(&self, progress: f32, overlays: &mut Vec<TextOverlay>, center: (f32, f32)) {
        let strength = (1.0 - progress).powf(0.6);
        if strength < 0.05 {
            return;
        }

        let column = (center.0 / 2.0).round() as isize;
        let mut row = (center.1 / 4.0).round() as isize - 3;
        if row < 2 {
            row = 2;
        }

        overlays.push(TextOverlay {
            text: self.insult,
            column,
            row,
            color: scale_color(self.accent_color, 0.6 + 0.5 * strength),
            intensity: strength,
        });
    }
}

const FIREWORK_EVENTS: [FireworkEvent; 6] = [
    FireworkEvent {
        start_time: 0.0,
        center: (0.22, 0.38),
        launch_x: 0.18,
        primary_color: Color::new(255, 120, 70),
        accent_color: Color::new(255, 230, 170),
        insult: "Spin harder, you glitter-fueled gremlin!",
        style: FireworkStyle::Spiral {
            arms: 6,
            swirl_turns: 2.2,
        },
        radius_factor: 0.45,
        seed: 13,
    },
    FireworkEvent {
        start_time: 5.0,
        center: (0.52, 0.30),
        launch_x: 0.50,
        primary_color: Color::new(90, 200, 255),
        accent_color: Color::new(180, 120, 255),
        insult: "Look at you, you neon-tailed disaster!",
        style: FireworkStyle::Ribbon {
            ribbons: 4,
            wave_count: 5.2,
        },
        radius_factor: 0.42,
        seed: 42,
    },
    FireworkEvent {
        start_time: 10.0,
        center: (0.78, 0.40),
        launch_x: 0.88,
        primary_color: Color::new(255, 110, 210),
        accent_color: Color::new(255, 245, 140),
        insult: "Dance, you cosmic trash panda!",
        style: FireworkStyle::Cascade {
            strands: 5,
            arch_height: 0.65,
        },
        radius_factor: 0.46,
        seed: 77,
    },
    FireworkEvent {
        start_time: 15.0,
        center: (0.30, 0.55),
        launch_x: 0.24,
        primary_color: Color::new(255, 200, 90),
        accent_color: Color::new(255, 90, 40),
        insult: "Explode already, you overcaffeinated firecracker!",
        style: FireworkStyle::Spiral {
            arms: 7,
            swirl_turns: 3.1,
        },
        radius_factor: 0.50,
        seed: 99,
    },
    FireworkEvent {
        start_time: 20.0,
        center: (0.60, 0.58),
        launch_x: 0.56,
        primary_color: Color::new(70, 255, 170),
        accent_color: Color::new(255, 180, 90),
        insult: "Twirl, you melodramatic thunder potato!",
        style: FireworkStyle::Ribbon {
            ribbons: 5,
            wave_count: 6.4,
        },
        radius_factor: 0.48,
        seed: 141,
    },
    FireworkEvent {
        start_time: 25.0,
        center: (0.88, 0.52),
        launch_x: 0.94,
        primary_color: Color::new(200, 160, 255),
        accent_color: Color::new(130, 220, 255),
        insult: "Glow on, you foul-mouthed disco pancake!",
        style: FireworkStyle::Cascade {
            strands: 6,
            arch_height: 0.75,
        },
        radius_factor: 0.44,
        seed: 314,
    },
];

fn paint_background(grid: &mut GridBuffer) {
    let width = grid.width();
    let height = grid.height();
    if width == 0 || height == 0 {
        return;
    }

    let top = Color::new(6, 8, 20);
    let bottom = Color::new(10, 14, 32);
    let denom = (height - 1).max(1) as f32;

    for y in 0..height {
        let t = y as f32 / denom;
        let color = blend_color(top, bottom, t);
        for x in 0..width {
            grid.set_cell_with_color(x, y, ' ', color);
        }
    }
}

fn transfer_braille(braille: &BrailleGrid, grid: &mut GridBuffer) {
    let width = grid.width();
    let height = grid.height();

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

fn draw_overlays(grid: &mut GridBuffer, overlays: &[TextOverlay]) {
    let width = grid.width() as isize;
    let height = grid.height() as isize;
    if width <= 0 || height <= 0 {
        return;
    }

    for overlay in overlays {
        if overlay.text.is_empty() {
            continue;
        }

        let text_len = overlay.text.chars().count() as isize;
        if text_len == 0 {
            continue;
        }

        let max_row = height - 2;
        let row = overlay.row.clamp(1, max_row).max(0) as usize;
        let max_start = (width - text_len).max(0);
        let start = (overlay.column - text_len / 2).clamp(0, max_start) as usize;
        let color = scale_color(overlay.color, 0.8 + 0.4 * overlay.intensity.clamp(0.0, 1.0));

        for (i, ch) in overlay.text.chars().enumerate() {
            if start + i >= grid.width() {
                break;
            }
            grid.set_cell_with_color(start + i, row, ch, color);
        }
    }
}

fn paint_dot(braille: &mut BrailleGrid, dims: &SceneDims, x: f32, y: f32, color: Color) {
    if x.is_nan() || y.is_nan() {
        return;
    }
    if x < 0.0 || y < 0.0 {
        return;
    }
    if x >= dims.dot_width as f32 || y >= dims.dot_height as f32 {
        return;
    }

    braille.set_dot_with_color(x.round() as usize, y.round() as usize, color);
}

fn draw_line(
    braille: &mut BrailleGrid,
    dims: &SceneDims,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    color: Color,
) {
    if dims.dot_width == 0 || dims.dot_height == 0 {
        return;
    }

    let max_x = dims.dot_width.saturating_sub(1) as f32;
    let max_y = dims.dot_height.saturating_sub(1) as f32;
    let x0 = clamp_to_range(x0, 0.0, max_x);
    let y0 = clamp_to_range(y0, 0.0, max_y);
    let x1 = clamp_to_range(x1, 0.0, max_x);
    let y1 = clamp_to_range(y1, 0.0, max_y);
    braille.draw_line_with_color(x0, y0, x1, y1, color);
}

fn clamp_to_range(value: f32, min: f32, max: f32) -> usize {
    if max <= min {
        min.max(0.0).round() as usize
    } else {
        value.clamp(min, max).round() as usize
    }
}

fn scale_color(color: Color, factor: f32) -> Color {
    Color::new(
        (color.r as f32 * factor).clamp(0.0, 255.0) as u8,
        (color.g as f32 * factor).clamp(0.0, 255.0) as u8,
        (color.b as f32 * factor).clamp(0.0, 255.0) as u8,
    )
}

fn blend_color(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::new(
        (a.r as f32 + (b.r as f32 - a.r as f32) * t).clamp(0.0, 255.0) as u8,
        (a.g as f32 + (b.g as f32 - a.g as f32) * t).clamp(0.0, 255.0) as u8,
        (a.b as f32 + (b.b as f32 - a.b as f32) * t).clamp(0.0, 255.0) as u8,
    )
}

fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn ease_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

fn ease_in_out_sine(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    0.5 - 0.5 * (PI * t).cos()
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut renderer = TerminalRenderer::new()?;
    let (width_u16, height_u16) = renderer.dimensions();
    let width = width_u16 as usize;
    let height = height_u16 as usize;

    let mut grid = GridBuffer::new(width, height);
    let mut braille = BrailleGrid::new(width, height);
    let dims = SceneDims::new(width, height);

    println!("Starting insult fireworks... Press 'q' or ESC to exit.");
    thread::sleep(Duration::from_millis(400));

    let start = Instant::now();
    let mut last_frame = Instant::now();

    loop {
        if event::poll(Duration::from_millis(1))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    _ => {}
                }
            }
        }

        let elapsed = start.elapsed().as_secs_f32();
        let show_time = elapsed.rem_euclid(LOOP_DURATION);

        braille.clear();
        let mut overlays = Vec::with_capacity(6);

        for event in FIREWORK_EVENTS.iter() {
            event.render(show_time, &dims, &mut braille, &mut overlays);
        }

        grid.clear();
        paint_background(&mut grid);
        transfer_braille(&braille, &mut grid);
        draw_overlays(&mut grid, &overlays);

        renderer.render(&grid)?;

        let frame_elapsed = last_frame.elapsed();
        if frame_elapsed < FRAME_TIME {
            thread::sleep(FRAME_TIME - frame_elapsed);
        }
        last_frame = Instant::now();
    }

    renderer.cleanup()?;
    Ok(())
}
