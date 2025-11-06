//! glTF Model Viewer Example
//!
//! This example demonstrates loading and rendering 3D models from glTF/GLB files
//! using the ray tracer. It supports both wireframe and solid rendering modes.
//!
//! Usage:
//!   cargo run --example gltf_viewer <path-to-model.glb> [wireframe|solid]
//!
//! Examples:
//!   cargo run --example gltf_viewer models/cube.glb
//!   cargo run --example gltf_viewer models/cube.glb wireframe
//!   cargo run --example gltf_viewer models/suzanne.glb solid

use crabmusic::visualization::ray_tracer::*;
use std::env;
use std::process;

fn main() {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <path-to-model.glb> [wireframe|solid]", args[0]);
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  {} models/cube.glb", args[0]);
        eprintln!("  {} models/cube.glb wireframe", args[0]);
        eprintln!("  {} models/suzanne.glb solid", args[0]);
        eprintln!();
        eprintln!("Render modes:");
        eprintln!("  wireframe - Classic vector display look (default)");
        eprintln!("  solid     - Realistic lighting with diffuse shading");
        process::exit(1);
    }

    let model_path = &args[1];
    let mode_arg = args.get(2).map(|s| s.as_str()).unwrap_or("wireframe");

    // Parse render mode
    let mode = if mode_arg.eq_ignore_ascii_case("solid") {
        RenderMode::Solid
    } else {
        RenderMode::Wireframe {
            step_rad: DEFAULT_WIREFRAME_STEP_RAD,
            tol_rad: DEFAULT_WIREFRAME_TOL_RAD,
        }
    };

    // Load the glTF model
    println!("Loading model: {}", model_path);
    let scene = match Scene::new_with_model(model_path) {
        Ok(scene) => {
            println!("✓ Model loaded successfully");
            scene
        }
        Err(e) => {
            eprintln!("✗ Failed to load model: {}", e);
            eprintln!();
            eprintln!("Make sure the file exists and is a valid glTF/GLB file.");
            eprintln!();
            eprintln!("You can download sample models from:");
            eprintln!("  https://github.com/KhronosGroup/glTF-Sample-Models");
            process::exit(1);
        }
    };

    // Set up camera
    // Position camera to view the model (adjust based on your model's size/position)
    let camera = Camera::new(
        Vector3::new(0.0, 0.0, 5.0), // Camera position (5 units back from origin)
        4.0,                         // Viewport width
        3.0,                         // Viewport height
    );

    // Render dimensions (high resolution for detail)
    let (w, h) = (160_usize, 96_usize);

    println!("Rendering in {:?} mode...", mode);
    let buffer = render(&scene, &camera, w, h, mode);

    // Convert to green Braille characters for terminal display
    let text = intensity_buffer_to_green_braille(&buffer);

    // Print the rendered model
    println!();
    print!("{}", text);
    println!();
    println!("Render complete!");
}
