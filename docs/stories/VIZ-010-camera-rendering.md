# VIZ-010: Camera & Rendering Pipeline

**Epic**: Epic 10: Braille-Based 3D Model Viewer
**Priority**: P3 (Experimental)
**Estimated Effort**: 2-3 days
**Status**: Draft

---

## Story

**As a** developer building a 3D ray tracer,
**I want** a camera system that generates rays and a rendering pipeline that casts rays to produce an intensity buffer,
**so that** I can render a 3D scene to a 2D array of intensity values suitable for visualization.

---

## Description

Implement the camera model and core rendering pipeline for the ray tracer. The camera generates rays for each pixel, and the renderer orchestrates the ray casting process to produce a 2D intensity buffer.

This story focuses on the rendering infrastructure without lighting calculations (VIZ-011) or display output (VIZ-012). The output is a raw 2D array of intensity values (0.0 to 1.0) representing hit/miss information.

Key components:
- **Camera**: Generates rays from a viewpoint through a virtual viewport
- **Scene**: Container for 3D objects (initially a single hard-coded sphere)
- **Renderer**: Main rendering loop that casts rays and collects intersection data
- **Intensity Buffer**: 2D array output representing the rendered image

---

## Acceptance Criteria

- [ ] `Camera` struct implemented with configurable viewport and focal length
- [ ] Camera can generate rays for any (u, v) pixel coordinate in range [0.0, 1.0]
- [ ] Generated rays are correctly normalized
- [ ] `Scene` struct contains a collection of Hittable objects
- [ ] Scene provides hard-coded initialization with a single sphere
- [ ] `render()` function takes scene, camera, width, height and returns 2D intensity buffer
- [ ] Rendering loop iterates over all pixels and casts rays
- [ ] Hit detection returns 1.0 for intersections, 0.0 for misses (placeholder shading)
- [ ] Intensity buffer has correct dimensions (height × width)
- [ ] Integration test verifies sphere appears in center of rendered buffer
- [ ] Performance target: <100ms for 80×24 buffer on reference hardware
- [ ] All code formatted with `rustfmt` and passes `clippy`
- [ ] Public APIs have documentation comments

---

## Technical Approach

### Module Structure

Add to `src/visualization/ray_tracer/`:

```
src/visualization/ray_tracer/
├── mod.rs          # Module exports
├── math.rs         # (from VIZ-009)
├── hittable.rs     # (from VIZ-009)
├── sphere.rs       # (from VIZ-009)
├── camera.rs       # 🆕 Camera implementation
├── scene.rs        # 🆕 Scene container
└── renderer.rs     # 🆕 Rendering pipeline
```

### Camera Design

Simple perspective camera:

```rust
pub struct Camera {
    pub origin: Vector3,
    pub viewport_width: f32,
    pub viewport_height: f32,
    pub focal_length: f32,
}

impl Camera {
    pub fn new(origin: Vector3, viewport_width: f32, viewport_height: f32) -> Self {
        Self {
            origin,
            viewport_width,
            viewport_height,
            focal_length: 1.0,
        }
    }

    /// Generate ray for pixel coordinates (u, v) in range [0.0, 1.0]
    pub fn get_ray(&self, u: f32, v: f32) -> Ray {
        // Calculate viewport position
        // Return normalized ray from origin through viewport point
    }
}
```

### Scene Structure

```rust
pub struct Scene {
    objects: Vec<Box<dyn Hittable>>,
}

impl Scene {
    pub fn new_with_sphere() -> Self {
        // Hard-coded sphere at origin with radius 1.0
        let sphere = Box::new(Sphere::new(Vector3::new(0.0, 0.0, -3.0), 1.0));
        Self {
            objects: vec![sphere],
        }
    }

    pub fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        // Test ray against all objects, return closest hit
    }
}
```

### Rendering Pipeline

```rust
pub fn render(
    scene: &Scene,
    camera: &Camera,
    width: usize,
    height: usize,
) -> Vec<Vec<f32>> {
    let mut buffer = vec![vec![0.0; width]; height];

    for y in 0..height {
        for x in 0..width {
            let u = x as f32 / (width - 1) as f32;
            let v = y as f32 / (height - 1) as f32;

            let ray = camera.get_ray(u, v);

            // Simple hit/miss shading (no lighting yet)
            let intensity = if scene.hit(&ray, 0.001, f32::MAX).is_some() {
                1.0 // Hit
            } else {
                0.0 // Miss
            };

            buffer[y][x] = intensity;
        }
    }

    buffer
}
```

---

## Dependencies

**Depends on**:
- VIZ-009 (Ray Tracing Primitives) - needs Vector3, Ray, Hittable, Sphere

