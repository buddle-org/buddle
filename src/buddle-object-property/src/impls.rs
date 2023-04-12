use std::{
    any::{Any, TypeId},
    collections::VecDeque,
    sync::Arc,
};

use anyhow::{anyhow, bail};
use buddle_math::*;
use buddle_utils::{bitint::*, color::Color, hash::StringIdBuilder};

use crate::{
    cpp::*, serde::*, type_info::*, Container, ContainerIter, PropertyClass, PropertyClassExt,
    Type, TypeMut, TypeOwned, TypeRef,
};

macro_rules! impl_leaf_info_for {
    ($ty:ty) => {
        // SAFETY: User is responsible for meeting invariants.
        unsafe impl Reflected for $ty {
            const TYPE_NAME: &'static str = $crate::__private::type_name::<Self>();

            const TYPE_INFO: &'static TypeInfo = &TypeInfo::leaf::<$ty>(None);
        }
    };

    ($ty:ty, $name:expr) => {
        // SAFETY: User is responsible for meeting invariants.
        unsafe impl $crate::type_info::Reflected for $ty {
            const TYPE_NAME: &'static str = $name;

            const TYPE_INFO: &'static TypeInfo = &TypeInfo::leaf::<$ty>(Some($name));
        }
    };
}

/// Provides blanket implementations for most [`Type`] trait methods,
/// except serialization.
#[macro_export]
macro_rules! impl_type_methods {
    ($kind:ident) => {
        #[inline]
        fn as_any(&self) -> &dyn ::std::any::Any {
            self
        }

        #[inline]
        fn as_any_mut(&mut self) -> &mut dyn ::std::any::Any {
            self
        }

        #[inline]
        fn type_ref(&self) -> $crate::TypeRef<'_> {
            $crate::TypeRef::$kind(self)
        }

        #[inline]
        fn type_mut(&mut self) -> $crate::TypeMut<'_> {
            $crate::TypeMut::$kind(self)
        }

        #[inline]
        fn type_owned(self: ::std::boxed::Box<Self>) -> $crate::TypeOwned {
            $crate::TypeOwned::$kind(self)
        }

        #[inline]
        fn set(
            &mut self,
            value: ::std::boxed::Box<dyn $crate::Type>,
        ) -> ::std::result::Result<(), ::std::boxed::Box<dyn $crate::Type>> {
            *self = *value.downcast()?;
            ::std::result::Result::Ok(())
        }
    };
}

macro_rules! impl_primitive {
    ($ty:ident, $name:expr) => {
        impl_leaf_info_for!($ty, $name);
        impl Type for $ty {
            impl_type_methods!(Value);

            fn serialize(&mut self, ser: &mut Serializer<'_>) {
                ser.writer().$ty(*self);
            }

            fn deserialize(&mut self, de: &mut Deserializer<'_>) -> anyhow::Result<()> {
                *self = de.reader().$ty()?;
                Ok(())
            }
        }
    };
}

impl_primitive!(bool, "bool");

impl_primitive!(i8, "char");
impl_primitive!(i16, "short");
impl_primitive!(i32, "int");

impl_primitive!(u8, "unsigned char");
impl_primitive!(u16, "unsigned short");
impl_primitive!(u32, "unsigned int");
impl_primitive!(u64, "unsigned __int64");

impl_primitive!(f32, "float");
impl_primitive!(f64, "double");

impl_leaf_info_for!(RawString, "std::string");
impl Type for RawString {
    impl_type_methods!(Value);

    fn serialize(&mut self, ser: &mut Serializer<'_>) {
        ser.write_str(&self.0);
    }

    fn deserialize(&mut self, de: &mut Deserializer<'_>) -> anyhow::Result<()> {
        *self = de.read_str().map(Self)?;
        Ok(())
    }
}

impl_leaf_info_for!(RawWideString, "std::wstring");
impl Type for RawWideString {
    impl_type_methods!(Value);

    fn serialize(&mut self, ser: &mut Serializer<'_>) {
        ser.write_wstr(&self.0);
    }

    fn deserialize(&mut self, de: &mut Deserializer<'_>) -> anyhow::Result<()> {
        *self = de.read_wstr().map(Self)?;
        Ok(())
    }
}

