//! Model Viewer visualizer - displays glTF models from the catalog with audio reactivity

use super::{lerp, BrailleGrid, Color, GridBuffer, Visualizer};
use crate::dsp::AudioParameters;
use crate::visualization::ray_tracer::{
    model_catalog, model_downloader, render_with_orientation, Camera, RenderMode, Scene,
    WireframeRotation,
};
use std::time::Instant;

pub struct ModelViewerVisualizer {
    scene: Option<Scene>,
    camera: Camera,
    mode: RenderMode,
    render_scale: f32,
    // Current model info
    current_model_index: usize,
    model_name: String,
    // Audio-reactive light intensity (smoothed)
    light_intensity: f32,
    smoothing: f32,
    // Rotation state
    rotation_y: f32,
    rotation_speed_y: f32, // radians per second
    auto_rotate: bool,
    last_time: Instant,
}

impl ModelViewerVisualizer {
    pub fn new() -> Self {
        Self::new_with_model_index(0)
    }

    pub fn new_with_model_index(model_index: usize) -> Self {
        let camera = Camera::new(
            crate::visualization::ray_tracer::math::Vector3::new(0.0, 0.0, 5.0),
            4.0,
            3.0,
        );

        let mut viz = Self {
            scene: None,
            camera,
            mode: RenderMode::Wireframe {
                step_rad: crate::visualization::ray_tracer::DEFAULT_WIREFRAME_STEP_RAD,
                tol_rad: crate::visualization::ray_tracer::DEFAULT_WIREFRAME_TOL_RAD,
            },
            render_scale: 0.6, // render at 60% resolution for performance on complex meshes
            current_model_index: model_index,
            model_name: String::from("Loading..."),
            light_intensity: 0.8,
            smoothing: 0.15,
            rotation_y: 0.0,
            rotation_speed_y: 0.6,
            auto_rotate: true,
            last_time: Instant::now(),
        };

        // Load the initial model
        viz.load_model(model_index);
        viz
    }

    /// Load a model by index from the catalog
    fn load_model(&mut self, index: usize) {
        let catalog = model_catalog::get_catalog();

        // Wrap index to catalog size
        let safe_index = if catalog.is_empty() {
            0
        } else {
            index % catalog.len()
        };

        if let Some(model_info) = catalog.get(safe_index) {
            self.current_model_index = safe_index;
            self.model_name = model_info.name.to_string();

            // Determine filename from URL to avoid mismatches and case issues
            let filename = model_downloader::filename_from_url(model_info.url);
            let model_path = if model_downloader::is_model_cached(&filename) {
                model_downloader::get_cached_model_path(&filename).unwrap()
            } else {
                // Download the model (+ resources if .gltf)
                match model_downloader::download_model_with_progress(model_info.url, &filename, model_info.name) {
                    Ok(path) => path,
                    Err(e) => {
                        tracing::error!("Failed to download model {}: {}", model_info.name, e);
                        self.model_name = format!("{} (Download Failed)", model_info.name);
                        return;
                    }
                }
            };

            // Load the scene
            match Scene::new_with_model(model_path.to_str().unwrap()) {
                Ok(scene) => {
                    self.scene = Some(scene);
                    tracing::info!("Loaded model: {}", model_info.name);
                }
                Err(e) => {
                    tracing::error!("Failed to load model {}: {}", model_info.name, e);
                    self.model_name = format!("{} (Load Failed)", model_info.name);
                    self.scene = None;
                }
            }
        }
    }

    /// Switch to the next model in the catalog
    pub fn next_model(&mut self) {
        let catalog = model_catalog::get_catalog();
        if !catalog.is_empty() {
            let next_index = (self.current_model_index + 1) % catalog.len();
            self.load_model(next_index);
        }
    }

    /// Switch to the previous model in the catalog
    pub fn prev_model(&mut self) {
        let catalog = model_catalog::get_catalog();
        if !catalog.is_empty() {
            let prev_index = if self.current_model_index == 0 {
                catalog.len() - 1
            } else {
                self.current_model_index - 1
            };
            self.load_model(prev_index);
        }
    }

    /// Get the current model name
    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    /// Set rotation speed
    pub fn set_rotation_speed_y(&mut self, speed: f32) {
        self.rotation_speed_y = speed.max(0.0);
    }

    /// Enable/disable auto-rotation
    pub fn set_auto_rotate(&mut self, enable: bool) {
        self.auto_rotate = enable;
    }

    /// Toggle render mode between wireframe and solid
    pub fn toggle_render_mode(&mut self) {
        self.mode = match self.mode {
            RenderMode::Solid => RenderMode::Wireframe {
                step_rad: crate::visualization::ray_tracer::DEFAULT_WIREFRAME_STEP_RAD,
                tol_rad: crate::visualization::ray_tracer::DEFAULT_WIREFRAME_TOL_RAD,
            },
            RenderMode::Wireframe { .. } => RenderMode::Solid,
        };
    }
    /// Get current wireframe parameters if in wireframe mode
    pub fn wire_params(&self) -> Option<(f32, f32)> {
        match self.mode {
            RenderMode::Wireframe { step_rad, tol_rad } => Some((step_rad, tol_rad)),
            _ => None,
        }
    }

