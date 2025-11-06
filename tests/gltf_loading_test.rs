//! Integration tests for glTF model loading and rendering
//!
//! These tests verify that the glTF loading pipeline works end-to-end:
//! 1. Load mesh data from glTF file
//! 2. Build triangle mesh
//! 3. Add to scene
//! 4. Render with ray tracer

use crabmusic::visualization::ray_tracer::*;

#[test]
fn test_gltf_loader_with_manual_data() {
    // Since we don't have test glTF files in the repo yet,
    // we'll test the pipeline by creating MeshData manually
    // and verifying the mesh construction works

    use crabmusic::visualization::ray_tracer::gltf_loader::MeshData;
    use crabmusic::visualization::ray_tracer::mesh::TriangleMesh;

    // Create a simple triangle mesh data (one triangle)
    let mesh_data = MeshData {
        positions: vec![
            Vector3::new(-1.0, 0.0, -3.0),
            Vector3::new(1.0, 0.0, -3.0),
            Vector3::new(0.0, 1.0, -3.0),
        ],
        normals: None,
        indices: vec![0, 1, 2],
    };

    // Build mesh from data
    let mesh = TriangleMesh::from_gltf_data(mesh_data);
    assert_eq!(mesh.triangle_count(), 1);

    // Add to scene
    let mut scene = Scene::new();
    scene.add_object(Box::new(mesh));
    scene.add_light(Light::new(Vector3::new(2.0, 2.0, 0.0), 1.0));

    // Render
    let camera = Camera::new(Vector3::new(0.0, 0.0, 0.0), 4.0, 3.0);
    let buffer = render(&scene, &camera, 40, 30, RenderMode::Solid);

    // Verify we got a buffer
    assert_eq!(buffer.len(), 30);
    assert_eq!(buffer[0].len(), 40);

    // Verify we hit something (center of view should hit the triangle)
    let center_intensity = buffer[15][20];
    assert!(
        center_intensity > 0.0,
        "Center should hit triangle, got intensity: {}",
        center_intensity
    );
}

#[test]
fn test_mesh_rendering_wireframe_mode() {
    use crabmusic::visualization::ray_tracer::gltf_loader::MeshData;
    use crabmusic::visualization::ray_tracer::mesh::TriangleMesh;

    // Create a simple quad (2 triangles)
    let mesh_data = MeshData {
        positions: vec![
            Vector3::new(-1.0, -1.0, -3.0), // Bottom-left
            Vector3::new(1.0, -1.0, -3.0),  // Bottom-right
            Vector3::new(1.0, 1.0, -3.0),   // Top-right
            Vector3::new(-1.0, 1.0, -3.0),  // Top-left
        ],
        normals: None,
        indices: vec![
            0, 1, 2, // First triangle
            0, 2, 3, // Second triangle
        ],
    };

    let mesh = TriangleMesh::from_gltf_data(mesh_data);
    assert_eq!(mesh.triangle_count(), 2);

    // Add to scene
    let mut scene = Scene::new();
    scene.add_object(Box::new(mesh));

    // Render in wireframe mode
    let camera = Camera::new(Vector3::new(0.0, 0.0, 0.0), 4.0, 3.0);
    let buffer = render(
        &scene,
        &camera,
        40,
        30,
        RenderMode::Wireframe {
            step_rad: DEFAULT_WIREFRAME_STEP_RAD,
            tol_rad: DEFAULT_WIREFRAME_TOL_RAD,
        },
    );

    // Verify buffer dimensions
    assert_eq!(buffer.len(), 30);
    assert_eq!(buffer[0].len(), 40);

    // In wireframe mode, we should have some hits (edges) and some misses (interior)
    let mut hit_count = 0;
    let mut miss_count = 0;

    for row in &buffer {
        for &intensity in row {
            if intensity > 0.5 {
                hit_count += 1;
            } else {
                miss_count += 1;
            }
        }
    }

    // Wireframe should have both hits (edges) and misses (interior/background)
    assert!(hit_count > 0, "Wireframe should have some edge pixels");
    assert!(miss_count > 0, "Wireframe should have some non-edge pixels");
}

