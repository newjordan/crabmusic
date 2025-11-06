# VIZ-011: Wireframe Rendering & Shading Modes

**Epic**: Epic 10: Braille-Based 3D Model Viewer
**Priority**: P3 (Experimental)
**Estimated Effort**: 2-3 days
**Status**: Draft

---

## Story

**As a** developer building a 3D ray tracer with retro terminal aesthetics,
**I want** wireframe rendering as the primary mode (classic vector display look) plus solid shading as an alternative mode,
**so that** I can achieve the iconic green-wireframe-on-black aesthetic while maintaining flexibility for realistic rendering.

---

## Description

Implement **two rendering modes** for the ray tracer, with **wireframe as the default**:

### Primary Mode: Wireframe Rendering (Default)
Render the sphere as a classic vector display wireframe using latitude/longitude grid lines. This creates the iconic retro terminal aesthetic (think Matrix, vector monitors, oscilloscopes).

**Wireframe characteristics**:
- Sphere rendered as grid of meridian and parallel lines
- Only edges/intersections shown (not filled surface)
- Clean, geometric appearance
- Perfect for Braille terminal display
- Classic green-on-black look (colors handled in VIZ-012)

### Secondary Mode: Solid Shading
Traditional diffuse (Lambertian) shading for realistic lighting when needed.

**Key components**:
- **RenderMode enum**: Wireframe (default) or Solid
- **Wireframe renderer**: Grid line detection and edge rendering
- **Light** (for solid mode): Point light source with position and intensity
- **Diffuse Shading** (for solid mode): Lambertian reflectance model

The output remains a 2D intensity buffer, but wireframe mode produces high-contrast edges while solid mode produces smooth gradients.

---

## Acceptance Criteria

### Core Requirements
- [ ] `RenderMode` enum with Wireframe and Solid variants
- [ ] Renderer accepts RenderMode parameter (defaults to Wireframe)
- [ ] Scene creation includes default render mode

### Wireframe Mode (Primary)
- [ ] Sphere rendered as latitude/longitude grid with configurable divisions (e.g., 16 meridians, 12 parallels)
- [ ] Wireframe edge detection correctly identifies grid line intersections
- [ ] Grid line pixels return 1.0 intensity, non-grid pixels return 0.0
- [ ] Wireframe produces clean, recognizable sphere outline
- [ ] Unit test verifies grid line calculation at various angles
- [ ] Integration test shows wireframe sphere with visible latitude/longitude lines

### Solid Shading Mode (Secondary)
- [ ] `Light` struct implemented with position (Vector3) and intensity (f32)
- [ ] Scene can contain a hard-coded light source (for solid mode)
- [ ] `calculate_diffuse_shading()` function implements Lambertian reflection
- [ ] Diffuse shading uses surface normal and light direction (dot product)
- [ ] Shading calculation clamps negative values to 0.0
- [ ] Solid mode shows gradual intensity falloff from light-facing to shadow side
- [ ] Unit tests verify shading calculations for various normal/light orientations
- [ ] Integration test verifies sphere has proper gradient in solid mode

### General
- [ ] All code formatted with `rustfmt` and passes `clippy`
- [ ] Public APIs have documentation comments
- [ ] Mode switching works correctly (can render both modes)

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
├── camera.rs       # (from VIZ-010)
├── scene.rs        # (from VIZ-010) - add render mode & lights
├── renderer.rs     # (from VIZ-010) - enhance with modes
├── wireframe.rs    # 🆕 Wireframe grid calculation
└── lighting.rs     # 🆕 Light struct and shading functions
```

### RenderMode Enum

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenderMode {
    Wireframe,
    Solid,
}

impl Default for RenderMode {
    fn default() -> Self {
        RenderMode::Wireframe // Default to wireframe!
    }
}
```

### Wireframe Rendering Algorithm

The key to wireframe is detecting when a ray hits near a latitude or longitude line on the sphere:

```rust
/// Check if hit point is on a wireframe grid line
pub fn is_on_wireframe(
    hit_point: &Vector3,
    sphere_center: &Vector3,
    sphere_radius: f32,
    meridians: usize,      // Longitude lines (e.g., 16)
    parallels: usize,      // Latitude lines (e.g., 12)
    line_width: f32,       // Angular tolerance (e.g., 0.05 radians)
) -> bool {
    // Convert hit point to spherical coordinates (theta, phi)
    let relative = *hit_point - *sphere_center;

    // Theta: angle around Y axis (longitude) [0, 2π]
    let theta = relative.z.atan2(relative.x);

    // Phi: angle from Y axis (latitude) [0, π]
    let phi = (relative.y / sphere_radius).acos();

    // Check if near a meridian line (longitude)
    let meridian_step = std::f32::consts::TAU / meridians as f32;
    let theta_mod = (theta % meridian_step).abs();
    let on_meridian = theta_mod < line_width || theta_mod > (meridian_step - line_width);

    // Check if near a parallel line (latitude)
    let parallel_step = std::f32::consts::PI / parallels as f32;
    let phi_mod = (phi % parallel_step).abs();
    let on_parallel = phi_mod < line_width || phi_mod > (parallel_step - line_width);

    on_meridian || on_parallel
}
```

