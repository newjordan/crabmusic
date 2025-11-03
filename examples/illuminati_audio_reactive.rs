//! Audio-Reactive Illuminati Braille Animation
//!
//! This example demonstrates an audio-reactive version of the Illuminati eye animation.
//! It captures audio from the default input device, processes it to extract amplitude,
//! and uses this data to modulate the animation's parameters, creating a dynamic,
//! pulsating effect that syncs with the music.

use anyhow::{Context, Result};
use crabmusic::audio::{AudioCaptureDevice, AudioRingBuffer, CpalAudioDevice};
use crabmusic::dsp::{AudioParameters, DspProcessor};
use crabmusic::rendering::TerminalRenderer;
use crabmusic::visualization::{BrailleGrid, Color, GridBuffer};
use crossterm::event::{self, Event, KeyCode};
use std::f32::consts::{PI, TAU};
use std::sync::Arc;
use std::time::{Duration, Instant};

const FRAME_DURATION: Duration = Duration::from_millis(50); // Faster refresh rate for audio reactivity

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- Audio Setup ---
    let ring_buffer = Arc::new(AudioRingBuffer::new(10));
        let mut audio_device = CpalAudioDevice::new_with_device(ring_buffer.clone(), None)
        .context("Failed to initialize audio device")?;
    let sample_rate = audio_device.get_config().sample_rate;
    let mut dsp_processor = DspProcessor::new(sample_rate, 2048)
        .context("Failed to initialize DSP processor")?;
    audio_device.start_capture()?;

    // --- Renderer and Animation Setup ---
    let mut renderer = TerminalRenderer::new()?;
    let (width, height) = renderer.dimensions();
    let mut grid = GridBuffer::new(width as usize, height as usize);
    let mut animation = IlluminatiAnimation::new();

    println!("Summoning audio-reactive illuminati... Press 'q' or ESC to exit.");
    std::thread::sleep(Duration::from_millis(520));

    let mut last_frame = Instant::now();

    // --- Main Loop ---
    loop {
        let frame_start = Instant::now();

        if event::poll(Duration::from_millis(1))? {
            if let Event::Key(key) = event::read()? {
                if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                    break;
                }
            }
        }

        let dt = frame_start.saturating_duration_since(last_frame).as_secs_f32().min(0.12);
        last_frame = frame_start;

        // --- Audio Processing ---
        let audio_params = if let Some(audio_buffer) = audio_device.read_samples() {
            dsp_processor.process(&audio_buffer)
        } else {
            AudioParameters::default()
        };

        // --- Update and Render ---
        animation.update(dt, &audio_params);
        grid.clear();
        animation.render(&mut grid);
        renderer.render(&grid)?;

        let elapsed = frame_start.elapsed();
        if elapsed < FRAME_DURATION {
            std::thread::sleep(FRAME_DURATION - elapsed);
        }
    }

    audio_device.stop_capture()?;
    renderer.cleanup()?;
    println!("\nVision fades. 👁️");

    Ok(())
}

struct IlluminatiAnimation {
    time: f32,
    bass_level: f32,
    mid_level: f32,
    treble_level: f32,
}

impl IlluminatiAnimation {
    fn new() -> Self {
        Self {
            time: 0.0,
            bass_level: 0.0,
            mid_level: 0.0,
            treble_level: 0.0,
        }
    }

    fn update(&mut self, dt: f32, audio_params: &AudioParameters) {
        self.time += dt;

        // Smoothly update audio levels to prevent jitter
        let smoothing_factor = 1.0 - (-dt / 0.1).exp(); // ~100ms smoothing
        self.bass_level = lerp(self.bass_level, audio_params.bass, smoothing_factor);
        self.mid_level = lerp(self.mid_level, audio_params.mid, smoothing_factor);
        self.treble_level = lerp(self.treble_level, audio_params.treble, smoothing_factor);
    }