macro_rules! impl_container {
    ($ty:path, $deref:ty, $push:ident, $pop:ident) => {
        unsafe impl<T: Default + Reflected + Type> Reflected for $ty {
            const TYPE_NAME: &'static str = T::TYPE_NAME;

            const TYPE_INFO: &'static TypeInfo = &TypeInfo::Leaf(ValueInfo {
                type_name: Self::TYPE_NAME,
                type_hash: T::TYPE_INFO.type_hash(),
                type_id: TypeId::of::<$ty>(),
            });
        }

        impl<T: Default + Reflected + Type> Type for $ty {
            impl_type_methods!(Container);

            fn serialize(&mut self, ser: &mut Serializer<'_>) {
                for value in self {
                    value.serialize(ser);
                }
            }

            fn deserialize(&mut self, de: &mut Deserializer<'_>) -> anyhow::Result<()> {
                let len = de.deserialize_container_len()?;

                self.clear();
                self.reserve(len);

                (0..len).try_for_each(|_| {
                    let mut new = T::default();

                    new.deserialize(de)?;
                    <$ty>::$push(self, new);

                    Ok(())
                })
            }
        }

        impl<T: Default + Reflected + Type> Container for $ty {
            fn get(&self, idx: usize) -> Option<&dyn Type> {
                <$deref>::get(self, idx).map(|e| e as &dyn Type)
            }

            fn get_mut(&mut self, idx: usize) -> Option<&mut dyn Type> {
                <$deref>::get_mut(self, idx).map(|e| e as &mut dyn Type)
            }

            fn push(&mut self, value: Box<dyn Type>) {
                <$ty>::$push(
                    self,
                    *value.downcast().unwrap_or_else(|v| {
                        panic!(
                            "Attempted to push invalid value of type {}!",
                            v.type_info().type_name()
                        )
                    }),
                )
            }

            fn pop(&mut self) -> Option<Box<dyn Type>> {
                <$ty>::$pop(self).map(|e| Box::new(e) as Box<dyn Type>)
            }

            fn reserve(&mut self, capacity: usize) {
                <$ty>::reserve(self, capacity);
            }

            fn len(&self) -> usize {
                <$ty>::len(self)
            }

            fn iter(&self) -> ContainerIter<'_> {
                ContainerIter::new(self)
            }
        }
    };
}

impl_container!(Vec<T>, [T], push, pop);
impl_container!(VecDeque<T>, Self, push_back, pop_back);

macro_rules! impl_simple {
    (@non_generic $ty:ident, $name:expr, $($idents:ident),* $(,)?) => {
        impl_leaf_info_for!($ty, $name);
        impl Type for $ty {
            impl_type_methods!(Value);

            fn serialize(&mut self, ser: &mut Serializer<'_>) {
                $(
                    self.$idents.serialize(ser);
                )*
            }

            fn deserialize(&mut self, de: &mut Deserializer<'_>) -> anyhow::Result<()> {
                $(
                    self.$idents.deserialize(de)?;
                )*
                Ok(())
            }
        }
    };

    (@generic $ty:path, $name:expr, $($idents:ident),* $(,)?) => {
        unsafe impl<T: Reflected + Type> Reflected for $ty {
            const TYPE_NAME: &'static str = $crate::__private::type_name::<Self>();

            const TYPE_INFO: &'static TypeInfo = &TypeInfo::Leaf(ValueInfo {
                type_name: Self::TYPE_NAME,
                type_hash: StringIdBuilder::new()
                    .feed_str($name)
                    .feed_str("<")
                    .feed_str(T::TYPE_NAME)
                    .feed_str(">")
                    .finish(),
                type_id: TypeId::of::<Self>(),
            });
        }

        impl<T: Reflected + Type> Type for $ty {
            impl_type_methods!(Value);

            fn serialize(&mut self, ser: &mut Serializer<'_>) {
                $(
                    self.$idents.serialize(ser);
                )*
            }

            fn deserialize(&mut self, de: &mut Deserializer<'_>) -> anyhow::Result<()> {
                $(
                    self.$idents.deserialize(de)?;
                )*
                Ok(())
            }
        }
    };
}

impl_simple!(@non_generic Vec3, "class Vector3D", x, y, z);
impl_simple!(@non_generic Vec3A, "class Vector3D", x, y, z);

impl_simple!(@non_generic Mat3, "class Matrix3x3", x_axis, y_axis, z_axis);
impl_simple!(@non_generic Mat3A, "class Matrix3x3", x_axis, y_axis, z_axis);

impl_simple!(@non_generic Quat, "class Quaternion", x, y, z, w);

