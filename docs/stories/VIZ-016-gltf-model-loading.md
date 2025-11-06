# VIZ-016: glTF/GLB Model Loading for Ray Tracer

**Epic**: Epic 10: Braille-Based 3D Model Viewer
**Priority**: P1 (Required for 3D model support)
**Estimated Effort**: 3-4 days
**Status**: Draft

---

## Story

**As a** developer building a 3D ray tracer visualization system,
**I want** to load 3D models from glTF/GLB files and convert them into raycast-able geometry,
**so that** users can visualize arbitrary 3D models (not just hard-coded spheres) in the terminal with wireframe and solid rendering modes.

---

## Description

Implement glTF 2.0 model loading capability for the ray tracer, enabling CrabMusic to render arbitrary 3D models in the terminal. This story focuses on:

1. **Loading glTF/GLB files** using the `gltf` crate
2. **Extracting triangle mesh data** (vertices, normals, indices)
3. **Creating a Triangle primitive** that implements the `Hittable` trait
4. **Building a TriangleMesh structure** for efficient ray-triangle intersection
5. **Integrating with the existing Scene** to support both spheres and meshes

The implementation adheres to **DRY** (Don't Repeat Yourself) and **SRP** (Single Responsibility Principle):
- Each module has one clear purpose
- Geometry loading is separate from ray intersection logic
- Triangle intersection uses a well-tested algorithm (Möller-Trumbore)
- Mesh data structures are reusable across different model formats

---

## Acceptance Criteria

### Core Loading (SRP: Model I/O)
- [ ] `gltf` crate added to `Cargo.toml` with appropriate version
- [ ] New module `src/visualization/ray_tracer/gltf_loader.rs` created
- [ ] `load_gltf()` function loads .gltf and .glb files
- [ ] Function extracts mesh primitives from the first mesh in the file
- [ ] Function returns structured mesh data (vertices, normals, indices)
- [ ] Error handling for missing files, invalid formats, missing data

### Triangle Primitive (SRP: Ray-Triangle Intersection)
- [ ] New module `src/visualization/ray_tracer/triangle.rs` created
- [ ] `Triangle` struct with three vertices (v0, v1, v2) and optional normals (n0, n1, n2)
- [ ] `Triangle` implements `Hittable` trait
- [ ] Ray-triangle intersection uses Möller-Trumbore algorithm
- [ ] Barycentric coordinates calculated for normal interpolation
- [ ] Unit tests verify intersection at various angles and positions
- [ ] Unit tests verify miss cases (ray parallel, behind, outside)

### Triangle Mesh (SRP: Mesh Management)
- [ ] New module `src/visualization/ray_tracer/mesh.rs` created
- [ ] `TriangleMesh` struct holds Vec of triangles
- [ ] `TriangleMesh` implements `Hittable` trait (tests all triangles)
- [ ] `TriangleMesh::from_gltf_data()` constructor builds mesh from loaded data
- [ ] Smooth shading support via interpolated vertex normals
- [ ] Flat shading fallback if normals not provided

### Scene Integration (SRP: Scene Composition)
- [ ] `Scene::add_object()` works with any `Box<dyn Hittable>` (already exists)
- [ ] New `Scene::new_with_model()` constructor loads a glTF file
- [ ] Scene can contain mix of spheres and meshes simultaneously
- [ ] Example program `examples/gltf_viewer.rs` demonstrates loading and rendering

### Testing & Validation
- [ ] Unit tests for Triangle intersection (hit, miss, edge cases)
- [ ] Unit tests for TriangleMesh construction from raw data
- [ ] Integration test loads a simple glTF file (e.g., cube or pyramid)
- [ ] Integration test renders loaded model to intensity buffer
- [ ] Visual test: render a glTF model in both wireframe and solid modes
- [ ] All tests pass with `cargo test`
- [ ] Code passes `rustfmt` and `clippy`

---

## Technical Approach

### Architecture: Separation of Concerns

```
┌─────────────────────────────────────────────────────────────┐
│                         Scene                                │
│  (Owns all Hittable objects, orchestrates rendering)        │
└────────────────┬────────────────────────────────────────────┘
                 │
        ┌────────┴────────┐
        │                 │
┌───────▼──────┐  ┌──────▼────────┐
│   Sphere     │  │ TriangleMesh  │  (Both implement Hittable)
│ (VIZ-009)    │  │   (VIZ-016)   │
└──────────────┘  └───────┬───────┘
                          │
                  ┌───────▼────────┐
                  │   Triangle     │  (Implements Hittable)
                  │  (VIZ-016)     │
                  └────────────────┘
                          ▲
                          │
                  ┌───────┴────────┐
                  │  gltf_loader   │  (Loads file, extracts data)
                  │  (VIZ-016)     │
                  └────────────────┘
```

### Module Structure

```
src/visualization/ray_tracer/
├── mod.rs              # Module exports (add triangle, mesh, gltf_loader)
├── math.rs             # (existing)
├── hittable.rs         # (existing)
├── sphere.rs           # (existing)
├── triangle.rs         # 🆕 Single triangle primitive
├── mesh.rs             # 🆕 Triangle mesh container
├── gltf_loader.rs      # 🆕 glTF file loading
├── camera.rs           # (existing)
├── scene.rs            # (existing - enhance with model loading)
├── renderer.rs         # (existing - no changes needed)
├── wireframe.rs        # (existing)
├── lighting.rs         # (existing)
└── braille.rs          # (existing)
```

---

## Implementation Details

### 1. Cargo Dependency

Add to `Cargo.toml`:
```toml
[dependencies]
gltf = "1.4"  # glTF 2.0 parser
```

### 2. glTF Loader (SRP: File I/O and Data Extraction)

**File**: `src/visualization/ray_tracer/gltf_loader.rs`

```rust
use super::math::Vector3;
use anyhow::{Context, Result};

/// Raw mesh data extracted from glTF
pub struct MeshData {
    pub positions: Vec<Vector3>,  // Vertex positions
    pub normals: Option<Vec<Vector3>>,  // Vertex normals (if present)
    pub indices: Vec<u32>,  // Triangle indices (groups of 3)
}

/// Load mesh data from a glTF or GLB file
pub fn load_gltf(path: &str) -> Result<MeshData> {
    let (document, buffers, _images) = gltf::import(path)
        .with_context(|| format!("Failed to load glTF file: {}", path))?;

    // Get the first mesh (simplification for MVP)
    let mesh = document.meshes().next()
        .context("No meshes found in glTF file")?;

    // Get the first primitive
    let primitive = mesh.primitives().next()
        .context("No primitives found in mesh")?;

    // Extract positions (required)
    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
    
    let positions: Vec<Vector3> = reader
        .read_positions()
        .context("No position data in primitive")?
        .map(|p| Vector3::new(p[0], p[1], p[2]))
        .collect();

    // Extract normals (optional)
    let normals = reader.read_normals().map(|iter| {
        iter.map(|n| Vector3::new(n[0], n[1], n[2])).collect()
    });

    // Extract indices (required for indexed geometry)
    let indices: Vec<u32> = reader
        .read_indices()
        .context("No index data in primitive")?
        .into_u32()
        .collect();

    Ok(MeshData {
        positions,
        normals,
        indices,
    })
}
```

**Key Design Decisions**:
- Returns structured data, not a mesh object (SRP: loading ≠ geometry)
- Uses `anyhow::Result` for rich error context
- Handles optional normals gracefully
- Focuses on first mesh/primitive (can extend later)

### 3. Triangle Primitive (SRP: Ray-Triangle Intersection)

**File**: `src/visualization/ray_tracer/triangle.rs`

```rust
use super::hittable::{HitRecord, Hittable};
use super::math::{Ray, Vector3};

/// A single triangle defined by three vertices
#[derive(Debug, Clone, Copy)]
pub struct Triangle {
    pub v0: Vector3,
    pub v1: Vector3,
    pub v2: Vector3,
    // Optional per-vertex normals for smooth shading
    pub n0: Option<Vector3>,
    pub n1: Option<Vector3>,
    pub n2: Option<Vector3>,
}

impl Triangle {
    pub fn new(v0: Vector3, v1: Vector3, v2: Vector3) -> Self {
        Self { v0, v1, v2, n0: None, n1: None, n2: None }
    }

    pub fn with_normals(
        v0: Vector3, v1: Vector3, v2: Vector3,
        n0: Vector3, n1: Vector3, n2: Vector3,
    ) -> Self {
        Self {
            v0, v1, v2,
            n0: Some(n0),
            n1: Some(n1),
            n2: Some(n2),
        }
    }

    /// Calculate geometric normal (flat shading)
    fn geometric_normal(&self) -> Vector3 {
        let edge1 = self.v1 - self.v0;
        let edge2 = self.v2 - self.v0;
        edge1.cross(&edge2).normalize()
    }
}

impl Hittable for Triangle {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        // Möller-Trumbore ray-triangle intersection algorithm
        const EPSILON: f32 = 1e-6;

        let edge1 = self.v1 - self.v0;
        let edge2 = self.v2 - self.v0;
        let h = ray.direction.cross(&edge2);
        let a = edge1.dot(&h);

        // Ray parallel to triangle
        if a.abs() < EPSILON {
            return None;
        }

        let f = 1.0 / a;
        let s = ray.origin - self.v0;
        let u = f * s.dot(&h);

        // Intersection outside triangle (barycentric u)
        if u < 0.0 || u > 1.0 {
            return None;
        }

        let q = s.cross(&edge1);
        let v = f * ray.direction.dot(&q);

        // Intersection outside triangle (barycentric v)
        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        // Calculate t (distance along ray)
        let t = f * edge2.dot(&q);

        if t < t_min || t > t_max {
            return None;
        }

        let point = ray.at(t);

        // Interpolate normal if vertex normals provided, else use geometric normal
        let normal = if let (Some(n0), Some(n1), Some(n2)) = (self.n0, self.n1, self.n2) {
            // Smooth shading: interpolate using barycentric coordinates
            let w = 1.0 - u - v;
            (n0 * w + n1 * u + n2 * v).normalize()
        } else {
            // Flat shading: use geometric normal
            self.geometric_normal()
        };

        Some(HitRecord::new(point, normal, t, ray))
    }
}
```

**Key Design Decisions**:
- Möller-Trumbore algorithm: industry-standard, efficient, well-tested
- Barycentric coordinates (u, v, w) enable smooth normal interpolation
- Graceful fallback to flat shading if normals missing
- EPSILON constant prevents floating-point precision issues

### 4. Triangle Mesh (SRP: Mesh Container)

**File**: `src/visualization/ray_tracer/mesh.rs`

```rust
use super::gltf_loader::MeshData;
use super::hittable::{HitRecord, Hittable};
use super::math::{Ray, Vector3};
use super::triangle::Triangle;

/// A collection of triangles forming a mesh
pub struct TriangleMesh {
    triangles: Vec<Triangle>,
}

impl TriangleMesh {
    /// Create mesh from glTF-loaded data
    pub fn from_gltf_data(data: MeshData) -> Self {
        let mut triangles = Vec::new();

        // Build triangles from indexed vertex data
        for chunk in data.indices.chunks(3) {
            if chunk.len() != 3 {
                continue; // Skip incomplete triangles
            }

            let i0 = chunk[0] as usize;
            let i1 = chunk[1] as usize;
            let i2 = chunk[2] as usize;

            let v0 = data.positions[i0];
            let v1 = data.positions[i1];
            let v2 = data.positions[i2];

            let triangle = if let Some(ref normals) = data.normals {
                Triangle::with_normals(v0, v1, v2, normals[i0], normals[i1], normals[i2])
            } else {
                Triangle::new(v0, v1, v2)
            };

            triangles.push(triangle);
        }

        Self { triangles }
    }

    pub fn triangle_count(&self) -> usize {
        self.triangles.len()
    }
}

impl Hittable for TriangleMesh {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut closest_so_far = t_max;
        let mut best_hit: Option<HitRecord> = None;

        // Test ray against all triangles, keep closest hit
        for triangle in &self.triangles {
            if let Some(hit) = triangle.hit(ray, t_min, closest_so_far) {
                closest_so_far = hit.t;
                best_hit = Some(hit);
            }
        }

        best_hit
    }
}
```

**Key Design Decisions**:
- Mesh is just a container of triangles (DRY: reuses Triangle logic)
- Linear search through triangles (can optimize with BVH later)
- Implements `Hittable` so it works seamlessly with Scene
- Constructor handles both smooth and flat shading

### 5. Scene Enhancement

**File**: `src/visualization/ray_tracer/scene.rs` (add method)

```rust
use super::gltf_loader::load_gltf;
use super::mesh::TriangleMesh;

impl Scene {
    /// Create scene with a glTF model
    pub fn new_with_model(path: &str) -> anyhow::Result<Self> {
        let mesh_data = load_gltf(path)?;
        let mesh = Box::new(TriangleMesh::from_gltf_data(mesh_data));
        
        let light = Light::new(Vector3::new(2.0, 2.0, 2.0), 1.0);
        
        Ok(Self {
            objects: vec![mesh],
            lights: vec![light],
        })
    }
}
```

---

## Dependencies

**Depends on**:
- VIZ-009 (Ray Tracing Primitives) - needs Vector3, Ray, Hittable trait
- VIZ-010 (Camera & Rendering) - uses existing renderer
- VIZ-011 (Lighting & Shading) - lighting works with any Hittable

**Blocks**:
- Future stories for advanced model features (animations, materials, textures)

---

## Testing Requirements

### Unit Tests

**triangle.rs tests**:
```rust
#[test]
fn test_triangle_hit_center() {
    let tri = Triangle::new(
        Vector3::new(-1.0, 0.0, -3.0),
        Vector3::new(1.0, 0.0, -3.0),
        Vector3::new(0.0, 1.0, -3.0),
    );
    let ray = Ray::new(Vector3::new(0.0, 0.3, 0.0), Vector3::new(0.0, 0.0, -1.0));
    
    let hit = tri.hit(&ray, 0.001, f32::MAX);
    assert!(hit.is_some());
}

#[test]
fn test_triangle_miss_outside() {
    let tri = Triangle::new(
        Vector3::new(-1.0, 0.0, -3.0),
        Vector3::new(1.0, 0.0, -3.0),
        Vector3::new(0.0, 1.0, -3.0),
    );
    let ray = Ray::new(Vector3::new(5.0, 5.0, 0.0), Vector3::new(0.0, 0.0, -1.0));
    
    let hit = tri.hit(&ray, 0.001, f32::MAX);
    assert!(hit.is_none());
}
```

---

## Notes for Dev Agent

### Implementation Order

1. Add `gltf` dependency to Cargo.toml
2. Create `gltf_loader.rs` with `load_gltf()` function
3. Create `triangle.rs` with Möller-Trumbore intersection
4. Write unit tests for Triangle (hit/miss cases)
5. Create `mesh.rs` with TriangleMesh container
6. Add `Scene::new_with_model()` constructor
7. Create `examples/gltf_viewer.rs` demo
8. Test with simple glTF models (cube, pyramid)

### Testing glTF Files

Use simple test models:
- **Cube**: 12 triangles, easy to verify
- **Pyramid**: 6 triangles, simple geometry
- Download from: https://github.com/KhronosGroup/glTF-Sample-Models

### Performance Considerations

- Linear triangle search is O(n) - acceptable for small models (<10k triangles)
- Future optimization: Bounding Volume Hierarchy (BVH) for large models
- Möller-Trumbore is already highly optimized

### Common Pitfalls

1. **Index out of bounds**: Validate indices against positions.len()
2. **Winding order**: glTF uses counter-clockwise by default
3. **Coordinate system**: glTF uses right-handed Y-up (same as our ray tracer)
4. **Missing normals**: Always provide flat shading fallback

---

## Completion Checklist

- [ ] `gltf` crate added to Cargo.toml
- [ ] `gltf_loader.rs` loads and extracts mesh data
- [ ] `triangle.rs` implements ray-triangle intersection
- [ ] Triangle unit tests pass (hit, miss, edge cases)
- [ ] `mesh.rs` builds TriangleMesh from MeshData
- [ ] Scene can load glTF models
- [ ] Example program renders a glTF model
- [ ] Integration test verifies model loading and rendering
- [ ] Code passes rustfmt and clippy
- [ ] Documentation comments complete

---

## Change Log

| Date | Version | Description | Author |
|------|---------|-------------|--------|
| 2025-11-06 | 1.0 | Initial story creation for glTF model loading | AI Agent |