    fn zoom_factor(&self) -> f32 {
        let slow = (self.time * 0.34).sin();
        let drift = (self.time * 0.085).sin();
        let combined = slow * 0.085 + drift * 0.02;
        let zoom = 1.0 + combined * 2.52;
        zoom.clamp(0.72, 1.32)
    }

    fn eye_open_amount(&self) -> f32 {
        let period = 8.6;
        let progress = (self.time / period).fract();
        let main_blink = if progress < 0.7 {
            1.0
        } else if progress < 0.82 {
            let t = (progress - 0.7) / 0.12;
            1.0 - smoothstep(t) * 0.88
        } else if progress < 0.88 {
            0.12
        } else {
            let t = (progress - 0.88) / 0.12;
            0.12 + smoothstep(t) * 0.88
        };

        // Treble can cause micro-blinks or flutters
        let audio_flutter = (self.treble_level * 1.5).min(0.3);
        (main_blink - audio_flutter).clamp(0.08, 1.0)
    }

    // --- All rendering functions below are mostly the same as the original, ---
    // --- but they will now use the audio-reactive parameters from above. ---

    fn render(&self, grid: &mut GridBuffer) {
        let width = grid.width();
        let height = grid.height();

        let mut braille = BrailleGrid::new(width, height);
        let geometry = self.triangle_geometry(&braille);
        let mut layers = vec![CellContribution::default(); width * height];

        self.draw_background(&mut braille, &mut layers, &geometry);
        self.draw_triangle(&mut braille, &mut layers, &geometry);
        self.draw_eye(&mut braille, &mut layers, &geometry);
        self.draw_inner_sigils(&mut braille, &mut layers, &geometry);

        for y in 0..height {
            for x in 0..width {
                let ch = braille.get_char(x, y);
                let color = self.compose_color(x, y, width, height, &layers[y * width + x]);
                grid.set_cell_with_color(x, y, ch, color);
            }
        }
    }

    fn draw_background(
        &self,
        braille: &mut BrailleGrid,
        layers: &mut [CellContribution],
        geometry: &TriangleGeometry,
    ) {
        let width = braille.width();
        let center = geometry.center;

        let mist_points = (width as f32 * 2.8) as usize;
        for i in 0..mist_points {
            let seed = i as f32 * 1.37;
            let phase = self.time * 0.22 + seed * 0.37;
            let radial = (phase.sin() * 0.5 + 0.5).powf(1.35);
            let radius =
                radial * geometry.scale * 1.18 + (self.time * 2.5 + seed).sin() * 3.5 + 6.0;
            let angle = seed * 0.91 + self.time * 0.18 + radial * 4.2;

            let pos = center.add(Vec2::from_polar(radius, angle));
            plot_layer(
                braille,
                layers,
                width,
                pos,
                Layer::Sigil,
                0.12 + (radial * 0.18),
            );
        }
    }

    fn draw_triangle(
        &self,
        braille: &mut BrailleGrid,
        layers: &mut [CellContribution],
        geometry: &TriangleGeometry,
    ) {
        let width = braille.width();

        let center = geometry.center;
        let top = geometry.top;
        let left = geometry.left;
        let right = geometry.right;
        let vertices = [top, right, left];

        for idx in 0..3 {
            let a = vertices[idx];
            let b = vertices[(idx + 1) % 3];
            let layer = if idx == 1 { Layer::Drip } else { Layer::Edge };
            self.draw_edge_strip(braille, layers, width, a, b, layer);
        }

        self.draw_dripping_edge(braille, layers, width, left, right, geometry);

        let inner = [
            center.add((top - center).mul(0.64)),
            center.add((right - center).mul(0.64)),
            center.add((left - center).mul(0.64)),
        ];

        for idx in 0..3 {
            let a = inner[idx];
            let b = inner[(idx + 1) % 3];
            self.draw_energy_line(braille, layers, width, a, b, 0.45);
        }

        for vertex in &vertices {
            self.draw_energy_line(braille, layers, width, center, *vertex, 0.36);
        }
    }

