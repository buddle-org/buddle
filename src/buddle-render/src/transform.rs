use std::ops;
use buddle_math::{Mat4, Quat, Vec3};
use buddle_nif::objects::NiAVObject;

#[derive(Clone, Copy, PartialEq)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: f32,
}

impl Transform {
    pub fn from_nif(av: &NiAVObject) -> Self {
        Transform {
            translation: av.translation.clone().into(),
            rotation: Quat::from_mat3(&av.rotation.clone().into()),
            scale: av.scale,
        }
    }
}

impl ops::Mul for Transform {
    type Output = Transform;

    fn mul(self, rhs: Self) -> Self::Output {
        Transform {
            translation: self.translation + self.rotation.mul_vec3(rhs.translation * self.scale),
            rotation: self.rotation * rhs.rotation,
            scale: self.scale * rhs.scale
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
            scale: 1.0,
        }
    }
}
