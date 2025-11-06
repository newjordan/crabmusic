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
    /// Wireframe grid with configurable spacing and thickness (radians)
    Wireframe { step_rad: f32, tol_rad: f32 },
    Solid,
}

impl Default for RenderMode {
    fn default() -> Self {
        RenderMode::Wireframe {
            step_rad: wireframe::DEFAULT_WIREFRAME_STEP_RAD,
            tol_rad: wireframe::DEFAULT_WIREFRAME_TOL_RAD,
        }
    }
}

pub use math::{Ray, Vector3};
pub use hittable::{HitRecord, Hittable};
pub use sphere::Sphere;
pub use camera::Camera;
pub use scene::Scene;
pub use renderer::{render, render_with_orientation, WireframeRotation};
pub use braille::intensity_buffer_to_green_braille;
pub use wireframe::{DEFAULT_WIREFRAME_STEP_RAD, DEFAULT_WIREFRAME_TOL_RAD};
pub use RenderMode::*;