    fn draw_edge_strip(
        &self,
        braille: &mut BrailleGrid,
        layers: &mut [CellContribution],
        width: usize,
        start: Vec2,
        end: Vec2,
        layer: Layer,
    ) {
        let direction = end - start;
        let length = direction.length().max(1.0);
        let steps = (length / 1.2).ceil() as usize;
        let normal = direction.perp().normalized();

        for step in 0..=steps {
            let t = step as f32 / steps as f32;
            let pos = start.lerp(end, t);
            let wobble = (self.time * 0.6 + t * TAU).sin() * 0.8;
            let base_pos = pos.add(normal.mul(wobble));

            let thickness = [(-2.4, 0.28), (-1.3, 0.58), (0.0, 1.0), (1.2, 0.58), (2.3, 0.28)];
            for (offset, weight) in thickness {
                let offset_pos = base_pos.add(normal.mul(offset));
                plot_layer(braille, layers, width, offset_pos, layer, weight);
            }
        }
    }

    fn draw_energy_line(
        &self,
        braille: &mut BrailleGrid,
        layers: &mut [CellContribution],
        width: usize,
        start: Vec2,
        end: Vec2,
        strength: f32,
    ) {
        let direction = end - start;
        let length = direction.length().max(1.0);
        let steps = (length / 1.8).ceil() as usize;
        // Energy pulse now reacts to mid-range frequencies
        let pulse_speed = 0.9 + strength * 1.8 + self.mid_level * 2.5;

        for step in 0..=steps {
            let t = step as f32 / steps as f32;
            let pos = start.lerp(end, t);
            let pulse = ((self.time * pulse_speed + t * 5.1).sin() * 0.5 + 0.5) * strength;
            plot_layer(braille, layers, width, pos, Layer::Sigil, 0.24 + pulse * 0.8);
        }
    }