### Light Structure

```rust
pub struct Light {
    pub position: Vector3,
    pub intensity: f32, // 0.0 to 1.0
}

impl Light {
    pub fn new(position: Vector3, intensity: f32) -> Self {
        Self { position, intensity }
    }
}
```

### Diffuse Shading Calculation

```rust
/// Calculate diffuse (Lambertian) shading intensity
pub fn calculate_diffuse_shading(
    hit_record: &HitRecord,
    light: &Light,
) -> f32 {
    // Direction from hit point to light
    let light_dir = (light.position - hit_record.point).normalize();

    // Lambertian reflectance: N · L
    let diffuse = hit_record.normal.dot(&light_dir).max(0.0);

    // Scale by light intensity
    diffuse * light.intensity
}
```

### Enhanced Scene

```rust
pub struct Scene {
    objects: Vec<Box<dyn Hittable>>,
    lights: Vec<Light>, // 🆕
}

impl Scene {
    pub fn new_with_sphere_and_light() -> Self {
        let sphere = Box::new(Sphere::new(Vector3::new(0.0, 0.0, -3.0), 1.0));
        let light = Light::new(Vector3::new(2.0, 2.0, 0.0), 1.0);

        Self {
            objects: vec![sphere],
            lights: vec![light],
        }
    }

    pub fn get_lights(&self) -> &[Light] {
        &self.lights
    }
}
```

### Enhanced Renderer (Both Modes)

```rust
pub fn render(
    scene: &Scene,
    camera: &Camera,
    width: usize,
    height: usize,
    mode: RenderMode,
) -> Vec<Vec<f32>> {
    let mut buffer = vec![vec![0.0; width]; height];

    for y in 0..height {
        for x in 0..width {
            let u = x as f32 / (width - 1) as f32;
            let v = y as f32 / (height - 1) as f32;

            let ray = camera.get_ray(u, v);

            let intensity = match scene.hit(&ray, 0.001, f32::MAX) {
                Some(hit_record) => {
                    match mode {
                        RenderMode::Wireframe => {
                            // Wireframe mode: 1.0 if on grid line, 0.0 otherwise
                            if is_on_wireframe(
                                &hit_record.point,
                                &Vector3::new(0.0, 0.0, -3.0), // Sphere center
                                1.0,                            // Sphere radius
                                16,                             // Meridians
                                12,                             // Parallels
                                0.05,                           // Line width
                            ) {
                                1.0
                            } else {
                                0.0
                            }
                        }
                        RenderMode::Solid => {
                            // Solid mode: Calculate lighting from all lights
                            scene.get_lights()
                                .iter()
                                .map(|light| calculate_diffuse_shading(&hit_record, light))
                                .sum::<f32>()
                                .min(1.0) // Clamp to max intensity
                        }
                    }
                }
                None => 0.0, // Background (black)
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
- VIZ-009 (Ray Tracing Primitives) - needs Vector3, HitRecord
- VIZ-010 (Camera & Rendering) - enhances existing renderer

**Blocks**:
- VIZ-012 (Braille Conversion) - needs realistic intensity values

---

## Architecture References

- **Source Tree**: `docs/architecture/source-tree.md`
  - Visualization module: Lines 218-232

- **Coding Standards**: `docs/architecture/coding-standards.md`
  - Floating-point comparisons: Lines 69-77
  - Inline hints on hot paths: Lines 228-239

- **Tech Stack**: `docs/architecture/tech-stack.md`
  - Rust version 1.75+: Line 11

---

## Testing Requirements

### Unit Tests

**lighting.rs tests**:
```rust
#[test]
fn test_diffuse_shading_perpendicular() {
    // Arrange: Normal points directly at light
    let hit_record = HitRecord {
        point: Vector3::new(0.0, 0.0, 0.0),
        normal: Vector3::new(1.0, 0.0, 0.0),
        t: 1.0,
        front_face: true,
    };
    let light = Light::new(Vector3::new(10.0, 0.0, 0.0), 1.0);

    // Act
    let intensity = calculate_diffuse_shading(&hit_record, &light);

    // Assert: Should be maximum (1.0)
    assert!((intensity - 1.0).abs() < 1e-6);
}

