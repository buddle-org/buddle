use std::any::Any;

use crate::{
    type_info::{Reflected, TypeInfo, ValueInfo},
    Type, TypeMut, TypeRef,
};

macro_rules! value_info_for {
    ($ty:ty) => {
        unsafe impl StaticReflected for $ty {
            const TYPE_INFO: &'static TypeInfo = &TypeInfo::Leaf(ValueInfo::new::<$ty>(None));
        }
    };

    ($ty:ty, $name:expr) => {
        unsafe impl Reflected for $ty {
            const TYPE_INFO: &'static TypeInfo =
                &TypeInfo::Leaf(ValueInfo::new::<$ty>(Some($name)));
        }
    };
}

macro_rules! impl_type_for {
    ($ty:ty, $name:expr) => {
        value_info_for!($ty, $name);
        impl Type for $ty {
            #[inline]
            fn as_any(&self) -> &dyn Any {
                self
            }

            #[inline]
            fn as_any_mut(&mut self) -> &mut dyn Any {
                self
            }

            #[inline]
            fn as_type(&self) -> &dyn Type {
                self
            }

            #[inline]
            fn as_type_mut(&mut self) -> &mut dyn Type {
                self
            }

            #[inline]
            fn type_ref(&self) -> TypeRef<'_> {
                TypeRef::Value(self)
            }

            #[inline]
            fn type_mut(&mut self) -> TypeMut<'_> {
                TypeMut::Value(self)
            }
        }
    };
}

impl_type_for!(i8, "char");
impl_type_for!(i16, "short");
impl_type_for!(i32, "int");
// XXX: `i64` is never used for anything.

impl_type_for!(u8, "unsigned char");
impl_type_for!(u16, "unsigned short");
impl_type_for!(u32, "unsigned int");
impl_type_for!(u64, "unsigned __int64");

impl_type_for!(String, "std::string");