    fn draw_eye(
        &self,
        braille: &mut BrailleGrid,
        layers: &mut [CellContribution],
        geometry: &TriangleGeometry,
    ) {
        let width = braille.width();
        let dot_width = braille.dot_width() as f32;
        let dot_height = braille.dot_height() as f32;

        let center = geometry.center;
        let frame_radius_x = geometry.scale * 0.26;
        let frame_radius_y = frame_radius_x * 0.66;
        let frame_shadow_radius_x = frame_radius_x * 0.88;
        let frame_shadow_radius_y = frame_radius_y * 0.88;

        let eye_radius_x = frame_radius_x * 0.68;
        let eye_radius_y = frame_radius_y * 0.74;

        let open = self.eye_open_amount();
        let visible_height = eye_radius_y * (0.18 + 0.82 * open);

        let look = self.look_vector().mul(open);
        let pupil_center = center.add(Vec2::new(
            look.x * eye_radius_x * 0.55,
            look.y * eye_radius_y * 0.6,
        ));

        let iris_radius_x = eye_radius_x * 0.88;
        let iris_radius_y = eye_radius_y * 0.9;
        let pupil_radius_x = eye_radius_x * (0.32 + (1.0 - open) * 0.08 + self.bass_level * 0.05);
        let pupil_radius_y = eye_radius_y * (0.36 + (1.0 - open) * 0.12 + self.bass_level * 0.06);

        let dot_min_x = ((center.x - frame_radius_x - 6.0).max(0.0)).floor() as usize;
        let dot_max_x = ((center.x + frame_radius_x + 6.0).min(dot_width - 1.0)).ceil() as usize;
        let dot_min_y = ((center.y - frame_radius_y - 6.0).max(0.0)).floor() as usize;
        let dot_max_y = ((center.y + frame_radius_y + 6.0).min(dot_height - 1.0)).ceil() as usize;

        for dot_y in dot_min_y..=dot_max_y {
            for dot_x in dot_min_x..=dot_max_x {
                let px = dot_x as f32 + 0.35;
                let py = dot_y as f32 + 0.45;

                let dx = px - center.x;
                let dy = py - center.y;

                let frame_metric =
                    (dx * dx) / (frame_radius_x * frame_radius_x)
                        + (dy * dy) / (frame_radius_y * frame_radius_y);
                if frame_metric > 1.08 {
                    continue;
                }

                if frame_metric >= 0.86 {
                    let rim = (1.12 - frame_metric).max(0.0) * 3.0
                        + (frame_metric - 0.86).max(0.0) * 2.4;
                    plot_layer(
                        braille,
                        layers,
                        width,
                        Vec2::new(px, py),
                        Layer::Frame,
                        rim * (0.7 + open * 0.3),
                    );
                    continue;
                }

                let shadow_metric =
                    (dx * dx) / (frame_shadow_radius_x * frame_shadow_radius_x)
                        + (dy * dy) / (frame_shadow_radius_y * frame_shadow_radius_y);
                if shadow_metric >= 0.7 {
                    let depth = (0.86 - shadow_metric).max(0.0);
                    if depth > 0.0 {
                        plot_layer(
                            braille,
                            layers,
                            width,
                            Vec2::new(px, py),
                            Layer::FrameShadow,
                            depth * (0.6 + (1.0 - open) * 0.8),
                        );
                    }
                }

                let eye_metric =
                    (dx * dx) / (eye_radius_x * eye_radius_x)
                        + (dy * dy) / (eye_radius_y * eye_radius_y);
                if eye_metric > 1.05 {
                    continue;
                }

                if dy.abs() > visible_height {
                    let eyelid_amount =
                        ((dy.abs() - visible_height) / (eye_radius_y - visible_height))
                            .clamp(0.0, 1.0);
                    let sweep = 0.45 + dy.signum() * 0.08;
                    plot_layer(
                        braille,
                        layers,
                        width,
                        Vec2::new(px, py),
                        Layer::Eyelid,
                        (1.0 - eyelid_amount * eyelid_amount) * (1.1 - open) * sweep.abs(),
                    );
                    continue;
                }

                let dx_iris = px - pupil_center.x;
                let dy_iris = py - pupil_center.y;

                let iris_metric = (dx_iris * dx_iris) / (iris_radius_x * iris_radius_x)
                    + (dy_iris * dy_iris) / (iris_radius_y * iris_radius_y);
                let pupil_metric = (dx_iris * dx_iris) / (pupil_radius_x * pupil_radius_x)
                    + (dy_iris * dy_iris) / (pupil_radius_y * pupil_radius_y);

                if pupil_metric <= 1.04 {
                    let core = (1.04 - pupil_metric).max(0.08);
                    plot_layer(
                        braille,
                        layers,
                        width,
                        Vec2::new(px, py),
                        Layer::Pupil,
                        core * (0.8 + open * 0.2),
                    );
                } else if iris_metric <= 1.06 {
                    let depth = (1.06 - iris_metric).max(0.05);
                    plot_layer(
                        braille,
                        layers,
                        width,
                        Vec2::new(px, py),
                        Layer::Iris,
                        depth * (0.65 + open * 0.35),
                    );
                } else {
                    let whiteness = (1.02 - eye_metric).max(0.04);
                    plot_layer(
                        braille,
                        layers,
                        width,
                        Vec2::new(px, py),
                        Layer::Sclera,
                        whiteness,
                    );
                }
            }
        }

        if open > 0.22 {
            let highlight_center =
                pupil_center.add(Vec2::new(-pupil_radius_x * 0.6, -pupil_radius_y * 0.28));
            let highlight_offsets = [
                Vec2::new(0.0, 0.0),
                Vec2::new(0.9, 0.7),
                Vec2::new(-0.6, -0.5),
                Vec2::new(0.4, -1.1),
            ];
            for offset in &highlight_offsets {
                plot_layer(
                    braille,
                    layers,
                    width,
                    highlight_center.add(*offset),
                    Layer::Highlight,
                    1.15,
                );
            }
        }

        let eyelid_steps = 120;
        for step in 0..=eyelid_steps {
            let theta = step as f32 / eyelid_steps as f32 * PI;
            let cos = theta.cos();
            let sin = theta.sin();
            let top = Vec2::new(center.x + eye_radius_x * cos, center.y - visible_height * sin);
            let bottom = Vec2::new(center.x + eye_radius_x * cos, center.y + visible_height * sin);
            let weight_top = (1.0 - cos * cos).powf(0.28) * (0.55 + (1.0 - open) * 0.85);
            let weight_bottom = (1.0 - cos * cos).powf(0.3) * (0.4 + (1.0 - open) * 0.6);

            plot_layer(braille, layers, width, top, Layer::Eyelid, weight_top);
            plot_layer(braille, layers, width, bottom, Layer::Eyelid, weight_bottom);
        }

        add_glow_rings(
            braille,
            layers,
            width,
            center,
            frame_radius_x * 0.48,
            4,
            0.24 + open * 0.18,
        );
    }

