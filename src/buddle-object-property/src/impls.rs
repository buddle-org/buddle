use std::{
    collections::VecDeque,
    num::{NonZeroI16, NonZeroI32, NonZeroI8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8},
};

use crate::{
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

/// Implements the [`Type`][crate::Type] trait for custom
/// types.
///
/// # Safety
///
/// `kind` may be any of the [`TypeRef`][crate::TypeRef]
/// variants which will be used in place.
///
/// # Example
///
/// ```
/// # use buddle_object_property::{impl_leaf_info_for, impl_type_for, Type};
/// struct Example;
/// impl_leaf_info_for!(Example);
/// impl_type_for!(Value: <> Type for Example);
/// ```
#[macro_export]
macro_rules! impl_type_for {
    ($kind:ident: $($desc:tt)*) => {
        impl $($desc)* {
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
        }
    };
}

macro_rules! impl_primitive {
    ($ty:ty, $name:expr) => {
        impl_leaf_info_for!($ty, $name);
        impl_type_for!(Value: <> $crate::Type for $ty);
    };
}

impl_primitive!(bool, "bool");

impl_primitive!(i8, "char");
impl_primitive!(i16, "short");
impl_primitive!(i32, "int");

impl_primitive!(NonZeroI8, "char");
impl_primitive!(NonZeroI16, "short");
impl_primitive!(NonZeroI32, "int");

impl_primitive!(u8, "unsigned char");
impl_primitive!(u16, "unsigned short");
impl_primitive!(u32, "unsigned int");
impl_primitive!(u64, "unsigned __int64");

impl_primitive!(NonZeroU8, "unsigned char");
impl_primitive!(NonZeroU16, "unsigned short");
impl_primitive!(NonZeroU32, "unsigned int");
impl_primitive!(NonZeroU64, "unsigned __int64");

impl_primitive!(f32, "float");
impl_primitive!(f64, "double");

macro_rules! impl_container {
    ($ty:path, $deref:ty, $push:ident, $pop:ident) => {
        unsafe impl<T: Reflected + Type> Reflected for $ty {
            const TYPE_INFO: &'static TypeInfo =
                &TypeInfo::leaf::<$ty>(Some(<T as Reflected>::TYPE_INFO.type_name()));
        }

        impl_type_for!(Container: <T: Reflected + Type> Type for $ty);

        impl<T: Reflected + Type> Container for $ty {
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
                        panic!("Attempted to push invalid value of type {}!", v.type_info().type_name())
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
