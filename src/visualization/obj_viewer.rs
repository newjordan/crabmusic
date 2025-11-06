//! OBJ Viewer visualizer - simple local model import (no network)

use super::{lerp, BrailleGrid, Color, GridBuffer, Visualizer};
use crate::dsp::AudioParameters;
use crate::visualization::ray_tracer::{render_with_orientation, render_edges_with_orientation, Camera, RenderMode, Scene, WireframeRotation};
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

pub struct ObjViewerVisualizer {
    scene: Option<Scene>,
    camera: Camera,
    mode: RenderMode,
    render_scale: f32,
    // Files in ./models with .obj extension
    files: Vec<PathBuf>,
    current_index: usize,
    display_name: String,
    // Audio-reactive and rotation state
    light_intensity: f32,
    smoothing: f32,
    rotation_y: f32,
    rotation_speed_y: f32, // rad/s
    auto_rotate: bool,
    last_time: Instant,
    // Edge/vertex render params (OBJ-only wireframe)
    edge_line_px: i32,
    vertex_dot_px: i32,
    // Model scale (zoom)
    model_scale: f32,
}

impl ObjViewerVisualizer {
    pub fn new() -> Self { Self::new_with_model_index(0) }

    pub fn new_with_model_index(index: usize) -> Self {
        let camera = Camera::new(
            crate::visualization::ray_tracer::math::Vector3::new(0.0, 0.0, 5.0),
            4.0,
            3.0,
        );
        let mut viz = Self {
            scene: None,
            camera,
            mode: RenderMode::Wireframe { step_rad: crate::visualization::ray_tracer::DEFAULT_WIREFRAME_STEP_RAD, tol_rad: crate::visualization::ray_tracer::DEFAULT_WIREFRAME_TOL_RAD },
            render_scale: 0.8,
            files: Vec::new(),
            current_index: 0,
            display_name: String::from("No OBJ found"),
            light_intensity: 0.8,
            smoothing: 0.15,
            rotation_y: 0.0,
            rotation_speed_y: 0.6,
            auto_rotate: true,
            last_time: Instant::now(),
            edge_line_px: 1,
            vertex_dot_px: 2,
            model_scale: 1.0,
        };
        viz.refresh_file_list();
        viz.load_model(index);
        viz
    }

    fn refresh_file_list(&mut self) {
        self.files = scan_obj_files();
        if self.files.is_empty() {
            self.display_name = "No OBJ found".into();
            self.scene = None;
        }
    }

    fn load_model(&mut self, index: usize) {
        if self.files.is_empty() { self.refresh_file_list(); }
        if self.files.is_empty() { return; }
        let idx = index % self.files.len();
        self.current_index = idx;
        let path = &self.files[idx];
        self.display_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("(invalid)").to_string();
        match Scene::new_with_obj_model(path.to_str().unwrap()) {
            Ok(scene) => { self.scene = Some(scene); }
            Err(e) => {
                self.scene = None;
                self.display_name = format!("{} (Load Failed)", self.display_name);
                tracing::error!("OBJ Viewer failed to load {}: {}", path.display(), e);
            }
        }
    }

    pub fn next_model(&mut self) {
        if !self.files.is_empty() {
            let next = (self.current_index + 1) % self.files.len();
            self.load_model(next);
        }
    }

    pub fn prev_model(&mut self) {
        if !self.files.is_empty() {
            let prev = if self.current_index == 0 { self.files.len() - 1 } else { self.current_index - 1 };
            self.load_model(prev);
        }
    }

    pub fn model_name(&self) -> &str { &self.display_name }

    pub fn set_auto_rotate(&mut self, enable: bool) { self.auto_rotate = enable; }

    pub fn toggle_render_mode(&mut self) {
        self.mode = match self.mode {
            RenderMode::Solid => RenderMode::Wireframe { step_rad: crate::visualization::ray_tracer::DEFAULT_WIREFRAME_STEP_RAD, tol_rad: crate::visualization::ray_tracer::DEFAULT_WIREFRAME_TOL_RAD },
            RenderMode::Wireframe { .. } => RenderMode::Solid,
        };
    }

    pub fn wire_params(&self) -> Option<(f32, f32)> {
        // Reuse fields to show edge/vertex sizes for status: map px -> pseudo radians
        match self.mode {
            RenderMode::Wireframe { .. } => {
                let step_rad = (self.edge_line_px as f32).to_radians();
                let tol_rad = (self.vertex_dot_px as f32).to_radians();
                Some((step_rad, tol_rad))
            }
            _ => None,
        }
    }
    pub fn set_wire_step_rad(&mut self, step_rad: f32) {
        // Map requested step to line thickness in pixels (1..6)
        let deg = step_rad.to_degrees().clamp(2.0, 60.0);
        let px = ((deg / 12.0).round() as i32).clamp(1, 6);
        self.edge_line_px = px;
        if let RenderMode::Solid = self.mode {
            self.mode = RenderMode::Wireframe { step_rad, tol_rad: crate::visualization::ray_tracer::DEFAULT_WIREFRAME_TOL_RAD };
        }
    }
    pub fn set_wire_tol_rad(&mut self, tol_rad: f32) {
        // Map requested tol to vertex dot size in pixels (1..6)
        let deg = tol_rad.to_degrees().clamp(1.0, 45.0);
        let px = ((deg / 9.0).round() as i32).clamp(1, 6);
        self.vertex_dot_px = px;
        if let RenderMode::Solid = self.mode {
            self.mode = RenderMode::Wireframe { step_rad: crate::visualization::ray_tracer::DEFAULT_WIREFRAME_STEP_RAD, tol_rad };
        }
    }
    pub fn wire_px(&self) -> Option<(i32, i32)> {
        match self.mode { RenderMode::Wireframe { .. } => Some((self.edge_line_px, self.vertex_dot_px)), _ => None }
    }