    fn draw_dripping_edge(
        &self,
        braille: &mut BrailleGrid,
        layers: &mut [CellContribution],
        width: usize,
        left: Vec2,
        right: Vec2,
        geometry: &TriangleGeometry,
    ) {
        let edge_base = left.lerp(right, 0.5);
        let downward = Vec2::new(0.0, 1.0);
        let drip_zoom = geometry.zoom;

        let lip_pulse = (self.time * 0.42).sin() * 0.4;
        let lip_steps = 120;
        for step in 0..=lip_steps {
            let t = step as f32 / lip_steps as f32;
            let ribbon = left.lerp(right, t);
            let sway = (self.time * 0.55 + t * 8.4).sin() * 0.9;
            let underset = ribbon.add(Vec2::new(
                sway * 0.4 * drip_zoom,
                (1.6 + lip_pulse * 0.8) * drip_zoom,
            ));

            let lip_profile = [(-1.8, 0.7), (-0.6, 0.95), (0.6, 0.95), (1.8, 0.7)];
            for (offset, weight) in lip_profile {
                let pos = underset.add(Vec2::new(offset * drip_zoom, 0.0));
                plot_layer(braille, layers, width, pos, Layer::Drip, weight);
            }
        }

        let drip_count = 9;
        for i in 0..drip_count {
            let base_t = (i as f32 + 0.2 * (self.time * 0.32 + i as f32 * 1.91).sin())
                / (drip_count as f32 - 1.0);
            let base_t = base_t.clamp(0.05, 0.95);
            let anchor = left.lerp(right, base_t);

            let phase = self.time * 0.48 + i as f32 * 1.37;
            let growth = (phase.sin() * 0.5 + 0.5).powf(1.4) + self.bass_level * 0.4;
            let extension = (6.0 + growth * 16.0) * drip_zoom;

            let tail_steps = (extension / 1.3).ceil() as usize;
            for step in 0..=tail_steps {
                let f = step as f32 / tail_steps as f32;
                let taper = (1.0 - f).powf(0.65);
                let swing = (self.time * 0.9 + base_t * 12.6 + f * 3.8).sin() * (1.2 - f * 0.9);
                let pos = anchor
                    .add(Vec2::new(
                        swing * 0.8 * growth * drip_zoom,
                        (2.8 + f * extension) * drip_zoom,
                    ))
                    .add(downward.mul(f * 1.4 * drip_zoom));
                plot_layer(braille, layers, width, pos, Layer::Drip, 0.85 * taper);
            }

            let tip = anchor.add(Vec2::new(
                (self.time * 1.15 + base_t * 6.4).sin() * 0.9 * drip_zoom,
                extension + 3.0 * drip_zoom,
            ));

            let droplets = [
                Vec2::new(0.0, 0.0),
                Vec2::new(0.6, 0.8),
                Vec2::new(-0.4, 0.9),
                Vec2::new(0.2, 1.7),
            ];
            for (idx, offset) in droplets.iter().enumerate() {
                let weight = if idx == 0 { 1.4 } else { 0.8 };
                plot_layer(
                    braille,
                    layers,
                    width,
                    tip.add(Vec2::new(offset.x * drip_zoom, offset.y * drip_zoom)),
                    Layer::Drip,
                    weight * (0.6 + growth * 0.4),
                );
            }

            add_glow_rings(
                braille,
                layers,
                width,
                tip,
                (2.6 + growth * 1.8) * drip_zoom,
                3,
                0.18 + growth * 0.22,
            );
        }

        let aura_steps = 32;
        for step in 0..aura_steps {
            let t = step as f32 / aura_steps as f32;
            let span = left.lerp(right, t);
            let tail = edge_base.add(Vec2::new(
                (self.time * 0.23 + t * 5.4).sin() * 6.0 * drip_zoom,
                (8.0 + (self.time * 0.6 + t * PI).sin() * 4.0) * drip_zoom,
            ));
            self.draw_energy_line(braille, layers, width, span, tail, 0.18);
        }
    }

