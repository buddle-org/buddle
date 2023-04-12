use super::*;

impl From<&TexCoord> for buddle_math::Vec2 {
    fn from(value: &TexCoord) -> Self {
        buddle_math::Vec2::new(value.u, value.v)
    }
}

impl From<TexCoord> for buddle_math::Vec2 {
    fn from(value: TexCoord) -> Self {
        (&value).into()
    }
}

impl From<&Vector3> for buddle_math::Vec3 {
    fn from(value: &Vector3) -> Self {
        buddle_math::Vec3::new(value.x, value.y, value.z)
    }
}

impl From<Vector3> for buddle_math::Vec3 {
    fn from(value: Vector3) -> Self {
        (&value).into()
    }
}

impl From<&Vector4> for buddle_math::Vec4 {
    fn from(value: &Vector4) -> Self {
        buddle_math::Vec4::new(value.x, value.y, value.z, value.w)
    }
}

impl From<Vector4> for buddle_math::Vec4 {
    fn from(value: Vector4) -> Self {
        (&value).into()
    }
}

impl From<&Quaternion> for buddle_math::Quat {
    fn from(value: &Quaternion) -> Self {
        buddle_math::Quat::from_xyzw(value.x, value.y, value.z, value.w)
    }
}

impl From<Quaternion> for buddle_math::Quat {
    fn from(value: Quaternion) -> Self {
        (&value).into()
    }
}

impl From<&Matrix33> for buddle_math::Mat3 {
    fn from(value: &Matrix33) -> Self {
        buddle_math::Mat3::from_cols_array(&[
            value.m11, value.m21, value.m31, value.m12, value.m22, value.m32, value.m13, value.m23,
            value.m33,
        ])
        .transpose()
    }
}

impl From<Matrix33> for buddle_math::Mat3 {
    fn from(value: Matrix33) -> Self {
        (&value).into()
    }
}
