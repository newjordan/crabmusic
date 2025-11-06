use crabmusic::visualization::ray_tracer::*;

fn main() {
    let scene = Scene::new_with_sphere_and_light();
    let camera = Camera::new(Vector3::new(0.0, 0.0, 0.0), 4.0, 3.0);

    let (w, h) = (160_usize, 96_usize);

    let mode_arg = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "wireframe".to_string());
    let mode = if mode_arg.eq_ignore_ascii_case("solid") {
        RenderMode::Solid
    } else {
        RenderMode::Wireframe {
            step_rad: DEFAULT_WIREFRAME_STEP_RAD,
            tol_rad: DEFAULT_WIREFRAME_TOL_RAD,
        }
    };

    let buffer = render(&scene, &camera, w, h, mode);
    let text = intensity_buffer_to_green_braille(&buffer);
    print!("{}", text);
}
