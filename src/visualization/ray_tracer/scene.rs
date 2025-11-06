//! Scene container and intersection (VIZ-010)

use super::hittable::{HitRecord, Hittable};
use super::lighting::Light;
use super::math::{Ray, Vector3};
use super::sphere::Sphere;

pub struct Scene {
    pub(crate) objects: Vec<Box<dyn Hittable + Send + Sync>>, // allow future parallelism
    pub(crate) lights: Vec<Light>,
}

impl Scene {
    pub fn new() -> Self { Self { objects: Vec::new(), lights: Vec::new() } }

    pub fn new_with_sphere() -> Self {
        let sphere = Box::new(Sphere::new(Vector3::new(0.0, 0.0, -3.0), 1.0));
        Self { objects: vec![sphere], lights: Vec::new() }
    }

    pub fn new_with_sphere_and_light() -> Self {
        let sphere = Box::new(Sphere::new(Vector3::new(0.0, 0.0, -3.0), 1.0));
        let light = Light::new(Vector3::new(-2.0, 2.0, 0.0), 1.0);
        Self { objects: vec![sphere], lights: vec![light] }
    }

    pub fn add_object(&mut self, obj: Box<dyn Hittable + Send + Sync>) { self.objects.push(obj); }
    pub fn add_light(&mut self, light: Light) { self.lights.push(light); }

    pub fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut closest_so_far = t_max;
        let mut best: Option<HitRecord> = None;
        for obj in &self.objects {
            if let Some(hr) = obj.hit(ray, t_min, closest_so_far) {
                closest_so_far = hr.t;
                best = Some(hr);
            }
        }
        best
    }

    pub fn lights(&self) -> &[Light] { &self.lights }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_single_sphere() {
        let scene = Scene::new_with_sphere();
        assert_eq!(scene.objects.len(), 1);
    }

    #[test]
    fn test_scene_hit_and_miss() {
        let scene = Scene::new_with_sphere();
        let cam_origin = Vector3::new(0.0, 0.0, 0.0);
        let hit_ray = Ray::new(cam_origin, Vector3::new(0.0, 0.0, -1.0));
        assert!(scene.hit(&hit_ray, 0.001, f32::MAX).is_some());
        let miss_ray = Ray::new(cam_origin, Vector3::new(1.0, 0.0, 0.0));
        assert!(scene.hit(&miss_ray, 0.001, f32::MAX).is_none());
    }
}

