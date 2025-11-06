//! Reusable 3D transform controls (yaw/pitch/roll + scale) for visualizers

use std::f32::consts::PI;

#[derive(Debug, Clone, Copy)]
pub struct Transform3DControls {
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
    pub scale: f32,
    pub auto_rotate: bool,
    pub yaw_speed: f32, // radians per second
}

impl Default for Transform3DControls {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            roll: 0.0,
            scale: 1.0,
            auto_rotate: true,
            yaw_speed: 0.6,
        }
    }
}

impl Transform3DControls {
    #[inline]
    pub fn update(&mut self, dt: f32) {
        if self.auto_rotate {
            self.yaw = (self.yaw + self.yaw_speed * dt) % (2.0 * PI);
        }
    }

    #[inline]
    pub fn set_auto_rotate(&mut self, enable: bool) { self.auto_rotate = enable; }

    #[inline]
    pub fn yaw_left(&mut self, step: f32) { self.yaw -= step; self.wrap_angles(); }
    #[inline]
    pub fn yaw_right(&mut self, step: f32) { self.yaw += step; self.wrap_angles(); }
    #[inline]
    pub fn pitch_up(&mut self, step: f32) { self.pitch += step; self.wrap_angles(); }
    #[inline]
    pub fn pitch_down(&mut self, step: f32) { self.pitch -= step; self.wrap_angles(); }
    #[inline]
    pub fn roll_ccw(&mut self, step: f32) { self.roll += step; self.wrap_angles(); }
    #[inline]
    pub fn roll_cw(&mut self, step: f32) { self.roll -= step; self.wrap_angles(); }

    #[inline]
    pub fn zoom_in(&mut self) { self.scale = (self.scale * 1.2).min(20.0); }
    #[inline]
    pub fn zoom_out(&mut self) { self.scale = (self.scale / 1.2).max(0.05); }

    #[inline]
    pub fn set_scale(&mut self, s: f32) { self.scale = s.clamp(0.05, 20.0); }

    #[inline]
    pub fn reset_orientation(&mut self) { self.yaw = 0.0; self.pitch = 0.0; self.roll = 0.0; }

    #[inline]
    fn wrap_angles(&mut self) {
        // Keep angles in [-pi, pi] for numerical stability
        let wrap = |a: f32| -> f32 {
            let mut t = a;
            while t > PI { t -= 2.0 * PI; }
            while t < -PI { t += 2.0 * PI; }
            t
        };
        self.yaw = wrap(self.yaw);
        self.pitch = wrap(self.pitch);
        self.roll = wrap(self.roll);
    }
}

