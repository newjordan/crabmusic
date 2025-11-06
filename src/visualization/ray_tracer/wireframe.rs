//! Wireframe grid detection on a sphere surface (VIZ-011)

use super::math::Vector3;

/// Returns true if the hit point lies on a wireframe grid line of the sphere.
/// `center` and `radius` describe the sphere being rendered.
pub fn is_on_wireframe(point: Vector3, center: Vector3, radius: f32) -> bool {
    let p = point - center;
    let r = radius.max(1e-6);
    let x = p.x / r; let y = p.y / r; let z = p.z / r;
    // Normalize to reduce numeric drift
    let len = (x * x + y * y + z * z).sqrt().max(1e-6);
    let x = x / len; let y = y / len; let z = z / len;

    // Spherical coordinates
    // theta: longitude in [-pi, pi]
    let theta = z.atan2(x);
    // phi: polar angle from +Y axis (0 at top, pi at bottom).
    let phi = (y).clamp(-1.0, 1.0).acos();

    // Grid every 10 degrees (~0.174533 radians). Use a small threshold
    // so the lines are thin.
    let step = 10.0_f32.to_radians();
    let tol = 0.03; // radians ~ 1.7 degree thickness

    // Distance to nearest multiple of step (in radians)
    let d_theta = {
        let m = theta.rem_euclid(step);
        m.min(step - m)
    };
    let d_phi = {
        let m = phi.rem_euclid(step);
        m.min(step - m)
    };

    d_theta < tol || d_phi < tol
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wireframe_equator_and_meridian() {
        let c = Vector3::new(0.0, 0.0, 0.0);
        let r = 1.0;
        // Equator (phi ~ pi/2) should be on a latitude line (depending on step).
        let on_equator = is_on_wireframe(Vector3::new(1.0, 0.0, 0.0), c, r);
        // Prime meridian (theta ~ 0)
        let on_meridian = is_on_wireframe(Vector3::new(0.0, 1.0, 0.0), c, r);
        assert!(on_equator || on_meridian);
    }
}

