use buddle_math::{Mat3, Vec2, Vec3};
use buddle_nif::compounds::{Matrix33, Quaternion};
use buddle_render::Camera;

pub struct CameraController {
    keymap: Vec<bool>,
    mouse_delta: Vec2,
}

impl CameraController {
    pub fn new() -> Self {
        CameraController {
            keymap: vec![false; 6],
            mouse_delta: Vec2::ZERO,
        }
    }

    pub fn add_mouse_delta(&mut self, delta: Vec2) {
        self.mouse_delta -= delta;
    }

    pub fn set_key_state(&mut self, key: usize, state: bool) {
        self.keymap[key] = state;
    }

    pub fn update_free(&mut self, camera: &mut Camera) {
        let forward = (camera.target - camera.position).normalize();
        let right = forward.cross(camera.up).normalize();

        let mut transform = Vec3::ZERO;

        if self.keymap[0] {
            transform += forward
        }
        if self.keymap[1] {
            transform -= forward
        }
        if self.keymap[2] {
            transform -= right
        }
        if self.keymap[3] {
            transform += right
        }
        if self.keymap[4] {
            transform += Vec3::Y
        }
        if self.keymap[5] {
            transform -= Vec3::Y
        }

        transform *= 10.0;

        camera.position += transform;
        camera.target += transform;

        let forward = (camera.target - camera.position).normalize();
        let right = forward.cross(camera.up).normalize();

        let rotation = Mat3::from_axis_angle(camera.up, self.mouse_delta.x * 0.001)
            * Mat3::from_axis_angle(right, self.mouse_delta.y * 0.001);

        camera.target = camera.position + rotation * forward;

        self.mouse_delta = Vec2::ZERO;
    }
}
