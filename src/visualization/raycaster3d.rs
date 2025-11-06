//! Raycaster 3D visualizer integrating the ray_tracer demo into the app channel

use super::{BrailleGrid, Color, GridBuffer, Visualizer, lerp};
use crate::dsp::AudioParameters;
use crate::visualization::ray_tracer::{render_with_orientation, WireframeRotation, Camera, RenderMode, Scene};
use std::time::Instant;

pub struct Raycaster3DVisualizer {
    scene: Scene,
    camera: Camera,
    mode: RenderMode,
    // User-controlled brightness boost (added to audio-reactive mapping)
    brightness_boost: f32,
    // Audio-reactive light intensity (smoothed)
    light_intensity: f32,
    smoothing: f32,
    // Rotation state
    rotation_y: f32,
    rotation_speed_y: f32, // radians per second
    last_time: Instant,
}

impl Raycaster3DVisualizer {
    pub fn new() -> Self {
        Self::new_with(
            RenderMode::Wireframe {
                step_rad: crate::visualization::ray_tracer::DEFAULT_WIREFRAME_STEP_RAD,
                tol_rad: crate::visualization::ray_tracer::DEFAULT_WIREFRAME_TOL_RAD,
            },
            0.0,
        )
    }

    pub fn new_with(mode: RenderMode, brightness_boost: f32) -> Self {
        let scene = Scene::new_with_sphere_and_light();
        let camera = Camera::new(
            crate::visualization::ray_tracer::math::Vector3::new(0.0, 0.0, 0.0),
            4.0,
            3.0,
        );
        Self {
            scene,
            camera,
            mode,
            brightness_boost,
            light_intensity: 0.6,
            smoothing: 0.15,
            rotation_y: 0.0,
            rotation_speed_y: 0.6,
            last_time: Instant::now(),
        }
    }

    #[inline]
    fn update_lights(&mut self, intensity: f32) {
        // Scene exposes lights as pub(crate), accessible inside crate
        for l in &mut self.scene.lights {
            l.intensity = intensity;
        }
    }
}

impl Visualizer for Raycaster3DVisualizer {
    fn update(&mut self, params: &AudioParameters) {
        // Map overall amplitude to light intensity in [0.3, 1.0], then apply user boost
        let base = 0.3 + params.amplitude.clamp(0.0, 1.0) * 0.7;
        let target = (base + self.brightness_boost).clamp(0.0, 1.0);
        self.light_intensity = lerp(self.light_intensity, target, self.smoothing);
        self.update_lights(self.light_intensity);

        // Advance rotation using wall-clock time
        let now = Instant::now();
        let dt = now.duration_since(self.last_time).as_secs_f32();
        self.last_time = now;
        self.rotation_y = (self.rotation_y + self.rotation_speed_y * dt) % (std::f32::consts::PI * 2.0);
    }

    fn render(&self, grid: &mut GridBuffer) {
        // Render into a high-res buffer (2x4 per cell for Braille)
        let w = grid.width() * 2;
        let h = grid.height() * 4;
        let buffer = render_with_orientation(
            &self.scene,
            &self.camera,
            w,
            h,
            self.mode,
            WireframeRotation { yaw: self.rotation_y, pitch: 0.0, roll: 0.0 },
        );

        // Convert to BrailleGrid and colorize green by per-cell max intensity
        let mut braille = BrailleGrid::new(grid.width(), grid.height());
        let mut cell_max: Vec<f32> = vec![0.0; grid.width() * grid.height()];

        for py in 0..h {
            for px in 0..w {
                let v = buffer[py][px].clamp(0.0, 1.0);
                if v <= 0.05 { continue; }
                let cx = px / 2;
                let cy = py / 4;
                let idx = cy * grid.width() + cx;
                // Set the corresponding braille dot at absolute dot coords
                braille.set_dot(px, py);
                if v > cell_max[idx] {
                    cell_max[idx] = v;
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
                    // Map intensity to a green ramp for visibility
                    let v = cell_max[y * grid.width() + x];
                    let g = (32.0 + (v * 223.0)) as u8; // 32..255
                    let color = Color::new(0, g, 0);
                    grid.set_cell_with_color(x, y, ch, color);
                }
            }
        }
    }

    fn name(&self) -> &str { "Raycaster 3D" }
}

