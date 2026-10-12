//! Perspective camera helpers.

use glam::{Mat4, Vec3};

/// Right-handed perspective camera.
#[derive(Clone, Debug)]
pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov_y_radians: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new(eye: Vec3, target: Vec3) -> Self {
        Self {
            eye,
            target,
            up: Vec3::Y,
            fov_y_radians: 60.0_f32.to_radians(),
            near: 0.1,
            far: 100.0,
        }
    }

    pub fn view(&self) -> Mat4 {
        Mat4::look_at_rh(self.eye, self.target, self.up)
    }

    pub fn projection(&self, aspect: f32) -> Mat4 {
        Mat4::perspective_rh(self.fov_y_radians, aspect, self.near, self.far)
    }

    pub fn view_projection(&self, aspect: f32) -> Mat4 {
        self.projection(aspect) * self.view()
    }

    /// Orbit around `target` on the XZ plane by `yaw` and adjust pitch-like elevation.
    pub fn orbit(&mut self, yaw: f32, pitch_delta: f32, radius_scale: f32) {
        let offset = self.eye - self.target;
        let radius = (offset.length() * radius_scale).max(0.5);
        let yaw_angle = offset.x.atan2(offset.z) + yaw;
        let mut pitch = (offset.y / radius).asin() + pitch_delta;
        pitch = pitch.clamp(-1.4, 1.4);
        self.eye = self.target
            + Vec3::new(
                radius * yaw_angle.sin() * pitch.cos(),
                radius * pitch.sin(),
                radius * yaw_angle.cos() * pitch.cos(),
            );
    }
}
