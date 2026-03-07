//! Primitives 3D visualizer
//!
//! A "demoscene" style 3D visualization featuring a central pulsing sphere
//! and orbiting satellites, rendered in retro wireframe style.

use super::{lerp, BrailleGrid, Color, GridBuffer, Visualizer};
use crate::dsp::AudioParameters;
use crate::visualization::color_schemes::ColorScheme;
use crate::visualization::ray_tracer::{
    math::Vector3, render_with_orientation, Camera, Light, RenderMode, Scene, Sphere,
    WireframeRotation,
};
use crate::visualization::Transform3DControls;
use std::time::Instant;

pub struct PrimitivesVisualizer {
    scene: Scene,
    camera: Camera,
    mode: RenderMode,

    // Animation state
    base_radius: f32,
    pulse_radius: f32,
    orbit_angle: f32,
    orbit_speed: f32,
    color_scheme: ColorScheme,
    controls: Transform3DControls,

    // Audio reactivity
    light_intensity: f32,
    flash_energy: f32,
    audio_pitch: f32,
    audio_roll: f32,
    smoothing: f32,

    last_time: Instant,
}

impl PrimitivesVisualizer {
    pub fn new(color_scheme: ColorScheme) -> Self {
        let mut scene = Scene::new();
        // Initial setup will be overwritten in first update, but good to have valid state
        scene.add_light(Light::new(Vector3::new(-2.0, 2.0, 0.0), 1.0));

        let camera = Camera::new(
            Vector3::new(0.0, 0.0, 0.0), // Camera at origin
            4.0,                         // Width
            3.0,                         // Height
        );

        Self {
            scene,
            camera,
            // Use Wireframe mode for retro style as requested
            mode: RenderMode::Wireframe {
                step_rad: 0.15, // Slightly coarser for retro feel
                tol_rad: 0.02,
            },
            base_radius: 1.0,
            pulse_radius: 1.0,
            orbit_angle: 0.0,
            orbit_speed: 1.0,
            color_scheme,
            controls: Transform3DControls::default(),
            light_intensity: 0.8,
            flash_energy: 0.0,
            audio_pitch: 0.0,
            audio_roll: 0.0,
            smoothing: 0.15,
            last_time: Instant::now(),
        }
    }

    pub fn set_color_scheme(&mut self, color_scheme: ColorScheme) {
        self.color_scheme = color_scheme;
    }

    pub fn toggle_auto_rotate(&mut self) -> bool {
        self.controls.auto_rotate = !self.controls.auto_rotate;
        self.controls.auto_rotate
    }

    pub fn yaw_left(&mut self, step: f32) {
        self.controls.yaw_left(step);
    }

    pub fn yaw_right(&mut self, step: f32) {
        self.controls.yaw_right(step);
    }

    pub fn pitch_up(&mut self, step: f32) {
        self.controls.pitch_up(step);
    }

    pub fn pitch_down(&mut self, step: f32) {
        self.controls.pitch_down(step);
    }

    pub fn roll_ccw(&mut self, step: f32) {
        self.controls.roll_ccw(step);
    }

    pub fn roll_cw(&mut self, step: f32) {
        self.controls.roll_cw(step);
    }

    pub fn zoom_in(&mut self) {
        self.controls.zoom_in();
    }

    pub fn zoom_out(&mut self) {
        self.controls.zoom_out();
    }

    fn color_for_intensity(&self, value: f32) -> Color {
        let boosted = (value * (0.9 + self.flash_energy * 0.35)).clamp(0.0, 1.0);
        if let Some(color) = self.color_scheme.get_color(boosted) {
            color
        } else {
            let brightness = (60.0 + boosted * 195.0) as u8;
            Color::new(0, brightness, (brightness as f32 * 0.18) as u8)
        }
    }

    fn update_scene(&mut self) {
        // Rebuild scene objects for animation
        self.scene.objects.clear();
        self.scene.lights.clear();

        // Central Core (Pulsing)
        // Positioned at z = -4.0 to be visible to camera at origin looking -Z
        let center_pos = Vector3::new(0.0, 0.0, -4.0);
        self.scene
            .add_object(Box::new(Sphere::new(center_pos, self.pulse_radius)));

        // Orbitals
        let orbit_radius = 2.5;
        let num_orbitals = 3;

        for i in 0..num_orbitals {
            let angle_offset = (i as f32) * (2.0 * std::f32::consts::PI / num_orbitals as f32);
            let current_angle = self.orbit_angle + angle_offset;

            // Orbit in X-Z plane around the center object
            let x = center_pos.x + orbit_radius * current_angle.cos();
            let z = center_pos.z + orbit_radius * current_angle.sin();
            // Add some bobbing in Y
            let y = center_pos.y + 0.5 * (current_angle * 2.0).sin();

            self.scene.add_object(Box::new(Sphere::new(
                Vector3::new(x, y, z),
                0.4, // Smaller radius for orbitals
            )));
        }

        // Dynamic Light
        // Position light to rotate with orbitals but slightly offset
        let light_x = center_pos.x + 3.0 * (self.orbit_angle + 1.0).cos();
        let light_z = center_pos.z + 3.0 * (self.orbit_angle + 1.0).sin();
        self.scene.add_light(Light::new(
            Vector3::new(light_x, 2.0, light_z),
            self.light_intensity,
        ));
    }
}