    pub fn zoom_in(&mut self) { self.model_scale = (self.model_scale * 1.2).min(20.0); }
    pub fn zoom_out(&mut self) { self.model_scale = (self.model_scale / 1.2).max(0.05); }

    pub fn focus_fit(&mut self) {
        let Some(scene) = &self.scene else { return; };
        let Some(verts) = scene.mesh_vertices() else { return; };
        // compute extents at scale=1 with current yaw/pitch=0 for stability
        use crate::visualization::ray_tracer::math::Vector3;
        use crate::visualization::ray_tracer::wireframe::rotate_normal_yaw_pitch as rot;
        let half_w = self.camera.viewport_width * 0.5;
        let half_h = self.camera.viewport_height * 0.5;
        let mut max_x = 1e-6f32;
        let mut max_y = 1e-6f32;
        for &p in verts.iter() {
            let pr = rot(p, self.rotation_y, 0.0);
            let q = Vector3::new(pr.x - self.camera.origin.x, pr.y - self.camera.origin.y, pr.z - self.camera.origin.z);
            if q.z >= -1e-3 { continue; }
            let t = -self.camera.focal_length / q.z;
            let x_plane = q.x * t;
            let y_plane = q.y * t;
            max_x = max_x.max(x_plane.abs());
            max_y = max_y.max(y_plane.abs());
        }
        if max_x > 0.0 && max_y > 0.0 {
            let sx = (half_w * 0.9) / max_x;
            let sy = (half_h * 0.9) / max_y;
            let target = sx.min(sy);
            self.model_scale = target.clamp(0.05, 20.0);
        }
    }

}


impl Visualizer for ObjViewerVisualizer {
    fn name(&self) -> &str { "OBJ Viewer" }

    fn update(&mut self, params: &AudioParameters) {
        // Smooth light
        let target = 0.3 + params.amplitude.clamp(0.0, 1.0) * 0.7;
        self.light_intensity = lerp(self.light_intensity, target, self.smoothing);
        if let Some(ref mut scene) = self.scene {
            for l in &mut scene.lights { l.intensity = self.light_intensity; }
        }

        // Time-based rotation
        let now = Instant::now();
        let dt = now.duration_since(self.last_time).as_secs_f32();
        self.last_time = now;
        if self.auto_rotate { self.rotation_y = (self.rotation_y + self.rotation_speed_y * dt) % (std::f32::consts::PI * 2.0); }
    }

    fn render(&self, grid: &mut GridBuffer) {
        let Some(ref scene) = self.scene else {
            // empty
            for y in 0..grid.height() { for x in 0..grid.width() { grid.set_cell(x, y, ' '); } }
            return;
        };

        let full_w = grid.width() * 2;
        let full_h = grid.height() * 4;
        let s = self.render_scale.clamp(0.3, 1.0);
        let w = ((full_w as f32) * s).max(1.0) as usize;
        let h = ((full_h as f32) * s).max(1.0) as usize;
        let buffer = match self.mode {
            RenderMode::Wireframe { .. } => {
                render_edges_with_orientation(
                    scene,
                    &self.camera,
                    w,
                    h,
                    self.rotation_y,
                    0.0,
                    self.model_scale,
                    self.vertex_dot_px,
                    self.edge_line_px,
                )
            }
            RenderMode::Solid => {
                render_with_orientation(
                    scene,
                    &self.camera,
                    w,
                    h,
                    self.mode,
                    WireframeRotation { yaw: self.rotation_y, pitch: 0.0, roll: 0.0 },
                )
            }
        };

        let mut braille = BrailleGrid::new(grid.width(), grid.height());
        let mut cell_max: Vec<f32> = vec![0.0; grid.width() * grid.height()];
        for py in 0..h {
            for px in 0..w {
                let v = buffer[py][px].clamp(0.0, 1.0);
                if v <= 0.05 { continue; }
                let sx = if w > 1 { (((px as f32) * ((full_w - 1) as f32) / ((w - 1) as f32)).round() as usize).min(full_w - 1) } else { 0 };
                let sy = if h > 1 { (((py as f32) * ((full_h - 1) as f32) / ((h - 1) as f32)).round() as usize).min(full_h - 1) } else { 0 };
                let cx = sx / 2; let cy = sy / 4; let idx = cy * grid.width() + cx;
                braille.set_dot(sx, sy);
                if v > cell_max[idx] { cell_max[idx] = v; }
            }
        }
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                let ch = braille.get_char(x, y);
                if ch == ' ' { grid.set_cell(x, y, ' '); } else {
                    let v = cell_max[y * grid.width() + x];
                    let g = (32.0 + (v * 223.0)) as u8;
                    grid.set_cell_with_color(x, y, ch, Color::new(0, g, 0));
                }
            }
        }
    }
}

fn scan_obj_files() -> Vec<PathBuf> {
    let mut out = Vec::new();
    let candidates = ["models", "./models", "assets/models", "./assets/models"]; // simple scan
    for dir in candidates {
        if let Ok(entries) = fs::read_dir(dir) {
            for e in entries.flatten() {
                let p = e.path();
                if p.is_file() {
                    if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
                        if ext.eq_ignore_ascii_case("obj") { out.push(p); }
                    }
                }
            }
        }
    }
    out.sort();
    out
}

