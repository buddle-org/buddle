use buddle_math::{Mat4, Quat, Vec3};
use buddle_nif::compounds::Vector3;

#[derive(Clone, Copy, PartialEq)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub fn from_nif(translation: Vector3, rotation: Quat, scale: f32) -> Self {
        Transform {
            translation: translation.into(),
            rotation: rotation.into(),
            scale: Vec3::splat(scale),
        }
    }
}

impl Into<Mat4> for Transform {
    fn into(self) -> Mat4 {
        Mat4::from_translation(self.translation)
    }
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}