    fn draw_inner_sigils(
        &self,
        braille: &mut BrailleGrid,
        layers: &mut [CellContribution],
        geometry: &TriangleGeometry,
    ) {
        let width = braille.width();
        let center = geometry.center.add(Vec2::new(0.0, -geometry.scale * 0.06));
        let radius = geometry.scale * 0.32;

        let mut angle = self.time * 0.26;
        for _ in 0..3 {
            for m in 0..16 {
                let progress = m as f32 / 16.0;
                let r = radius * (0.45 + progress * 0.55);
                let pos = center.add(Vec2::from_polar(
                    r,
                    angle + progress * TAU + (self.time * 0.15),
                ));
                plot_layer(
                    braille,
                    layers,
                    width,
                    pos,
                    Layer::Sigil,
                    0.18 + progress * 0.22,
                );
            }
            angle += TAU / 3.0;
        }

        let orbiters = 5;
        for i in 0..orbiters {
            let p = i as f32 / orbiters as f32;
            let sweep = self.time * 0.4 + p * TAU;
            let orbit_radius = radius * (0.65 + 0.18 * (self.time * 0.9 + p * 13.0).sin());
            let pos = center.add(Vec2::from_polar(orbit_radius, sweep));
            plot_layer(braille, layers, width, pos, Layer::Glow, 0.42);
            plot_layer(braille, layers, width, pos, Layer::Sigil, 0.38);
        }
    }

    fn look_vector(&self) -> Vec2 {
        let x = (self.time * 0.22).sin() * 0.55 + (self.time * 0.079).sin() * 0.35;
        let y = (self.time * 0.18).cos() * 0.45 + (self.time * 0.051).sin() * 0.28;
        let mut vec = Vec2::new(x, y);
        let len = vec.length();
        if len > 1.0 {
            vec = vec.mul(1.0 / len);
        }
        vec
    }

    fn compose_color(
        &self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        mix: &CellContribution,
    ) -> Color {
        let mut color = self.base_color(x, y, width, height);

        // Make colors pop more with audio
        let bass_glow = (self.bass_level * 0.6).min(1.0);
        let treble_sparkle = (self.treble_level * 0.7).min(1.0);

        if mix.sigil > 0.0 {
            color = blend(color, Color::new(110, 70, 170), (mix.sigil * 0.7).min(1.0));
        }
        if mix.glow > 0.0 {
            color = add_color(color, Color::new(170, 200, 120), (mix.glow * 0.55 + bass_glow * 0.4).min(1.0));
        }
        if mix.edge > 0.0 {
            color = blend(color, Color::new(245, 220, 150), (mix.edge * 0.9).min(1.0));
        }
        if mix.sclera > 0.0 {
            color = blend(color, Color::new(235, 238, 240), mix.sclera.min(1.0));
        }
        if mix.iris > 0.0 {
            color = blend(color, Color::new(70, 220, 200), (mix.iris * 0.95).min(1.0));
        }
        if mix.pupil > 0.0 {
            color = blend(color, Color::new(20, 30, 45), (mix.pupil * 0.85).min(1.0));
        }
        if mix.eyelid > 0.0 {
            color = blend(color, Color::new(210, 150, 90), (mix.eyelid * 0.8).min(1.0));
        }
        if mix.highlight > 0.0 {
            color = blend(color, Color::new(255, 255, 255), (mix.highlight + treble_sparkle).min(1.0));
        }
        if mix.drip > 0.0 {
            color = add_color(color, Color::new(105, 230, 170), (mix.drip * 0.7).min(1.0));
        }
        if mix.frame > 0.0 {
            color = blend(color, Color::new(245, 215, 150), (mix.frame * 0.85).min(1.0));
        }
        if mix.frame_shadow > 0.0 {
            color = blend(color, Color::new(80, 55, 35), (mix.frame_shadow * 0.6).min(1.0));
        }

        color
    }

