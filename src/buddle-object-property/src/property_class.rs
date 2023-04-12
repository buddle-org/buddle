use std::any::TypeId;

use crate::{
    r#type::Type,
    type_info::{PropertyAccess, PropertyList, TypeInfo},
};

macro_rules! unsafe_debug_unwrap {
    (($expr:expr): $pat:pat => $res:expr) => {
        match $expr {
            $pat => $res,

            // Panic in debug builds to make debugging easier for
            // unsound trait implementations.
            #[cfg(debug_assertions)]
            _ => unreachable!(),

            // In release builds, we may assume that unsound code has
            // been fixed and tested in dbeug, so we risk the UB there.
            #[cfg(not(debug_assertions))]
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    };
}

/// The PropertyClass abstraction in the *ObjectProperty* system.
///
/// `PropertyClass`es represent reflected Rust structs and provide dynamic
/// access to their fields and the associated type info.
///
/// It is advised to leave implementing this trait to the `#[derive(Type)]`
/// macro unless you have a specific reason not to.
pub trait PropertyClass: Type {
    /// Gets the [`PropertyList`] for the represented type.
    fn property_list(&self) -> &'static PropertyList {
        // SAFETY: unsafe Reflected impl guarantees correct type_info.
        unsafe_debug_unwrap!((self.type_info()): TypeInfo::Class(list) => list)
    }

    /// Creates a default instance of this type as a reflected value.
    fn make_default() -> Box<dyn PropertyClass>
    where
        Self: Sized;

    /// Provides immutable access to a property's value.
    ///
    /// A [`PropertyAccess`] can be obtained through the type's
    /// [`PropertyList`].
    ///
    /// # Panics
    ///
    /// Panics when an invalid [`PropertyAccess`] object was supplied.
    fn property(&self, view: PropertyAccess<'_>) -> &dyn Type {
        view.value(self.type_id())
            .map(|p| {
                let ptr = self as *const Self;

                // SAFETY: We're coming from a reference, so the pointer is valid.
                // The lifetime will be inferred to be that of `self`.
                unsafe { p.value(ptr.cast()) }
            })
            .expect("invalid PropertyAccess object supplied")
    }

    /// Provides mutable access to a property's value.
    ///
    /// A [`PropertyAccess`] can be obtained through the type's
    /// [`PropertyList`].
    ///
    /// # Panics
    ///
    /// Panics when an invalid [`PropertyAccess`] object was supplied.
    fn property_mut(&mut self, view: PropertyAccess<'_>) -> &mut dyn Type {
        view.value(self.type_id())
            .map(|p| {
                let ptr = self as *mut Self;

                // SAFETY: We're coming from a reference, so the pointer is valid.
                // The lifetime will be inferred to be that of `self`.
                unsafe { p.value_mut(ptr.cast()) }
            })
            .expect("invalid PropertyAccess object supplied")
    }

    /// Gets the base [`PropertyClass`] for this object, if one exists.
    fn base(&self) -> Option<&dyn PropertyClass>;

    /// Gets the base [`PropertyClass`] for this object, if one exists.
    fn base_mut(&mut self) -> Option<&mut dyn PropertyClass>;

    /// Implementation-specific behavior for a class before it is serialized.
    fn on_pre_save(&mut self);

    /// Implementation-specific behavior for a class after it was serialized.
    fn on_post_save(&mut self);

    /// Implementation-specific behavior for a class before it is deserialized.
    fn on_pre_load(&mut self);

    /// Implementation-specific behavior for a class after it was deserialized.
    fn on_post_load(&mut self);
}

/// Extension trait to [`PropertyClass`]es which provides shortcuts for
/// downcasting and accessing bases.
pub trait PropertyClassExt: PropertyClass {
    /// Recursively tries to find a base class `T` in the emulated
    /// inheritance tree.
    fn base_as<T: PropertyClass>(&self) -> Option<&T> {
        if self.type_id() == TypeId::of::<T>() {
            // This class is already the T we're looking for.
            Some(self.as_any().downcast_ref().unwrap())
        } else {
            // Recursively scan the base of this type for T.
            self.base().and_then(|base| base.base_as())
        }
    }

    /// Recursively tries to find a base class `T` in the emulated
    /// inheritance tree.
    fn base_as_mut<T: PropertyClass>(&mut self) -> Option<&mut T> {
        if self.type_id() == TypeId::of::<T>() {
            // This class is already the T we're looking for.
            Some(self.as_any_mut().downcast_mut().unwrap())
        } else {
            // Recursively scan the base of this type for T.
            self.base_mut().and_then(|base| base.base_as_mut())
        }
    }

    /// Provides access to an immutable property as a downcasted type.
    fn property_as<T: Type>(&self, view: PropertyAccess<'_>) -> Option<&T> {
        self.property(view).downcast_ref()
    }

    /// Provides access to a mutable property as a downcasted type.
    fn property_as_mut<T: Type>(&mut self, view: PropertyAccess<'_>) -> Option<&mut T> {
        self.property_mut(view).downcast_mut()
    }
}

impl PropertyClassExt for dyn PropertyClass {}
impl<P: PropertyClass> PropertyClassExt for P {}