impl Visualizer for PrimitivesVisualizer {
    fn update(&mut self, params: &AudioParameters) {
        // Time delta
        let now = Instant::now();
        let dt = now
            .duration_since(self.last_time)
            .as_secs_f32()
            .clamp(0.001, 0.1);
        self.last_time = now;

        // 1. Bass -> Central Core Pulse
        // Map bass (0.0-1.0) to radius modulation
        let target_radius = self.base_radius + (params.bass * 0.8);
        self.pulse_radius = lerp(self.pulse_radius, target_radius, self.smoothing);

        // 2. Mid -> Orbit Rotation Speed
        // Base speed + burst from mids
        let current_speed = self.orbit_speed + (params.mid * 3.0);
        self.orbit_angle = (self.orbit_angle + current_speed * dt) % (2.0 * std::f32::consts::PI);

        // 3. Treble -> Light Intensity / Brightness
        let target_intensity = 0.5 + (params.treble * 1.5);
        self.light_intensity = lerp(self.light_intensity, target_intensity, self.smoothing);

        // 4. Audio-reactive orientation polish
        self.controls.yaw_speed = 0.35 + params.mid * 1.4;
        self.controls.update(dt);
        self.audio_pitch = lerp(self.audio_pitch, (params.treble - 0.5) * 0.75, 0.08);
        self.audio_roll = lerp(self.audio_roll, (params.bass - params.treble) * 0.45, 0.06);

        let flash_target = if params.beat_bass { 1.0 } else { 0.0 };
        self.flash_energy = lerp(
            self.flash_energy,
            flash_target,
            if params.beat_bass { 0.45 } else { 0.08 },
        );

        // Rebuild scene with new properties
        self.update_scene();
    }

    fn render(&self, grid: &mut GridBuffer) {
        // Render into a high-res buffer (2x4 per cell for Braille)
        let w = grid.width() * 2;
        let h = grid.height() * 4;

        // No global rotation for the whole scene, we rotate objects internally
        let camera = Camera {
            viewport_width: self.camera.viewport_width / self.controls.scale,
            viewport_height: self.camera.viewport_height / self.controls.scale,
            ..self.camera
        };

        let buffer = render_with_orientation(
            &self.scene,
            &camera,
            w,
            h,
            self.mode,
            WireframeRotation {
                yaw: self.controls.yaw,
                pitch: self.controls.pitch + self.audio_pitch,
                roll: self.controls.roll + self.audio_roll,
            },
        );

        // Convert to BrailleGrid
        let mut braille = BrailleGrid::new(grid.width(), grid.height());
        let mut cell_intensity: Vec<f32> = vec![0.0; grid.width() * grid.height()];

        for py in 0..h {
            for px in 0..w {
                let v = buffer[py][px].clamp(0.0, 1.0);
                if v <= 0.05 {
                    continue;
                }

                // Set braille dot
                braille.set_dot(px, py);

                // Track max intensity for the cell to determine color
                let cx = px / 2;
                let cy = py / 4;
                let idx = cy * grid.width() + cx;
                if v > cell_intensity[idx] {
                    cell_intensity[idx] = v;
                }
            }
        }

        // Write to GridBuffer
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                let ch = braille.get_char(x, y);
                if ch == ' ' {
                    grid.set_cell(x, y, ' ');
                } else {
                    // Retro green phosphor look
                    let v = cell_intensity[y * grid.width() + x];
                    let color = self.color_for_intensity(v);
                    grid.set_cell_with_color(x, y, ch, color);
                }
            }
        }
    }

    fn name(&self) -> &str {
        "Primitives 3D"
    }
}

#[cfg(test)]
mod tests {
    use super::PrimitivesVisualizer;
    use crate::visualization::color_schemes::{ColorScheme, ColorSchemeType};

    #[test]
    fn monochrome_primitives_keep_phosphor_bias() {
        let viz = PrimitivesVisualizer::new(ColorScheme::new(ColorSchemeType::Monochrome));
        let color = viz.color_for_intensity(0.5);
        assert!(color.g > color.r);
        assert!(color.g > color.b);
    }

    #[test]
    fn colored_primitives_use_active_scheme() {
        let viz = PrimitivesVisualizer::new(ColorScheme::new(ColorSchemeType::HeatMap));
        let color = viz.color_for_intensity(1.0);
        assert!(color.r >= color.g);
    }

    #[test]
    fn primitives_zoom_controls_adjust_scale() {
        let mut viz = PrimitivesVisualizer::new(ColorScheme::new(ColorSchemeType::Monochrome));
        let original = viz.controls.scale;
        viz.zoom_in();
        assert!(viz.controls.scale > original);
        viz.zoom_out();
        assert!(viz.controls.scale <= original * 1.01);
    }

    #[test]
    fn primitives_auto_rotate_toggle_flips_state() {
        let mut viz = PrimitivesVisualizer::new(ColorScheme::new(ColorSchemeType::Monochrome));
        assert!(viz.controls.auto_rotate);
        assert!(!viz.toggle_auto_rotate());
        assert!(viz.toggle_auto_rotate());
    }
}