**Blocks**:
- VIZ-011 (Lighting & Shading) - will enhance the intensity calculation
- VIZ-012 (Braille Conversion) - needs intensity buffer output

---

## Architecture References

- **Source Tree**: `docs/architecture/source-tree.md`
  - Visualization module: Lines 218-232
  - Module organization principles: Lines 348-366

- **Coding Standards**: `docs/architecture/coding-standards.md`
  - Pre-allocated buffers: Lines 51-56
  - No allocations in hot paths: Lines 204-226
  - Performance profiling: Lines 241-249

- **Tech Stack**: `docs/architecture/tech-stack.md`
  - Rust version 1.75+: Line 11

---

## Testing Requirements

### Unit Tests

**camera.rs tests**:
- `test_camera_ray_generation` - Verify rays pass through correct viewport points
- `test_camera_ray_normalized` - Ensure generated ray directions are normalized
- `test_camera_corners` - Test rays at viewport corners (u,v = 0,0 / 0,1 / 1,0 / 1,1)

**scene.rs tests**:
- `test_scene_single_sphere` - Scene contains sphere
- `test_scene_hit` - Ray intersects sphere in scene
- `test_scene_miss` - Ray misses all objects
- `test_scene_closest_hit` - Multiple objects, returns closest (future enhancement)

**renderer.rs tests**:
- `test_render_dimensions` - Output buffer has correct dimensions
- `test_render_empty_scene` - All zeros for empty scene
- `test_render_single_sphere` - Non-zero intensities where sphere projects

### Integration Test

Create `tests/ray_tracer_test.rs`:

```rust
#[test]
fn test_ray_tracer_renders_sphere() {
    let scene = Scene::new_with_sphere();
    let camera = Camera::new(
        Vector3::new(0.0, 0.0, 0.0),
        4.0,
        3.0,
    );
    let buffer = render(&scene, &camera, 40, 30);

    // Verify dimensions
    assert_eq!(buffer.len(), 30);
    assert_eq!(buffer[0].len(), 40);

    // Verify center has hit (sphere projects to center)
    let center_intensity = buffer[15][20];
    assert!(center_intensity > 0.0, "Center should hit sphere");

    // Verify corners are misses
    let corner_intensity = buffer[0][0];
    assert!(corner_intensity < 0.1, "Corner should miss sphere");
}
```

---

## Notes for Dev Agent

### Implementation Order

1. **Camera** (camera.rs):
   - Implement struct with viewport parameters
   - Implement `get_ray()` with viewport coordinate calculation
   - Unit test ray generation at various u,v coordinates

2. **Scene** (scene.rs):
   - Implement container for Hittable objects
   - Implement hard-coded `new_with_sphere()`
   - Implement `hit()` that finds closest intersection

3. **Renderer** (renderer.rs):
   - Pre-allocate 2D buffer
   - Implement nested pixel loop
   - Simple hit/miss intensity (0.0 or 1.0)
   - Integration test to verify sphere rendering

### Performance Considerations

**Pre-allocation**:
- Allocate buffer once before pixel loop: `vec![vec![0.0; width]; height]`
- Do NOT allocate inside the nested loop

**Profiling**:
If performance issues arise:
```bash
cargo build --release
time target/release/examples/ray_tracer_demo
```

Expected: 80×24 should render in ~10-50ms on modern hardware

**Optimization Opportunities** (if needed):
- Parallel pixel loop with rayon (future enhancement)
- SIMD vector operations (future enhancement)
- For MVP, simple scalar code is sufficient

### Common Pitfalls

1. **Coordinate system**: Match viewport u,v to terminal y,x correctly
2. **t_min value**: Use 0.001 to avoid shadow acne (self-intersection)
3. **Ray direction normalization**: Camera should generate normalized rays
4. **Aspect ratio**: viewport_width/viewport_height should match output width/height

### Code Quality

Run before committing:
```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test --lib
cargo test --test ray_tracer_test
```

---

## Completion Checklist

- [ ] Camera struct implemented
- [ ] Camera generates correct rays
- [ ] Scene struct implemented
- [ ] Scene contains hard-coded sphere
- [ ] Renderer produces correct dimensions
- [ ] Integration test verifies sphere rendering
- [ ] Performance <100ms for 80×24
- [ ] All unit tests pass
- [ ] Documentation comments complete
- [ ] Code passes rustfmt and clippy

---

## Change Log

| Date | Version | Description | Author |
|------|---------|-------------|--------|
| 2025-11-05 | 1.0 | Initial story creation - split from VIZ-009-MVP | Sarah (PO) |
