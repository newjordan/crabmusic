# VIZ-009: Ray Tracing Primitives & Math Foundation

**Epic**: Epic 10: Braille-Based 3D Model Viewer
**Priority**: P3 (Experimental)
**Estimated Effort**: 2-3 days
**Status**: Draft

---

## Story

**As a** developer building a 3D ray tracer for terminal rendering,
**I want** fundamental 3D math primitives (Vector3, Ray) and intersection testing (Hittable trait, Sphere),
**so that** I can establish the mathematical foundation for ray tracing and test ray-object intersections.

---

## Description

Implement the core mathematical primitives and intersection testing framework needed for ray tracing. This story establishes the foundational data structures and algorithms without rendering or display concerns.

This includes:
- **Vector3**: 3D vector struct with standard operations (add, subtract, multiply, dot product, cross product, normalization)
- **Ray**: Ray struct representing a parametric ray with origin and direction
- **Hittable Trait**: Interface for objects that can be intersected by rays
- **Sphere**: Sphere implementation with ray-sphere intersection algorithm
- **HitRecord**: Data structure capturing intersection details (point, normal, distance)

All implementations are pure Rust with no external math libraries (nalgebra, glam). This keeps dependencies minimal and provides learning value.

---

## Acceptance Criteria

- [ ] `Vector3` struct implemented with x, y, z fields
- [ ] Vector3 supports addition, subtraction, scalar multiplication
- [ ] Vector3 implements dot product, cross product, length, and normalization
- [ ] `Ray` struct implemented with origin (Vector3) and direction (Vector3)
- [ ] Ray has `at(t: f32)` method to get point along ray
- [ ] `Hittable` trait defined with `hit()` method signature
- [ ] `HitRecord` struct captures intersection point, normal, t-value, and front/back face
- [ ] `Sphere` struct implemented with center and radius
- [ ] Sphere implements `Hittable` trait with correct ray-sphere intersection math
- [ ] Ray-sphere intersection returns closest hit in valid t range
- [ ] Comprehensive unit tests for all vector operations
- [ ] Unit tests verify sphere intersection (hit, miss, tangent cases)
- [ ] All code formatted with `rustfmt` and passes `clippy`
- [ ] Public APIs have documentation comments

---

## Technical Approach

### Module Structure

Create `src/visualization/ray_tracer/` module:

```
src/visualization/ray_tracer/
├── mod.rs          # Module exports
├── math.rs         # Vector3 and Ray
├── hittable.rs     # Hittable trait and HitRecord
└── sphere.rs       # Sphere implementation
```

### Vector3 Implementation

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self { /* ... */ }
    pub fn length(&self) -> f32 { /* ... */ }
    pub fn normalize(&self) -> Vector3 { /* ... */ }
    pub fn dot(&self, other: &Vector3) -> f32 { /* ... */ }
    pub fn cross(&self, other: &Vector3) -> Vector3 { /* ... */ }
}

// Implement Add, Sub, Mul traits for ergonomic usage
```

### Ray-Sphere Intersection Algorithm

Solve quadratic equation for ray parameter `t`:
- Ray: `P(t) = origin + t * direction`
- Sphere: `|P - center|² = radius²`
- Substitute and solve: `at² + bt + c = 0`
- Return closest valid `t` in range `[t_min, t_max]`

### Epsilon Comparisons

Use epsilon for floating-point comparisons per coding standards:

```rust
const EPSILON: f32 = 1e-6;
if value.abs() < EPSILON { /* treat as zero */ }
```

---

## Dependencies

**Depends on**:
- None (foundational story)

**Blocks**:
- VIZ-010 (Camera & Rendering Pipeline) - needs Vector3, Ray, Hittable
- VIZ-011 (Lighting & Shading) - needs Vector3, HitRecord

---

## Architecture References

- **Source Tree**: `docs/architecture/source-tree.md`
  - Visualization module: Lines 218-232
  - Module organization: Lines 348-366

- **Coding Standards**: `docs/architecture/coding-standards.md`
  - Floating-point comparisons: Line 69-77
  - No unwrap without justification: Lines 30-45
  - Naming conventions: Lines 108-117
  - Testing standards: Lines 269-297

- **Tech Stack**: `docs/architecture/tech-stack.md`
  - Rust version 1.75+: Line 11
  - No external math dependencies

---

## Testing Requirements

### Unit Tests (in `#[cfg(test)]` modules)

**math.rs tests**:
- `test_vector3_length` - Verify Pythagorean theorem
- `test_vector3_normalize` - Verify normalized vector has length 1.0
- `test_vector3_dot_product` - Verify dot product formula
- `test_vector3_cross_product` - Verify orthogonality
- `test_ray_at` - Verify parametric ray equation

**sphere.rs tests**:
- `test_sphere_hit_from_outside` - Ray intersects sphere from outside
- `test_sphere_hit_from_inside` - Ray originates inside sphere
- `test_sphere_miss` - Ray misses sphere entirely
- `test_sphere_tangent` - Ray grazes sphere surface
- `test_sphere_closest_hit` - Multiple intersections, return closest

### Test Structure (AAA Pattern)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector3_normalize() {
        // Arrange
        let v = Vector3::new(3.0, 4.0, 0.0);

        // Act
        let normalized = v.normalize();

        // Assert
        let length = normalized.length();
        assert!((length - 1.0).abs() < 1e-6);
    }
}
```

---

## Notes for Dev Agent

### Implementation Order

1. **Start with Vector3** (math.rs):
   - Implement struct and basic methods
   - Add operator overloads (Add, Sub, Mul)
   - Write and verify unit tests

2. **Implement Ray** (math.rs):
   - Simple struct with origin and direction
   - Add `at(t)` method
   - Unit test parametric equation

3. **Define Hittable** (hittable.rs):
   - Create trait with `hit()` signature
   - Define HitRecord struct
   - Document front_face logic

4. **Implement Sphere** (sphere.rs):
   - Sphere struct (center, radius)
   - Ray-sphere intersection math
   - Comprehensive intersection tests

### Common Pitfalls

1. **Direction normalization**: Ray direction should be normalized
2. **t_min > 0**: Avoid shadow acne by using small positive t_min (e.g., 0.001)
3. **Front face detection**: Compare ray direction with outward normal to determine front/back
4. **Quadratic discriminant**: Check discriminant before sqrt to avoid NaN

### Performance Notes

- Use `#[inline]` on hot path functions (dot, cross, normalize)
- Vector operations are simple enough that compiler will optimize well
- No allocations needed in any of these functions

### Code Quality

Run before committing:
```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test --lib
```

---

## Completion Checklist

- [ ] Module structure created
- [ ] Vector3 fully implemented and tested
- [ ] Ray fully implemented and tested
- [ ] Hittable trait defined
- [ ] HitRecord struct defined
- [ ] Sphere implements Hittable
- [ ] All unit tests pass
- [ ] Documentation comments on all public APIs
- [ ] Code passes rustfmt and clippy
- [ ] No unwrap() without expect() + justification

---

## Change Log

| Date | Version | Description | Author |
|------|---------|-------------|--------|
| 2025-11-05 | 1.0 | Initial story creation - split from VIZ-009-MVP | Sarah (PO) |
