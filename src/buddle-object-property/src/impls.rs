use std::collections::VecDeque;

use crate::{
    serde::{self, de::DynDeserializer, ser::DynSerializer, Baton},
    type_info::{Reflected, TypeInfo},
    Container, ContainerIter, Type,
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
            const TYPE_INFO: &'static $crate::type_info::TypeInfo =
                &$crate::type_info::TypeInfo::leaf::<$ty>(::std::option::Option::None);
        }
    };

    ($ty:ty, $name:expr) => {
        // SAFETY: User is responsible for meeting the invariants.
        unsafe impl $crate::type_info::Reflected for $ty {
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
        fn type_ref(&self) -> $crate::TypeRef<'_> {
            $crate::TypeRef::$kind(self)
        }

        #[inline]
        fn type_mut(&mut self) -> $crate::TypeMut<'_> {
            $crate::TypeMut::$kind(self)
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

macro_rules! impl_container {
    ($ty:path, $deref:ty, $push:ident, $pop:ident) => {
        unsafe impl<T: Default + Reflected + Type> Reflected for $ty {
            const TYPE_INFO: &'static TypeInfo =
                &TypeInfo::leaf::<$ty>(Some(<T as Reflected>::TYPE_INFO.type_name()));
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
