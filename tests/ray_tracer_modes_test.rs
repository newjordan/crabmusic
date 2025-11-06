use crabmusic::visualization::ray_tracer::*;

#[test]
fn test_wireframe_mode_produces_lines() {
    let scene = Scene::new_with_sphere();
    let camera = Camera::new(Vector3::new(0.0, 0.0, 0.0), 4.0, 3.0);
    let (w, h) = (80_usize, 60_usize);
    let buffer = render(&scene, &camera, w, h, RenderMode::Wireframe { step_rad: DEFAULT_WIREFRAME_STEP_RAD, tol_rad: DEFAULT_WIREFRAME_TOL_RAD });

    // Sample a horizontal scanline through the center and ensure we see some bright points
    let y = h / 2;
    let bright = buffer[y].iter().filter(|v| **v > 0.9).count();
    assert!(bright >= 1, "Expected at least one wireframe line across center");
}

#[test]
fn test_solid_mode_has_nonzero_center_with_light() {
    let scene = Scene::new_with_sphere_and_light();
    let camera = Camera::new(Vector3::new(0.0, 0.0, 0.0), 4.0, 3.0);
    let (w, h) = (40_usize, 30_usize);
    let buffer = render(&scene, &camera, w, h, RenderMode::Solid);
    let center = buffer[h/2][w/2];
    assert!(center > 0.2, "Center should be lit by the scene light");
}

