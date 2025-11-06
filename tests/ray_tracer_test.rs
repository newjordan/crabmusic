use crabmusic::visualization::ray_tracer::*;

#[test]
fn test_ray_tracer_renders_sphere_center_and_corners() {
    let scene = Scene::new_with_sphere();
    let camera = Camera::new(Vector3::new(0.0, 0.0, 0.0), 4.0, 3.0);
    let (w, h) = (40_usize, 30_usize);

    let buffer = render(&scene, &camera, w, h, RenderMode::Wireframe);

    // Dimensions
    assert_eq!(buffer.len(), h);
    assert_eq!(buffer[0].len(), w);

    // Center should hit
    let center = buffer[h/2][w/2];
    assert!(center > 0.0, "Center should hit sphere");

    // Corners should miss
    assert!(buffer[0][0] < 0.1);
    assert!(buffer[0][w-1] < 0.1);
    assert!(buffer[h-1][0] < 0.1);
    assert!(buffer[h-1][w-1] < 0.1);
}

