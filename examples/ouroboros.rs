//! Ouroboros snake rendered with animated Braille dots.
//!
//! The `Head` owns the single source of truth, evolving parameters through a
//! logistic map. The `Neck` reads those parameters every frame and spins a fresh
//! body geometry. The `Throat` constricts that body, measures its harmony, and
//! deletes it while feeding balance reports back into the head. The visible
//! result is a looping snake that continuously renews itself in Braille.

use crabmusic::rendering::TerminalRenderer;
use crabmusic::visualization::{BrailleGrid, Color, GridBuffer};
use crossterm::event::{self, Event, KeyCode};
use std::f32::consts::TAU;
use std::thread;
use std::time::{Duration, Instant};

const FRAME_TIME: Duration = Duration::from_millis(96);
const GOLDEN_RATIO: f32 = 1.618_033_9;
const SEGMENT_COUNT: usize = 220;

#[derive(Clone, Copy)]
struct Vec2 {
    x: f32,
    y: f32,
}

impl Vec2 {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    fn from_polar(radius: f32, angle: f32) -> Self {
        Self {
            x: radius * angle.cos(),
            y: radius * angle.sin(),
        }
    }

    fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    fn normalized(self) -> Self {
        let len = self.length();
        if len <= 1e-5 {
            Self::zero()
        } else {
            Self {
                x: self.x / len,
                y: self.y / len,
            }
        }
    }

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    fn mul(self, scalar: f32) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }

    fn cross(self, other: Self) -> f32 {
        self.x * other.y - self.y * other.x
    }
}

struct SceneDims {
    width: usize,
    height: usize,
    dot_width: usize,
    dot_height: usize,
    min_dot: usize,
}

impl SceneDims {
    fn new(width: usize, height: usize) -> Self {
        let safe_width = width.max(32);
        let safe_height = height.max(16);
        let dot_width = safe_width * 2;
        let dot_height = safe_height * 4;
        let min_dot = dot_width.min(dot_height);
        Self {
            width: safe_width,
            height: safe_height,
            dot_width,
            dot_height,
            min_dot,
        }
    }

    fn scale_for(&self, radius: f32) -> f32 {
        if radius <= 1e-3 {
            return self.min_dot as f32 * 0.35;
        }
        self.min_dot as f32 * 0.45 / radius
    }

    fn center_dot(&self) -> Vec2 {
        Vec2::new(self.dot_width as f32 / 2.0, self.dot_height as f32 / 2.0)
    }
}

#[derive(Clone, Copy)]
struct Segment {
    pos: Vec2,
    thickness: f32,
    chroma: f32,
}

struct Body {
    segments: Vec<Segment>,
    mean_radius: f32,
    curvature: f32,
    max_radius: f32,
}

struct Truth {
    phase: f32,
    wave_amplitude: f32,
    twist: f32,
    pulse: f32,
    energy_target: f32,
}

struct Head {
    seed: f32,
    logistic_r: f32,
    energy_target: f32,
    phase: f32,
    wave_amplitude: f32,
    twist_phase: f32,
    pulse_phase: f32,
    last_time: f32,
}

impl Head {
    fn new(seed: f32, logistic_r: f32, energy_target: f32) -> Self {
        let clamped_seed = seed.clamp(0.0001, 0.9999);
        Self {
            seed: clamped_seed,
            logistic_r,
            energy_target,
            phase: 0.0,
            wave_amplitude: 0.18,
            twist_phase: 0.0,
            pulse_phase: 0.0,
            last_time: 0.0,
        }
    }