impl_simple!(@generic Point<T>, "class Point", x, y);
impl_simple!(@generic Size<T>, "class Size", width, height);
impl_simple!(@generic Rect<T>, "class Rect", left, top, right, bottom);
impl_simple!(@non_generic Euler, "class Euler", pitch, yaw, roll); // TODO: Is this order correct?

impl_simple!(@non_generic Color, "class Color", b, g, r, a);

macro_rules! impl_bit_uint {
    ($ty:ident, $name:expr, $raw:ident, $bits:expr) => {
        impl_leaf_info_for!($ty, $name);
        impl Type for $ty {
            impl_type_methods!(Value);

            fn serialize(&mut self, ser: &mut Serializer<'_>) {
                ser.writer().write_bitint(<$raw>::from(*self), $bits);
            }

            fn deserialize(&mut self, de: &mut Deserializer<'_>) -> anyhow::Result<()> {
                let value: $raw = de.reader().read_bitint($bits)?;
                *self = Self::new(value);

                Ok(())
            }
        }
    };
}

macro_rules! impl_bit_int {
    ($ty:ident, $name:expr, $uraw:ident, $raw:ident, $bits:expr) => {
        impl_leaf_info_for!($ty, $name);
        impl Type for $ty {
            impl_type_methods!(Value);

            fn serialize(&mut self, ser: &mut Serializer<'_>) {
                ser.writer().write_bitint(<$raw>::from(*self), $bits);
            }

            fn deserialize(&mut self, de: &mut Deserializer<'_>) -> anyhow::Result<()> {
                let value: $uraw = de.reader().read_bitint($bits)?;
                *self = Self::new(sign_extend!($raw, value, $bits));

                Ok(())
            }
        }
    };
}

impl_leaf_info_for!(u1, "bui1");
impl Type for u1 {
    impl_type_methods!(Value);

    fn serialize(&mut self, ser: &mut Serializer<'_>) {
        const ZERO: u1 = u1::new(0);
        ser.writer().bool(*self != ZERO);
    }

    fn deserialize(&mut self, de: &mut Deserializer<'_>) -> anyhow::Result<()> {
        let value = de.reader().read_bit()?;
        *self = Self::new(value as u8);

        Ok(())
    }
}

impl_bit_uint!(u2, "bui2", u8, 2);
impl_bit_uint!(u3, "bui3", u8, 3);
impl_bit_uint!(u4, "bui4", u8, 4);
impl_bit_uint!(u5, "bui5", u8, 5);
impl_bit_uint!(u6, "bui6", u8, 6);
impl_bit_uint!(u7, "bui7", u8, 7);
impl_bit_uint!(u24, "u24", u32, 24);

impl_leaf_info_for!(i1, "bi1");
impl Type for i1 {
    impl_type_methods!(Value);

    fn serialize(&mut self, ser: &mut Serializer<'_>) {
        const ZERO: i1 = i1::new(0);
        ser.writer().bool(*self != ZERO);
    }

    fn deserialize(&mut self, de: &mut Deserializer<'_>) -> anyhow::Result<()> {
        let value = de.reader().read_bit()?;
        *self = Self::new(sign_extend!(i8, value as u8, 1));

        Ok(())
    }
}

impl_bit_int!(i2, "bi2", u8, i8, 2);
impl_bit_int!(i3, "bi3", u8, i8, 3);
impl_bit_int!(i4, "bi4", u8, i8, 4);
impl_bit_int!(i5, "bi5", u8, i8, 5);
impl_bit_int!(i6, "bi6", u8, i8, 6);
impl_bit_int!(i7, "bi7", u8, i8, 7);
impl_bit_int!(i24, "s24", u32, i32, 24);

unsafe impl<T: Reflected + PropertyClass> Reflected for Ptr<T> {
    const TYPE_NAME: &'static str = T::TYPE_NAME;

    const TYPE_INFO: &'static TypeInfo = &TypeInfo::Leaf(ValueInfo {
        type_name: Self::TYPE_NAME,
        type_hash: StringIdBuilder::new()
            .feed_str(Self::TYPE_NAME)
            .feed_str("*")
            .finish(),
        type_id: TypeId::of::<Self>(),
    });
}