    fn triangle_geometry(&self, braille: &BrailleGrid) -> TriangleGeometry {
        let dot_width = braille.dot_width() as f32;
        let dot_height = braille.dot_height() as f32;
        let anchor = Vec2::new(dot_width * 0.5, dot_height * 0.54);
        let base_scale = dot_height.min(dot_width) * 0.68;
        let zoom = self.zoom_factor();
        let scale = base_scale * zoom;

        let top = Vec2::new(anchor.x, anchor.y - scale * 0.58);
        let left = Vec2::new(anchor.x - scale * 0.64, anchor.y + scale * 0.48);
        let right = Vec2::new(anchor.x + scale * 0.64, anchor.y + scale * 0.48);

        let centroid = Vec2::new(
            (top.x + left.x + right.x) / 3.0,
            (top.y + left.y + right.y) / 3.0,
        );

        TriangleGeometry {
            center: centroid,
            top,
            left,
            right,
            scale,
            zoom,
        }
    }

    fn base_color(&self, x: usize, y: usize, width: usize, height: usize) -> Color {
        let nx = x as f32 / width as f32 - 0.5;
        let ny = y as f32 / height as f32 - 0.5;
        let dist = (nx * nx + ny * ny).sqrt();

        let swirl = ((dist * 6.8) - self.time * 0.17).sin() * 0.5 + 0.5;
        let vertical = y as f32 / height as f32;
        let pulse = ((self.time * 0.23) + nx * 3.2).sin() * 0.5 + 0.5;

        let base_r = 12.0 + swirl * 18.0 + pulse * 10.0 + self.bass_level * 40.0;
        let base_g = 20.0 + swirl * 24.0 + (1.0 - vertical) * 22.0 + self.mid_level * 30.0;
        let base_b = 28.0 + swirl * 35.0 + vertical * 18.0 + (1.0 - dist).max(0.0) * 24.0 + self.treble_level * 25.0;

        Color::new(
            base_r.clamp(0.0, 255.0) as u8,
            base_g.clamp(0.0, 255.0) as u8,
            base_b.clamp(0.0, 255.0) as u8,
        )
    }
}

// --- Utility Structs and Functions (mostly unchanged) ---

struct TriangleGeometry {
    center: Vec2,
    top: Vec2,
    left: Vec2,
    right: Vec2,
    scale: f32,
    zoom: f32,
}

#[derive(Clone, Copy, Default)]
struct CellContribution {
    glow: f32,
    edge: f32,
    sclera: f32,
    iris: f32,
    pupil: f32,
    highlight: f32,
    eyelid: f32,
    sigil: f32,
    drip: f32,
    frame: f32,
    frame_shadow: f32,
}

impl CellContribution {
    fn add(&mut self, layer: Layer, weight: f32) {
        let w = weight.max(0.0);
        match layer {
            Layer::Glow => self.glow += w,
            Layer::Edge => self.edge += w,
            Layer::Sclera => self.sclera += w,
            Layer::Iris => self.iris += w,
            Layer::Pupil => self.pupil += w,
            Layer::Highlight => self.highlight += w,
            Layer::Eyelid => self.eyelid += w,
            Layer::Sigil => self.sigil += w,
            Layer::Drip => self.drip += w,
            Layer::Frame => self.frame += w,
            Layer::FrameShadow => self.frame_shadow += w,
        }
    }
}