#[test]
fn test_mesh_with_smooth_normals() {
    use crabmusic::visualization::ray_tracer::gltf_loader::MeshData;
    use crabmusic::visualization::ray_tracer::mesh::TriangleMesh;

    // Create a triangle with smooth normals (all pointing in +Z)
    let mesh_data = MeshData {
        positions: vec![
            Vector3::new(-1.0, 0.0, -3.0),
            Vector3::new(1.0, 0.0, -3.0),
            Vector3::new(0.0, 1.0, -3.0),
        ],
        normals: Some(vec![
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(0.0, 0.0, 1.0),
        ]),
        indices: vec![0, 1, 2],
    };

    let mesh = TriangleMesh::from_gltf_data(mesh_data);
    assert_eq!(mesh.triangle_count(), 1);

    // Add to scene with light
    let mut scene = Scene::new();
    scene.add_object(Box::new(mesh));
    scene.add_light(Light::new(Vector3::new(0.0, 0.0, 5.0), 1.0)); // Light in front

    // Render in solid mode
    let camera = Camera::new(Vector3::new(0.0, 0.0, 0.0), 4.0, 3.0);
    let buffer = render(&scene, &camera, 40, 30, RenderMode::Solid);

    // With normals pointing toward the light, we should get good illumination
    let center_intensity = buffer[15][20];
    assert!(
        center_intensity > 0.5,
        "Smooth normals should produce good lighting, got: {}",
        center_intensity
    );
}

#[test]
fn test_multiple_meshes_in_scene() {
    use crabmusic::visualization::ray_tracer::gltf_loader::MeshData;
    use crabmusic::visualization::ray_tracer::mesh::TriangleMesh;

    // Create two separate triangle meshes at different depths
    let mesh_data_1 = MeshData {
        positions: vec![
            Vector3::new(-1.0, 0.0, -5.0), // Far triangle
            Vector3::new(1.0, 0.0, -5.0),
            Vector3::new(0.0, 1.0, -5.0),
        ],
        normals: None,
        indices: vec![0, 1, 2],
    };

    let mesh_data_2 = MeshData {
        positions: vec![
            Vector3::new(-0.5, -0.5, -3.0), // Near triangle (smaller)
            Vector3::new(0.5, -0.5, -3.0),
            Vector3::new(0.0, 0.5, -3.0),
        ],
        normals: None,
        indices: vec![0, 1, 2],
    };

    let mesh1 = TriangleMesh::from_gltf_data(mesh_data_1);
    let mesh2 = TriangleMesh::from_gltf_data(mesh_data_2);

    // Add both to scene
    let mut scene = Scene::new();
    scene.add_object(Box::new(mesh1));
    scene.add_object(Box::new(mesh2));
    scene.add_light(Light::new(Vector3::new(2.0, 2.0, 0.0), 1.0));

    // Render
    let camera = Camera::new(Vector3::new(0.0, 0.0, 0.0), 4.0, 3.0);
    let buffer = render(&scene, &camera, 40, 30, RenderMode::Solid);

    // Should hit something in the center (the nearer triangle)
    let center_intensity = buffer[15][20];
    assert!(center_intensity > 0.0, "Should hit one of the triangles");
}

#[test]
fn test_mesh_and_sphere_together() {
    use crabmusic::visualization::ray_tracer::gltf_loader::MeshData;
    use crabmusic::visualization::ray_tracer::mesh::TriangleMesh;

    // Create a triangle mesh
    let mesh_data = MeshData {
        positions: vec![
            Vector3::new(-1.0, -1.0, -3.0),
            Vector3::new(1.0, -1.0, -3.0),
            Vector3::new(0.0, 0.0, -3.0),
        ],
        normals: None,
        indices: vec![0, 1, 2],
    };

    let mesh = TriangleMesh::from_gltf_data(mesh_data);

    // Create a sphere
    let sphere = Sphere::new(Vector3::new(0.0, 1.0, -3.0), 0.5);

    // Add both to scene
    let mut scene = Scene::new();
    scene.add_object(Box::new(mesh));
    scene.add_object(Box::new(sphere));
    scene.add_light(Light::new(Vector3::new(2.0, 2.0, 0.0), 1.0));

    // Render
    let camera = Camera::new(Vector3::new(0.0, 0.0, 0.0), 4.0, 3.0);
    let buffer = render(&scene, &camera, 40, 30, RenderMode::Solid);

    // Should hit something (either mesh or sphere depending on ray direction)
    let mut total_hits = 0;
    for row in &buffer {
        for &intensity in row {
            if intensity > 0.0 {
                total_hits += 1;
            }
        }
    }

    assert!(total_hits > 0, "Should hit either the mesh or sphere");
}

// Note: To test actual glTF file loading, you would need to:
// 1. Add test glTF files to a tests/assets/ directory
// 2. Use Scene::new_with_model("tests/assets/cube.glb")
// 3. Verify the scene loads and renders correctly
//
// For now, these tests verify the pipeline works with manually constructed data.