    fn emit_truth(&mut self, time: f32) -> Truth {
        let dt = (time - self.last_time).clamp(0.0, 0.25);
        self.last_time = time;

        let next = self.logistic_r * self.seed * (1.0 - self.seed);
        self.seed = next.clamp(0.0001, 0.9999);
        let chaotic = self.seed;

        let phase_speed = 0.45 + 0.25 * (chaotic - 0.5);
        self.phase = (self.phase + dt * phase_speed).rem_euclid(TAU);

        let target_amp = 0.18 + 0.12 * (chaotic - 0.5);
        self.wave_amplitude = (self.wave_amplitude * 0.9 + target_amp * 0.1).clamp(0.05, 0.4);

        let twist_speed = 1.2 + 0.8 * (chaotic - 0.5);
        self.twist_phase = (self.twist_phase + dt * twist_speed).rem_euclid(TAU);

        let pulse_speed = 1.8 + 1.1 * (chaotic - 0.5);
        self.pulse_phase = (self.pulse_phase + dt * pulse_speed).rem_euclid(TAU);

        Truth {
            phase: self.phase,
            wave_amplitude: self.wave_amplitude,
            twist: self.twist_phase,
            pulse: self.pulse_phase.sin(),
            energy_target: self.energy_target,
        }
    }

    fn harmonize(&mut self, report: &ContractionReport) {
        let mixed = 0.72 * self.energy_target + 0.28 * report.released;
        self.energy_target = mixed.clamp(0.65, 1.35);

        let adjustment = (report.swirl * 0.17 + report.deviation * 0.11).clamp(-0.25, 0.25);
        self.seed = (self.seed + adjustment)
            .rem_euclid(1.0)
            .clamp(0.0001, 0.9999);
    }
}

struct Neck;

impl Neck {
    fn weave_body(&self, truth: &Truth, time: f32) -> Body {
        let mut segments = Vec::with_capacity(SEGMENT_COUNT);
        let mut radius_acc = 0.0;
        let mut max_radius: f32 = 0.0;

        for i in 0..SEGMENT_COUNT {
            let t = i as f32 / SEGMENT_COUNT as f32;
            let angle = truth.phase + t * TAU;
            let ripple = (angle * 1.6 + truth.wave_amplitude * 6.0 + time * 0.8).sin();
            let twist = (angle * 4.2 + truth.twist).cos();
            let spiral = (angle * GOLDEN_RATIO + time * 0.5).sin();

            let radius = truth.energy_target
                * (1.0 + truth.wave_amplitude * ripple + 0.06 * twist + 0.03 * spiral);
            let thickness = 0.16 + 0.1 * (0.5 + 0.5 * ripple) + 0.06 * (1.0 - t).powf(1.3);
            let chroma = 0.5 + 0.5 * (twist * 0.6 + truth.pulse * 0.4 + (1.0 - t).powf(1.2));

            segments.push(Segment {
                pos: Vec2::from_polar(radius, angle),
                thickness,
                chroma,
            });

            radius_acc += radius;
            max_radius = max_radius.max(radius);
        }

        let mut curvature_acc = 0.0;
        for window in segments.windows(3) {
            let a = window[0].pos;
            let b = window[1].pos;
            let c = window[2].pos;
            curvature_acc += b.sub(a).cross(c.sub(b));
        }

        let mean_radius = radius_acc / SEGMENT_COUNT as f32;
        let curvature = curvature_acc / SEGMENT_COUNT as f32;

        Body {
            segments,
            mean_radius,
            curvature,
            max_radius,
        }
    }
}

struct Throat;

struct ContractionReport {
    released: f32,
    deviation: f32,
    swirl: f32,
}