impl<T: Reflected + PropertyClass> Type for Ptr<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn type_ref(&self) -> TypeRef<'_> {
        TypeRef::Value(self)
    }

    fn type_mut(&mut self) -> TypeMut<'_> {
        TypeMut::Value(self)
    }

    fn type_owned(self: ::std::boxed::Box<Self>) -> TypeOwned {
        TypeOwned::Value(self)
    }

    fn set(&mut self, value: Box<dyn Type>) -> Result<(), Box<dyn Type>> {
        match value.downcast::<Self>() {
            // If `value` is another pointer, we can safely override it.
            Ok(value) => {
                *self = *value;
                Ok(())
            }

            // Otherwise, check if `value` is a class type derived from `T`.
            Err(value) => match value.type_owned() {
                TypeOwned::Class(c) if c.base_as::<T>().is_some() => {
                    self.value = Some(c);
                    Ok(())
                }

                v => Err(v.into_type()),
            },
        }
    }

    fn serialize(&mut self, ser: &mut Serializer<'_>) {
        ser.try_serialize(self.raw_mut());
    }

    fn deserialize(&mut self, de: &mut Deserializer<'_>) -> anyhow::Result<()> {
        *self = match de.try_deserialize()? {
            Some(v) => Self::try_new(v).map_err(|v| {
                anyhow!("received incompatible type: {}", v.type_info().type_name())
            })?,

            None => Self::null(),
        };

        Ok(())
    }
}

unsafe impl<T: Reflected + PropertyClass> Reflected for SharedPtr<T> {
    const TYPE_NAME: &'static str = T::TYPE_NAME;

    const TYPE_INFO: &'static TypeInfo = &TypeInfo::Leaf(ValueInfo {
        type_name: Self::TYPE_NAME,
        type_hash: StringIdBuilder::new()
            .feed_str("class SharedPointer<")
            .feed_str(Self::TYPE_NAME)
            .feed_str(">")
            .finish(),
        type_id: TypeId::of::<Self>(),
    });
}

impl<T: Reflected + PropertyClass> Type for SharedPtr<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn type_ref(&self) -> TypeRef<'_> {
        TypeRef::Value(self)
    }

    fn type_mut(&mut self) -> TypeMut<'_> {
        TypeMut::Value(self)
    }

    fn type_owned(self: ::std::boxed::Box<Self>) -> TypeOwned {
        TypeOwned::Value(self)
    }

    fn set(&mut self, value: Box<dyn Type>) -> Result<(), Box<dyn Type>> {
        match value.downcast::<Self>() {
            // If `value` is another pointer, we can safely override it.
            Ok(value) => {
                *self = *value;
                Ok(())
            }

            // Otherwise, check if `value` is a class type derived from `T`.
            Err(value) => match value.type_owned() {
                TypeOwned::Class(c) if c.base_as::<T>().is_some() => {
                    self.value = Arc::from(c);
                    Ok(())
                }

                v => Err(v.into_type()),
            },
        }
    }

    fn serialize(&mut self, ser: &mut Serializer<'_>) {
        ser.try_serialize(self.raw_mut());
    }

    fn deserialize(&mut self, de: &mut Deserializer<'_>) -> anyhow::Result<()> {
        let value = de.deserialize()?;
        *self = Self::try_new(Arc::from(value))
            .map_err(|v| anyhow!("received incompatible type: {}", v.type_info().type_name()))?;

        Ok(())
    }
}

unsafe impl<T: Reflected + PropertyClass> Reflected for WeakPtr<T> {
    const TYPE_NAME: &'static str = T::TYPE_NAME;

    const TYPE_INFO: &'static TypeInfo = &TypeInfo::Leaf(ValueInfo {
        type_name: Self::TYPE_NAME,
        type_hash: StringIdBuilder::new()
            .feed_str("class WeakPointer<")
            .feed_str(Self::TYPE_NAME)
            .feed_str(">")
            .finish(),
        type_id: TypeId::of::<Self>(),
    });
}

impl<T: Reflected + PropertyClass> Type for WeakPtr<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn type_ref(&self) -> TypeRef<'_> {
        TypeRef::Value(self)
    }

    fn type_mut(&mut self) -> TypeMut<'_> {
        TypeMut::Value(self)
    }

    fn type_owned(self: ::std::boxed::Box<Self>) -> TypeOwned {
        TypeOwned::Value(self)
    }

    fn set(&mut self, value: Box<dyn Type>) -> Result<(), Box<dyn Type>> {
        *self = *value.downcast()?;
        Ok(())
    }

    fn serialize(&mut self, _: &mut Serializer<'_>) {
        // Serialization is unsupported, so we do nothing.
    }

    fn deserialize(&mut self, _: &mut Deserializer<'_>) -> anyhow::Result<()> {
        bail!("Deserialization of weak pointers is not supported");
    }
}
