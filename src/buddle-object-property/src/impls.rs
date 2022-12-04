use std::{any::TypeId, collections::VecDeque, sync::Arc};

use buddle_math::*;
use buddle_utils::{color::Color, hash::StringIdBuilder};

use crate::{
    cpp::*,
    serde::{self, de::DynDeserializer, ser::DynSerializer, Baton, IdentityType},
    type_info::*,
    Container, ContainerIter, PropertyClass, PropertyClassExt, Type, TypeMut, TypeOwned, TypeRef,
};

/// Implements the [`Reflected`][crate::type_info::Reflected]
/// trait for custom types.
///
/// # Safety
///
/// This macro always produced info for **leaf types**, do
/// not use it with [`PropertyClass`][crate::PropertyClass]es.
#[macro_export]
macro_rules! impl_leaf_info_for {
    ($ty:ty) => {
        // SAFETY: User is responsible for meeting the invariants.
        unsafe impl $crate::type_info::Reflected for $ty {
            const TYPE_NAME: &'static str = $crate::__private::type_name::<Self>();

            const TYPE_INFO: &'static $crate::type_info::TypeInfo =
                &$crate::type_info::TypeInfo::leaf::<$ty>(::std::option::Option::None);
        }
    };

    ($ty:ty, $name:expr) => {
        // SAFETY: User is responsible for meeting the invariants.
        unsafe impl $crate::type_info::Reflected for $ty {
            const TYPE_NAME: &'static str = $name;

            const TYPE_INFO: &'static $crate::type_info::TypeInfo =
                &$crate::type_info::TypeInfo::leaf::<$ty>(::std::option::Option::Some($name));
        }
    };
}