impl Throat {
    fn constrict(&self, body: Body, target_energy: f32) -> ContractionReport {
        let released = body.mean_radius;
        let deviation = target_energy - released;
        let swirl = body.curvature * 0.012;
        ContractionReport {
            released,
            deviation,
            swirl,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut renderer = TerminalRenderer::new()?;
    let (width_u16, height_u16) = renderer.dimensions();
    let width = width_u16 as usize;
    let height = height_u16 as usize;

    let dims = SceneDims::new(width, height);
    let mut grid = GridBuffer::new(dims.width, dims.height);
    let mut braille = BrailleGrid::new(dims.width, dims.height);

    let mut head = Head::new(0.347, 3.92, 1.0);
    let neck = Neck;
    let throat = Throat;

    println!("Ouroboros snake — press 'q' or ESC to exit.");
    thread::sleep(Duration::from_millis(360));

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

        let raw_time = start.elapsed().as_secs_f32();
        let time = raw_time / 3.0;
        let truth = head.emit_truth(time);
        let body = neck.weave_body(&truth, time);

        braille.clear();
        render_body(&mut braille, &dims, &truth, &body, time);
        render_head(&mut braille, &dims, &body, time);

        grid.clear();
        transfer_braille(&braille, &mut grid);
        renderer.render(&grid)?;

        let report = throat.constrict(body, truth.energy_target);
        head.harmonize(&report);

        let elapsed = last_frame.elapsed();
        if elapsed < FRAME_TIME {
            thread::sleep(FRAME_TIME - elapsed);
        }
        last_frame = Instant::now();
    }

    renderer.cleanup()?;
    Ok(())
}

fn render_body(braille: &mut BrailleGrid, dims: &SceneDims, truth: &Truth, body: &Body, time: f32) {
    let scale = dims.scale_for(body.max_radius);
    let len = body.segments.len();
    if len < 3 {
        return;
    }

    let base = Color::new(32, 160, 118);
    let shade = Color::new(8, 52, 38);
    let highlight = Color::new(168, 240, 186);
    let constrict_band = (truth.pulse.abs() * 0.45 + 0.35) * len as f32;
    const GRID_OFFSETS: [f32; 3] = [-1.0, 0.0, 1.0];

    for (index, segment) in body.segments.iter().enumerate() {
        let prev = body.segments[(index + len - 1) % len].pos;
        let next = body.segments[(index + 1) % len].pos;
        let tangent = next.sub(prev).normalized();
        let normal = Vec2::new(-tangent.y, tangent.x);

        let fade = (1.0 - index as f32 / len as f32).powf(1.35);
        let constrict = ((constrict_band - index as f32).abs() / len as f32).clamp(0.0, 1.0);
        let lattice_wave = (time * 1.1 + index as f32 * GOLDEN_RATIO * 0.35).sin();
        let flow = (segment.chroma * 0.6 + truth.pulse * 0.25 + fade * 0.25 + lattice_wave * 0.15)
            .clamp(0.0, 1.0);

        let base_color = blend_color(shade, highlight, 0.25 + 0.45 * flow);
        let constrict_color = blend_color(base_color, Color::new(18, 26, 36), constrict * 0.9);
        let lattice_color = blend_color(base_color, highlight, 0.45 + 0.2 * flow);
        let node_glow = blend_color(lattice_color, highlight, 0.5);
        let fill_color = blend_color(constrict_color, base, 0.5 + 0.3 * fade);

        let core_radius = (segment.thickness
            * (1.2 + 0.25 * truth.wave_amplitude + 0.1 * truth.pulse))
            .clamp(0.16, 0.8);
        stamp_world_disc(
            braille,
            dims,
            segment.pos,
            core_radius * 1.35,
            scale,
            fill_color,
        );

        let spacing_u = segment.thickness * (0.85 + 0.25 * fade);
        let spacing_v = segment.thickness * (0.78 + 0.18 * fade);
        let line_radius = (segment.thickness * 0.12).clamp(0.04, 0.3);

        for (ui, &u) in GRID_OFFSETS.iter().enumerate() {
            for (vi, &v) in GRID_OFFSETS.iter().enumerate() {
                let offset_u = tangent.mul(u * spacing_u);
                let offset_v = normal.mul(v * spacing_v);
                let node_center = segment.pos.add(offset_u.add(offset_v));
                let weight = (1.0 - (u.abs() + v.abs()) * 0.22).clamp(0.0, 1.0);
                let node_radius = (core_radius * (0.38 + 0.32 * weight)).clamp(0.06, 0.55);
                let color_mix = blend_color(lattice_color, node_glow, 0.6 * weight + 0.2);
                stamp_world_disc(braille, dims, node_center, node_radius, scale, color_mix);

                if ui == 0 {
                    let start = node_center.sub(tangent.mul(spacing_u * 0.6));
                    let end = node_center.add(tangent.mul(spacing_u * 0.6));
                    draw_world_line(
                        braille,
                        dims,
                        start,
                        end,
                        line_radius * (0.6 + 0.2 * weight),
                        scale,
                        blend_color(lattice_color, highlight, 0.3 * weight + 0.1),
                    );
                }
                if vi == 0 {
                    let start = node_center.sub(normal.mul(spacing_v * 0.6));
                    let end = node_center.add(normal.mul(spacing_v * 0.6));
                    draw_world_line(
                        braille,
                        dims,
                        start,
                        end,
                        line_radius * (0.6 + 0.2 * weight),
                        scale,
                        blend_color(shade, lattice_color, 0.4 + 0.3 * weight),
                    );
                }
            }
        }

        for offset in [-1.0_f32, 0.0, 1.0] {
            let shift_normal = normal.mul(offset * spacing_v);
            let start_u = segment
                .pos
                .sub(tangent.mul(spacing_u * 1.4))
                .add(shift_normal);
            let end_u = segment
                .pos
                .add(tangent.mul(spacing_u * 1.4))
                .add(shift_normal);
            draw_world_line(
                braille,
                dims,
                start_u,
                end_u,
                line_radius * 0.75,
                scale,
                blend_color(lattice_color, highlight, 0.25 + 0.25 * fade),
            );

            let shift_tangent = tangent.mul(offset * spacing_u);
            let start_v = segment
                .pos
                .sub(normal.mul(spacing_v * 1.4))
                .add(shift_tangent);
            let end_v = segment
                .pos
                .add(normal.mul(spacing_v * 1.4))
                .add(shift_tangent);
            draw_world_line(
                braille,
                dims,
                start_v,
                end_v,
                line_radius * 0.75,
                scale,
                blend_color(shade, lattice_color, 0.45 + 0.25 * fade),
            );
        }
    }
}

fn render_head(braille: &mut BrailleGrid, dims: &SceneDims, body: &Body, time: f32) {
    if body.segments.len() < 4 {
        return;
    }

    let scale = dims.scale_for(body.max_radius);
    let head_seg = &body.segments[0];
    let next = &body.segments[1];
    let prev = &body.segments[body.segments.len() - 1];

    let forward = next.pos.sub(head_seg.pos).normalized();
    let backward = head_seg.pos.sub(prev.pos).normalized();
    let dir = forward.add(backward).normalized();
    let side = Vec2::new(-dir.y, dir.x);

    let jaw_color = Color::new(214, 248, 156);
    let head_color = Color::new(84, 214, 150);
    let eye_color = Color::new(255, 96, 64);
    let head_radius = head_seg.thickness * 1.35;
    let skull = head_seg.pos.add(dir.mul(head_seg.thickness * 0.35));
    stamp_world_disc(braille, dims, skull, head_radius, scale, head_color);

    let snout = head_seg.pos.add(dir.mul(head_seg.thickness * 1.6));
    stamp_world_disc(
        braille,
        dims,
        snout,
        head_seg.thickness * 1.05,
        scale,
        jaw_color,
    );

    let left_jaw = head_seg
        .pos
        .add(side.mul(head_seg.thickness * 0.9))
        .add(dir.mul(head_seg.thickness * 0.3));
    let right_jaw = head_seg
        .pos
        .sub(side.mul(head_seg.thickness * 0.9))
        .add(dir.mul(head_seg.thickness * 0.3));

    stamp_world_disc(
        braille,
        dims,
        left_jaw,
        head_seg.thickness * 0.65,
        scale,
        jaw_color,
    );
    stamp_world_disc(
        braille,
        dims,
        right_jaw,
        head_seg.thickness * 0.65,
        scale,
        jaw_color,
    );

    let eye_blink = (time * 2.6).sin().powi(2);
    let eye_offset = head_seg.thickness * (0.65 + 0.2 * eye_blink);
    let eye_radius = head_seg.thickness * (0.28 + 0.05 * eye_blink);

    let left_eye = skull
        .add(side.mul(eye_offset))
        .add(dir.mul(head_seg.thickness * 0.3));
    let right_eye = skull
        .sub(side.mul(eye_offset))
        .add(dir.mul(head_seg.thickness * 0.3));

    stamp_world_disc(braille, dims, left_eye, eye_radius, scale, eye_color);
    stamp_world_disc(braille, dims, right_eye, eye_radius, scale, eye_color);

    let pupil_color = Color::new(12, 8, 6);
    stamp_world_disc(
        braille,
        dims,
        left_eye.add(dir.mul(head_seg.thickness * 0.15)),
        eye_radius * 0.45,
        scale,
        pupil_color,
    );
    stamp_world_disc(
        braille,
        dims,
        right_eye.add(dir.mul(head_seg.thickness * 0.15)),
        eye_radius * 0.45,
        scale,
        pupil_color,
    );
}

fn transfer_braille(braille: &BrailleGrid, grid: &mut GridBuffer) {
    let width = grid.width();
    let height = grid.height();
    for y in 0..height {
        for x in 0..width {
            let ch = braille.get_char(x, y);
            if ch == '⠀' {
                continue;
            }
            if let Some(color) = braille.get_color(x, y) {
                grid.set_cell_with_color(x, y, ch, color);
            } else {
                grid.set_cell(x, y, ch);
            }
        }
    }
}

fn stamp_world_disc(
    braille: &mut BrailleGrid,
    dims: &SceneDims,
    world_pos: Vec2,
    world_radius: f32,
    scale: f32,
    color: Color,
) {
    let center = world_to_dot(dims, world_pos, scale);
    let radius = (world_radius * scale).max(0.8);
    stamp_disc_dots(braille, center, radius, color);
}

fn draw_world_line(
    braille: &mut BrailleGrid,
    dims: &SceneDims,
    start: Vec2,
    end: Vec2,
    radius: f32,
    scale: f32,
    color: Color,
) {
    let steps = (start.distance(end) * scale * 1.5).ceil() as usize;
    let steps = steps.max(3);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let point = Vec2 {
            x: start.x + (end.x - start.x) * t,
            y: start.y + (end.y - start.y) * t,
        };
        stamp_world_disc(braille, dims, point, radius, scale, color);
    }
}