#[test]
fn test_diffuse_shading_parallel() {
    // Arrange: Normal perpendicular to light (90 degrees)
    let hit_record = HitRecord {
        point: Vector3::new(0.0, 0.0, 0.0),
        normal: Vector3::new(0.0, 1.0, 0.0),
        t: 1.0,
        front_face: true,
    };
    let light = Light::new(Vector3::new(10.0, 0.0, 0.0), 1.0);

    // Act
    let intensity = calculate_diffuse_shading(&hit_record, &light);

    // Assert: Should be zero (no light contribution)
    assert!(intensity.abs() < 1e-6);
}

#[test]
fn test_diffuse_shading_behind() {
    // Arrange: Light behind surface (negative dot product)
    let hit_record = HitRecord {
        point: Vector3::new(0.0, 0.0, 0.0),
        normal: Vector3::new(1.0, 0.0, 0.0),
        t: 1.0,
        front_face: true,
    };
    let light = Light::new(Vector3::new(-10.0, 0.0, 0.0), 1.0);

    // Act
    let intensity = calculate_diffuse_shading(&hit_record, &light);

    // Assert: Should be zero (clamped negative)
    assert!(intensity.abs() < 1e-6);
}

#[test]
fn test_diffuse_shading_angle_45() {
    // Arrange: 45-degree angle
    let hit_record = HitRecord {
        point: Vector3::new(0.0, 0.0, 0.0),
        normal: Vector3::new(1.0, 0.0, 0.0).normalize(),
        t: 1.0,
        front_face: true,
    };
    let light = Light::new(Vector3::new(10.0, 10.0, 0.0), 1.0);

    // Act
    let intensity = calculate_diffuse_shading(&hit_record, &light);

    // Assert: Should be ~0.707 (cos(45°))
    assert!((intensity - 0.707).abs() < 0.01);
}
```

### Integration Test

Enhance `tests/ray_tracer_test.rs`:

```rust
#[test]
fn test_lit_sphere_has_gradient() {
    let scene = Scene::new_with_sphere_and_light();
    let camera = Camera::new(Vector3::new(0.0, 0.0, 0.0), 4.0, 3.0);
    let buffer = render(&scene, &camera, 40, 30);

    // Find brightest and darkest pixels on sphere
    let mut max_intensity = 0.0;
    let mut min_intensity = 1.0;

    for row in &buffer {
        for &intensity in row {
            if intensity > 0.0 { // Only consider hit pixels
                max_intensity = max_intensity.max(intensity);
                min_intensity = min_intensity.min(intensity);
            }
        }
    }

    // Verify gradient exists (lit side brighter than shadow side)
    assert!(max_intensity > 0.5, "Lit side should be bright");
    assert!(min_intensity < max_intensity * 0.5, "Shadow side should be darker");
}
```

---

## Notes for Dev Agent

### Implementation Order

1. **Light struct** (lighting.rs):
   - Simple struct with position and intensity
   - No complex logic needed

2. **Diffuse shading function** (lighting.rs):
   - Implement Lambertian formula
   - Unit tests for various angles
   - Remember to normalize light_dir
   - Clamp negative values to 0.0

3. **Enhance Scene** (scene.rs):
   - Add lights vector
   - Update `new_with_sphere_and_light()`
   - Add getter for lights

4. **Enhance Renderer** (renderer.rs):
   - Replace hit/miss with lighting calculation
   - Sum contributions from all lights
   - Clamp total intensity to 1.0

5. **Integration test**:
   - Verify gradient exists
   - Check realistic intensity distribution

### Visual Debugging

If output doesn't look right:
1. Print sample intensity values to console
2. Verify light position relative to sphere
3. Check normal vectors are correct (outward-facing)
4. Verify light direction normalization

### Performance

Lighting adds minimal overhead:
- One normalize() per ray (already fast)
- One dot product per ray
- Should still meet <100ms target for 80×24

### Common Pitfalls

1. **Forgetting max(0.0)**: Negative dot products must be clamped
2. **Not normalizing light_dir**: Direction vector must be unit length
3. **Light position**: Place light where it illuminates the sphere (not behind camera)
4. **Intensity accumulation**: Multiple lights should sum, then clamp to 1.0

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

- [ ] Light struct implemented
- [ ] Diffuse shading function implemented
- [ ] Shading unit tests pass (perpendicular, parallel, behind, angle)
- [ ] Scene enhanced with lights
- [ ] Renderer integrates lighting
- [ ] Integration test verifies gradient
- [ ] Visual output shows realistic sphere shading
- [ ] Documentation comments complete
- [ ] Code passes rustfmt and clippy

---

## Change Log

| Date | Version | Description | Author |
|------|---------|-------------|--------|
| 2025-11-05 | 1.0 | Initial story creation - split from VIZ-009-MVP | Sarah (PO) |
| 2025-11-05 | 1.1 | Updated to include wireframe mode as primary rendering mode | Sarah (PO) |