    /// Set wireframe step (in radians)
    pub fn set_wire_step_rad(&mut self, step_rad: f32) {
        let step = step_rad.max(2.0_f32.to_radians()).min(45.0_f32.to_radians());
        match self.mode {
            RenderMode::Wireframe { tol_rad, .. } => {
                self.mode = RenderMode::Wireframe { step_rad: step, tol_rad };
            }
            RenderMode::Solid => {
                // Switch to wireframe with provided step
                self.mode = RenderMode::Wireframe { step_rad: step, tol_rad: crate::visualization::ray_tracer::DEFAULT_WIREFRAME_TOL_RAD };
            }
        }
    }

    /// Set wireframe tolerance (thickness) in radians
    pub fn set_wire_tol_rad(&mut self, tol_rad: f32) {
        let tol = tol_rad.max(0.002).min(0.15);
        match self.mode {
            RenderMode::Wireframe { step_rad, .. } => {
                self.mode = RenderMode::Wireframe { step_rad, tol_rad: tol };
            }
            RenderMode::Solid => {
                // Switch to wireframe with provided tolerance
                self.mode = RenderMode::Wireframe { step_rad: crate::visualization::ray_tracer::DEFAULT_WIREFRAME_STEP_RAD, tol_rad: tol };
            }
        }
    }

    #[inline]
    fn update_lights(&mut self, intensity: f32) {
        if let Some(ref mut scene) = self.scene {
            for l in &mut scene.lights {
                l.intensity = intensity;
            }
        }
    }
}

impl Visualizer for ModelViewerVisualizer {
    fn name(&self) -> &str {
        "Model Viewer"
    }

    fn update(&mut self, params: &AudioParameters) {
        // Map overall amplitude to light intensity in [0.3, 1.0]
        let target = 0.3 + params.amplitude.clamp(0.0, 1.0) * 0.7;
        self.light_intensity = lerp(self.light_intensity, target, self.smoothing);
        self.update_lights(self.light_intensity);

        // Advance rotation using wall-clock time (if auto-rotate)
        let now = Instant::now();
        let dt = now.duration_since(self.last_time).as_secs_f32();
        self.last_time = now;
        if self.auto_rotate {
            self.rotation_y =
                (self.rotation_y + self.rotation_speed_y * dt) % (std::f32::consts::PI * 2.0);
        }
    }

    fn render(&self, grid: &mut GridBuffer) {
        // If no scene loaded, show error message
        let Some(ref scene) = self.scene else {
            // Clear grid and show error
            for y in 0..grid.height() {
                for x in 0..grid.width() {
                    grid.set_cell(x, y, ' ');
                }
            }
            return;
        };

        // Render into a high-res buffer (2x4 per cell for Braille)
        let full_w = grid.width() * 2;
        let full_h = grid.height() * 4;
        let s = self.render_scale.clamp(0.2, 1.0);
        let w = ((full_w as f32) * s).max(1.0) as usize;
        let h = ((full_h as f32) * s).max(1.0) as usize;
        let buffer = render_with_orientation(
            scene,
            &self.camera,
            w,
            h,
            self.mode,
            WireframeRotation {
                yaw: self.rotation_y,
                pitch: 0.0,
                roll: 0.0,
            },
        );

        // Convert to Braille and colorize GREEN like the Raycaster3D channel
        let mut braille = BrailleGrid::new(grid.width(), grid.height());
        let mut cell_max: Vec<f32> = vec![0.0; grid.width() * grid.height()];

        for py in 0..h {
            for px in 0..w {
                let v = buffer[py][px].clamp(0.0, 1.0);
                if v <= 0.05 { continue; }
                // Map reduced-resolution sample back to full Braille grid space
                let sx = if w > 1 { (((px as f32) * ((full_w - 1) as f32) / ((w - 1) as f32)).round() as usize).min(full_w - 1) } else { 0 };
                let sy = if h > 1 { (((py as f32) * ((full_h - 1) as f32) / ((h - 1) as f32)).round() as usize).min(full_h - 1) } else { 0 };
                let cx = sx / 2;
                let cy = sy / 4;
                let idx = cy * grid.width() + cx;
                braille.set_dot(sx, sy);
                if v > cell_max[idx] { cell_max[idx] = v; }
            }
        }

        // Write to GridBuffer using a green ramp for consistency
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                let ch = braille.get_char(x, y);
                if ch == ' ' {
                    grid.set_cell(x, y, ' ');
                } else {
                    let v = cell_max[y * grid.width() + x];
                    let g = (32.0 + (v * 223.0)) as u8; // 32..255
                    let color = Color::new(0, g, 0);
                    grid.set_cell_with_color(x, y, ch, color);
                }
            }
        }
    }
}

