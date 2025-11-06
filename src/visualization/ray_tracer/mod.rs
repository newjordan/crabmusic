//! Ray tracer foundational module (VIZ-009)

pub mod math;
pub mod hittable;
pub mod sphere;
pub mod camera;
pub mod scene;
pub mod renderer;
pub mod wireframe;
pub mod lighting;
pub mod braille;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenderMode {
    Wireframe,
    Solid,
}

impl Default for RenderMode {
    fn default() -> Self { RenderMode::Wireframe }
}

pub use math::{Ray, Vector3};
pub use hittable::{HitRecord, Hittable};
pub use sphere::Sphere;
pub use camera::Camera;
pub use scene::Scene;
pub use renderer::render;
pub use braille::intensity_buffer_to_green_braille;
pub use RenderMode::*;