fn world_to_dot(dims: &SceneDims, world: Vec2, scale: f32) -> Vec2 {
    let center = dims.center_dot();
    Vec2::new(center.x + world.x * scale, center.y - world.y * scale)
}

fn stamp_disc_dots(braille: &mut BrailleGrid, center: Vec2, radius: f32, color: Color) {
    let radius = radius.max(0.5);
    let dot_width = braille.dot_width() as isize;
    let dot_height = braille.dot_height() as isize;

    let min_x = (center.x - radius - 1.0).floor() as isize;
    let max_x = (center.x + radius + 1.0).ceil() as isize;
    let min_y = (center.y - radius - 1.0).floor() as isize;
    let max_y = (center.y + radius + 1.0).ceil() as isize;

    for y in min_y.max(0)..=max_y.min(dot_height - 1) {
        for x in min_x.max(0)..=max_x.min(dot_width - 1) {
            let dx = x as f32 + 0.5 - center.x;
            let dy = y as f32 + 0.5 - center.y;
            if dx * dx + dy * dy <= radius * radius {
                braille.set_dot_with_color(x as usize, y as usize, color);
            }
        }
    }
}

fn blend_color(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::new(
        (a.r as f32 + (b.r as f32 - a.r as f32) * t).clamp(0.0, 255.0) as u8,
        (a.g as f32 + (b.g as f32 - a.g as f32) * t).clamp(0.0, 255.0) as u8,
        (a.b as f32 + (b.b as f32 - a.b as f32) * t).clamp(0.0, 255.0) as u8,
    )
}

trait Distance {
    fn distance(self, other: Self) -> f32;
}

impl Distance for Vec2 {
    fn distance(self, other: Self) -> f32 {
        self.sub(other).length()
    }
}