/// Implements most of the [`Type`][crate::Type] methods
/// in-place.
///
/// # Example
///
/// ```
/// # use buddle_object_property::{impl_leaf_info_for, impl_type_methods, Type};
/// struct Example;
///
/// impl Type for Example {
///     impl_type_methods!(Value);
/// }
/// impl_leaf_info_for!(Example);
/// ```
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
        fn as_type(&self) -> &dyn $crate::Type {
            self
        }

        #[inline]
        fn as_type_mut(&mut self) -> &mut dyn $crate::Type {
            self
        }

        #[inline]
        fn as_boxed_type(self: ::std::boxed::Box<Self>) -> ::std::boxed::Box<dyn $crate::Type> {
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

            fn serialize(&self, serializer: &mut dyn DynSerializer, _: Baton) -> serde::Result<()> {
                serializer.marshal().$ty(*self)
            }

            fn deserialize(
                &mut self,
                deserializer: &mut dyn DynDeserializer,
                _: Baton,
            ) -> serde::Result<()> {
                *self = deserializer.unmarshal().$ty()?;
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

    fn serialize(&self, serializer: &mut dyn DynSerializer, _: Baton) -> serde::Result<()> {
        serializer.marshal().str(&self.0)
    }

    fn deserialize(
        &mut self,
        deserializer: &mut dyn DynDeserializer,
        _: Baton,
    ) -> serde::Result<()> {
        *self = deserializer.unmarshal().str().map(Self)?;
        Ok(())
    }
}

impl_leaf_info_for!(RawWideString, "std::wstring");
impl Type for RawWideString {
    impl_type_methods!(Value);

    fn serialize(&self, serializer: &mut dyn DynSerializer, _: Baton) -> serde::Result<()> {
        serializer.marshal().wstr(&self.0)
    }

    fn deserialize(
        &mut self,
        deserializer: &mut dyn DynDeserializer,
        _: Baton,
    ) -> serde::Result<()> {
        *self = deserializer.unmarshal().wstr().map(Self)?;
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

            fn serialize(
                &self,
                serializer: &mut dyn DynSerializer,
                baton: Baton,
            ) -> serde::Result<()> {
                // Serialize the entire container.
                serializer.container(self, baton)
            }

            fn deserialize(
                &mut self,
                deserializer: &mut dyn DynDeserializer,
                baton: Baton,
            ) -> serde::Result<()> {
                // We want to get rid of all the elements to avoid mixup
                // between what was serialized and what was already there.
                self.clear();

                // Deserialize the entire container.
                deserializer.container(
                    &mut |visitor, baton| {
                        let len = visitor.element_count().unwrap_or(0);

                        // Reserve sufficient space for the elements in advance.
                        self.reserve(len);

                        // Deserialize every individual element.
                        for _ in 0..len {
                            let mut element = T::default();
                            visitor.next(&mut element, baton)?;
                            <$ty>::$push(self, element);
                        }

                        Ok(())
                    },
                    baton,
                )
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

            fn clear(&mut self) {
                <$ty>::clear(self);
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

            fn serialize(
                &self,
                serializer: &mut dyn DynSerializer,
                b: Baton,
            ) -> serde::Result<()> {
                // XXX: Human-readable output?
                $(
                    self.$idents.serialize(serializer, b)?;
                )*

                Ok(())
            }

            fn deserialize(
                &mut self,
                deserializer: &mut dyn DynDeserializer,
                b: Baton,
            ) -> serde::Result<()> {
                // XXX: Human-readable output?
                $(
                    self.$idents.deserialize(deserializer, b)?;
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

            fn serialize(
                &self,
                serializer: &mut dyn DynSerializer,
                b: Baton,
            ) -> serde::Result<()> {
                // XXX: Human-readable output?
                $(
                    self.$idents.serialize(serializer, b)?;
                )*

                Ok(())
            }

            fn deserialize(
                &mut self,
                deserializer: &mut dyn DynDeserializer,
                b: Baton,
            ) -> serde::Result<()> {
                // XXX: Human-readable output?
                $(
                    self.$idents.deserialize(deserializer, b)?;
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
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_type(&self) -> &dyn Type {
        self
    }

    fn as_type_mut(&mut self) -> &mut dyn Type {
        self
    }

    fn as_boxed_type(self: Box<Self>) -> Box<dyn Type> {
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
            Ok(value) => {
                *self = *value;
                Ok(())
            }
            Err(value) => match value.type_owned() {
                TypeOwned::Class(c) if c.base_as::<T>().is_some() => {
                    self.value = Some(c);
                    Ok(())
                }
                x => Err(x.into_type()),
            },
        }
    }

    fn serialize(&self, serializer: &mut dyn DynSerializer, baton: Baton) -> serde::Result<()> {
        // First, serialize the identity of the object we have one.
        let identity = self.value.as_ref().map(|v| v.property_list());
        serializer.identity(identity, IdentityType::RawPtr, baton)?;

        // When a value is also present, serialize it.
        if let Some(value) = self.value.as_deref() {
            serializer.class(value, baton)?;
        }

        Ok(())
    }

    fn deserialize(
        &mut self,
        deserializer: &mut dyn DynDeserializer,
        baton: Baton,
    ) -> serde::Result<()> {
        if let Some(identity) = deserializer.identity(IdentityType::RawPtr, baton)? {
            // Create the default instance of the type we're expecting.
            if let Err(e) = self.as_type_mut().set(identity.make_default()) {
                return Err(serde::Error::custom(format_args!(
                    "failed to deserialize incompatible pointer type {}",
                    e.type_info().type_name()
                )));
            }

            // Deserialize the pointed-to value.
            deserializer.class(self.value.as_deref_mut().unwrap(), baton)?;
        } else {
            self.value = None;
        }

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
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_type(&self) -> &dyn Type {
        self
    }

    fn as_type_mut(&mut self) -> &mut dyn Type {
        self
    }

    fn as_boxed_type(self: Box<Self>) -> Box<dyn Type> {
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
            Ok(value) => {
                *self = *value;
                Ok(())
            }
            Err(value) => match value.type_owned() {
                TypeOwned::Class(c) if c.base_as::<T>().is_some() => {
                    self.value = Some(Arc::from(c));
                    Ok(())
                }
                x => Err(x.into_type()),
            },
        }
    }

    fn serialize(&self, serializer: &mut dyn DynSerializer, baton: Baton) -> serde::Result<()> {
        // First, serialize the identity of the object we have one.
        let identity = self.value.as_ref().map(|v| v.property_list());
        serializer.identity(identity, IdentityType::RawPtr, baton)?;

        // When a value is also present, serialize it.
        if let Some(value) = self.value.as_deref() {
            serializer.class(value, baton)?;
        }

        Ok(())
    }

    fn deserialize(
        &mut self,
        deserializer: &mut dyn DynDeserializer,
        baton: Baton,
    ) -> serde::Result<()> {
        if let Some(identity) = deserializer.identity(IdentityType::RawPtr, baton)? {
            // Create the default instance of the type we're expecting.
            if let Err(e) = self.as_type_mut().set(identity.make_default()) {
                return Err(serde::Error::custom(format_args!(
                    "Failed to deserialize incompatible pointer type {}",
                    e.type_info().type_name()
                )));
            }

            // Deserialize the pointed-to value.
            // We previously just set the value, so we shouldn't
            // have any borrowers which would make unwrap fail.
            deserializer.class(self.value.as_mut().and_then(Arc::get_mut).unwrap(), baton)?;
        } else {
            self.value = None;
        }

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
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_type(&self) -> &dyn Type {
        self
    }

    fn as_type_mut(&mut self) -> &mut dyn Type {
        self
    }

    fn as_boxed_type(self: Box<Self>) -> Box<dyn Type> {
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
}