#[derive(Clone, Copy)]
enum Layer {
    Glow,
    Edge,
    Sclera,
    Iris,
    Pupil,
    Highlight,
    Eyelid,
    Sigil,
    Drip,
    Frame,
    FrameShadow,
}

#[derive(Clone, Copy, Default)]
struct Vec2 {
    x: f32,
    y: f32,
}

impl Vec2 {
    fn new(x: f32, y: f32) -> Self { Self { x, y } }
    fn add(self, other: Vec2) -> Self { Self { x: self.x + other.x, y: self.y + other.y } }
    fn sub(self, other: Vec2) -> Self { Self { x: self.x - other.x, y: self.y - other.y } }
    fn mul(self, scalar: f32) -> Self { Self { x: self.x * scalar, y: self.y * scalar } }
    fn length(self) -> f32 { (self.x * self.x + self.y * self.y).sqrt() }
    fn normalized(self) -> Self {
        let len = self.length();
        if len < 1e-5 { Self { x: 0.0, y: 0.0 } } else { self.mul(1.0 / len) }
    }
    fn perp(self) -> Self { Self { x: -self.y, y: self.x } }
    fn lerp(self, other: Vec2, t: f32) -> Self { self.add((other - self).mul(t)) }
    fn from_polar(radius: f32, angle: f32) -> Self { Self { x: radius * angle.cos(), y: radius * angle.sin() } }
}

impl std::ops::Add for Vec2 { type Output = Vec2; fn add(self, rhs: Vec2) -> Vec2 { self.add(rhs) } }
impl std::ops::Sub for Vec2 { type Output = Vec2; fn sub(self, rhs: Vec2) -> Vec2 { self.sub(rhs) } }

fn plot_layer(
    braille: &mut BrailleGrid,
    mixes: &mut [CellContribution],
    width: usize,
    pos: Vec2,
    layer: Layer,
    weight: f32,
) {
    let dot_x = pos.x.round() as isize;
    let dot_y = pos.y.round() as isize;
    if dot_x < 0 || dot_y < 0 { return; }
    let (dot_x, dot_y) = (dot_x as usize, dot_y as usize);

    if dot_x >= braille.dot_width() || dot_y >= braille.dot_height() { return; }
    braille.set_dot(dot_x, dot_y);

    let (cell_x, cell_y) = (dot_x / 2, dot_y / 4);
    let idx = cell_y * width + cell_x;
    if idx < mixes.len() {
        mixes[idx].add(layer, weight);
    }
}

fn add_glow_rings(
    braille: &mut BrailleGrid,
    mixes: &mut [CellContribution],
    width: usize,
    center: Vec2,
    base_radius: f32,
    rings: usize,
    weight: f32,
) {
    for ring in 1..=rings {
        let radius = base_radius * ring as f32;
        let samples = 12 + ring * 6;
        let falloff = weight / ring as f32;
        for sample in 0..samples {
            let angle = sample as f32 / samples as f32 * TAU;
            let pos = center.add(Vec2::from_polar(radius, angle));
            plot_layer(braille, mixes, width, pos, Layer::Glow, falloff);
        }
    }
}

fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn blend(base: Color, overlay: Color, weight: f32) -> Color {
    let w = weight.clamp(0.0, 1.0);
    Color::new(
        lerp_channel(base.r, overlay.r, w),
        lerp_channel(base.g, overlay.g, w),
        lerp_channel(base.b, overlay.b, w),
    )
}

fn add_color(base: Color, addition: Color, weight: f32) -> Color {
    let w = weight.clamp(0.0, 1.0);
    let r = base.r as f32 + addition.r as f32 * w;
    let g = base.g as f32 + addition.g as f32 * w;
    let b = base.b as f32 + addition.b as f32 * w;
    Color::new(r.clamp(0.0, 255.0) as u8, g.clamp(0.0, 255.0) as u8, b.clamp(0.0, 255.0) as u8)
}

fn lerp_channel(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).clamp(0.0, 255.0).round() as u8
}