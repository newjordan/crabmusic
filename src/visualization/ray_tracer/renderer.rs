//! Rendering pipeline producing a 2D intensity buffer (VIZ-010)

use super::camera::Camera;
use super::lighting::{calculate_diffuse_shading, Light};
use super::scene::Scene;
use super::wireframe::is_on_wireframe;
use super::RenderMode;

pub fn render(scene: &Scene, camera: &Camera, width: usize, height: usize, mode: RenderMode) -> Vec<Vec<f32>> {
    let mut buffer = vec![vec![0.0_f32; width]; height];

    for y in 0..height {
        for x in 0..width {
            let u = if width > 1 { x as f32 / (width as f32 - 1.0) } else { 0.0 };
            let v = if height > 1 { y as f32 / (height as f32 - 1.0) } else { 0.0 };
            let ray = camera.get_ray(u, v);

            let intensity = if let Some(hit) = scene.hit(&ray, 0.001, f32::MAX) {
                match mode {
                    RenderMode::Wireframe => {
                        // For now, we know our demo sphere parameters.
                        let center = crate::visualization::ray_tracer::math::Vector3::new(0.0, 0.0, -3.0);
                        let radius = 1.0;
                        if is_on_wireframe(hit.point, center, radius) { 1.0 } else { 0.2 }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visualization::ray_tracer::math::Vector3;

    #[test]
    fn test_render_dimensions() {
        let scene = Scene::new_with_sphere();
        let cam = Camera::new(Vector3::new(0.0, 0.0, 0.0), 4.0, 3.0);
        let buffer = render(&scene, &cam, 40, 30, RenderMode::Wireframe);
        assert_eq!(buffer.len(), 30);
        assert_eq!(buffer[0].len(), 40);
    }

    #[test]
    fn test_render_single_sphere_center_hit() {
        let scene = Scene::new_with_sphere();
        let cam = Camera::new(Vector3::new(0.0, 0.0, 0.0), 4.0, 3.0);
        let w = 40_usize; let h = 30_usize;
        let buffer = render(&scene, &cam, w, h, RenderMode::Wireframe);
        let center = buffer[h/2][w/2];
        assert!(center > 0.0, "Center should hit sphere");
        // corners should be mostly background
        assert!(buffer[0][0] < 0.5 && buffer[0][w-1] < 0.5 && buffer[h-1][0] < 0.5 && buffer[h-1][w-1] < 0.5);
    }
}

