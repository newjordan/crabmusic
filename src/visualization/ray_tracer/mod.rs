//! Ray tracer foundational module (VIZ-009)

pub mod braille;
pub mod camera;
mod gltf_loader;
pub mod hittable;
pub mod lighting;
pub mod math;
pub mod mesh;
// Removed deprecated glTF catalog/downloader modules
// mod model_catalog;
// mod model_downloader;
pub mod obj_loader;
pub mod renderer;
pub mod scene;
pub mod sphere;
pub mod triangle;
pub mod wireframe;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenderMode {
    /// Wireframe grid with configurable spacing and thickness (radians)
    Wireframe {
        step_rad: f32,
        tol_rad: f32,
    },
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

#[allow(unused_imports)]
pub use braille::intensity_buffer_to_green_braille;
#[allow(unused_imports)]
pub use camera::Camera;
#[allow(unused_imports)]
pub use hittable::{HitRecord, Hittable};
#[allow(unused_imports)]
pub use lighting::Light;
#[allow(unused_imports)]
pub use math::{Ray, Vector3};
#[allow(unused_imports)]
pub use mesh::TriangleMesh;
#[allow(unused_imports)]
pub use renderer::{
    render, render_edges_with_orientation, render_with_orientation, WireframeRotation,
};
#[allow(unused_imports)]
pub use scene::Scene;
#[allow(unused_imports)]
pub use sphere::Sphere;
#[allow(unused_imports)]
pub use triangle::Triangle;
#[allow(unused_imports)]
pub use wireframe::{DEFAULT_WIREFRAME_STEP_RAD, DEFAULT_WIREFRAME_TOL_RAD};
#[allow(unused_imports)]
pub use RenderMode::*;
