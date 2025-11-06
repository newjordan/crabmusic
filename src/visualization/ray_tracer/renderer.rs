//! Rendering pipeline producing a 2D intensity buffer (VIZ-010)

use super::camera::Camera;
use super::lighting::calculate_diffuse_shading;
use super::scene::Scene;
use super::wireframe::{
    is_on_wireframe_normal,
    is_on_wireframe_normal_rotated,
    DEFAULT_WIREFRAME_STEP_RAD,
    DEFAULT_WIREFRAME_TOL_RAD,
};
use super::RenderMode;

#[derive(Debug, Clone, Copy)]
pub struct WireframeRotation {
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32, // reserved for future use
}
impl Default for WireframeRotation {
    fn default() -> Self { Self { yaw: 0.0, pitch: 0.0, roll: 0.0 } }
}

pub fn render_with_orientation(
    scene: &Scene,
    camera: &Camera,
    width: usize,
    height: usize,
    mode: RenderMode,
    orientation: WireframeRotation,
) -> Vec<Vec<f32>> {
    let mut buffer = vec![vec![0.0_f32; width]; height];

    for y in 0..height {
        for x in 0..width {
            let u = if width > 1 { x as f32 / (width as f32 - 1.0) } else { 0.0 };
            let v = if height > 1 { y as f32 / (height as f32 - 1.0) } else { 0.0 };
            let ray = camera.get_ray(u, v);

            let intensity = if let Some(hit) = scene.hit(&ray, 0.001, f32::MAX) {
                match mode {
                    RenderMode::Wireframe { step_rad, tol_rad } => {
                        // Apply optional orientation to the normal before grid test
                        if is_on_wireframe_normal_rotated(
                            hit.normal,
                            step_rad,
                            tol_rad,
                            orientation.yaw,
                            orientation.pitch,
                        ) {
                            1.0
                        } else {
                            0.0
                        }
                    }
                    RenderMode::Solid => {
                        let mut sum = 0.0;
                        for light in scene.lights() { sum += calculate_diffuse_shading(hit.point, hit.normal, light); }
                        sum.clamp(0.0, 1.0)
                    }
                }
            } else {
                0.0
            };

            buffer[y][x] = intensity;
        }
    }

    buffer
}

pub fn render(scene: &Scene, camera: &Camera, width: usize, height: usize, mode: RenderMode) -> Vec<Vec<f32>> {
    render_with_orientation(scene, camera, width, height, mode, WireframeRotation::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visualization::ray_tracer::math::Vector3;

    #[test]
    fn test_render_dimensions() {
        let scene = Scene::new_with_sphere();
        let cam = Camera::new(Vector3::new(0.0, 0.0, 0.0), 4.0, 3.0);
        let buffer = render(&scene, &cam, 40, 30, RenderMode::Wireframe { step_rad: DEFAULT_WIREFRAME_STEP_RAD, tol_rad: DEFAULT_WIREFRAME_TOL_RAD });
        assert_eq!(buffer.len(), 30);
        assert_eq!(buffer[0].len(), 40);
    }

    #[test]
    fn test_render_single_sphere_center_hit() {
        let scene = Scene::new_with_sphere();
        let cam = Camera::new(Vector3::new(0.0, 0.0, 0.0), 4.0, 3.0);
        let w = 40_usize; let h = 30_usize;
        let buffer = render(&scene, &cam, w, h, RenderMode::Wireframe { step_rad: DEFAULT_WIREFRAME_STEP_RAD, tol_rad: DEFAULT_WIREFRAME_TOL_RAD });
        let center = buffer[h/2][w/2];
        assert!(center > 0.0, "Center should hit sphere");
        // corners should be mostly background
        assert!(buffer[0][0] < 0.5 && buffer[0][w-1] < 0.5 && buffer[h-1][0] < 0.5 && buffer[h-1][w-1] < 0.5);
    }
}

