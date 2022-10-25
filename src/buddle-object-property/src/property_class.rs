use std::any::TypeId;

use crate::{
    type_info::{PropertyAccess, PropertyList, TypeInfo},
    Type,
};

macro_rules! unsafe_debug_unwrap {
    (($expr:expr): $pat:pat => $res:expr) => {
        match $expr {
            $pat => $res,

            // Panic in debug builds to make debugging easier
            // for unsound trait implementations.
            #[cfg(debug_assertions)]
            _ => unreachable!(),

            // In release builds, we may assume that unsound
            // code has been fixed and tested, so we risk the
            // UB there.
            #[cfg(not(debug_assertions))]
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    };
}

/// A trait providing the functionality of a property class.
///
/// Property classes are Rust structs which provide reflected
/// access and metadata to their fields.
///
/// This works by requesting the [`PropertyList`] for the type
/// with [`PropertyClass::property_list`], and then operating
/// on it.
///
/// [`PropertyAccess`]es are used to access the object's fields
/// dynamically.
pub trait PropertyClass: Type {
    /// Gets the [`PropertyList`] for the represented type.
    fn property_list(&self) -> &'static PropertyList {
        // SAFETY: unsafe Reflected impl guarantees correct type_info.
        unsafe_debug_unwrap!((self.type_info()): TypeInfo::Class(list) => list)
    }

    /// Provides reflective immutable access to a property's
    /// value.
    ///
    /// A [`PropertyAccess`] can be obtained through the
    /// [`PropertyClass::property_list`] object.
    fn property(&self, view: PropertyAccess<'_>) -> Option<&dyn Type> {
        view.value(self.type_id()).map(|property| {
            let ptr = self as *const Self;

            // SAFETY: We're coming from a reference, so the
            // pointer is valid. The inferred lifetime will
            // be that of `self`.
            unsafe { property.value(ptr.cast()) }
        })
    }

    /// Provides reflective mutable access to a property's
    /// value.
    ///
    /// A [`PropertyAccess`] can be obtained through the
    /// [`PropertyClass::property_list`] object.
    fn property_mut(&mut self, view: PropertyAccess<'_>) -> Option<&mut dyn Type> {
        view.value(self.type_id()).map(|property| {
            let ptr = self as *mut Self;

            // SAFETY: We're coming from a reference, so the
            // pointer is valid. The inferred lifetime will
            // be that of `self`.
            unsafe { property.value_mut(ptr.cast()) }
        })
    }

    /// Implementation-specific behavior for classes before
    /// they are serialized.
    fn on_pre_save(&mut self) {}

    /// Implementation-specific behavior for classes after
    /// they were serialized.
    fn on_post_save(&mut self) {}

    /// Implementation-specific behavior for classes before
    /// they are deserialized.
    fn on_pre_load(&mut self) {}

    /// Implementation-specific behavior for classes after
    /// they were deserialized.
    fn on_post_load(&mut self) {}
}

/// Extension trait to [`PropertyClass`] which provides
/// shortcuts for downcasting/accessing bases.
pub trait PropertyClassExt: PropertyClass {
    /// Gets the base [`PropertyClass`] object associated
    /// with this one, if exists.
    fn base(&self) -> Option<&dyn PropertyClass>;

    /// Gets the base [`PropertyClass`] object associated
    /// with this one, if exists.
    fn base_mut(&mut self) -> Option<&mut dyn PropertyClass>;

    /// Recursively tries to find a base class `T` in the
    /// emulated inheritance tree.
    fn base_as<T: PropertyClass>(&self) -> Option<&T> {
        if self.type_id() == TypeId::of::<T>() {
            // This class is already the T we're looking for.
            Some(self.as_type().downcast_ref().unwrap())
        } else {
            // Recursively scan the base of this type for T.
            self.base().and_then(|base| base.base_as())
        }
    }

    /// Recursively tries to find a base class `T` in the
    /// emulated inheritance tree.
    fn base_as_mut<T: PropertyClass>(&mut self) -> Option<&mut T> {
        if self.type_id() == TypeId::of::<T>() {
            // This class is already the T we're looking for.
            Some(self.as_type_mut().downcast_mut().unwrap())
        } else {
            // Recursively scan the base of this type for T.
            self.base_mut().and_then(|base| base.base_as_mut())
        }
    }

    /// Provides reflective access to an immutable property
    /// as a downcasted type.
    fn property_as<T: Type>(&self, view: PropertyAccess<'_>) -> Option<&T> {
        self.property(view).and_then(<dyn Type>::downcast_ref)
    }

    /// Provides reflective access to a mutable property as
    /// a downcasted type.
    fn property_as_mut<T: Type>(&mut self, view: PropertyAccess<'_>) -> Option<&mut T> {
        self.property_mut(view).and_then(<dyn Type>::downcast_mut)
    }
}

impl PropertyClassExt for dyn PropertyClass {
    fn base(&self) -> Option<&dyn PropertyClass> {
        let list = self.property_list();
        list.base_value(self)
    }

    fn base_mut(&mut self) -> Option<&mut dyn PropertyClass> {
        let list = self.property_list();
        list.base_value_mut(self)
    }
}

impl<P: PropertyClass> PropertyClassExt for P {
    fn base(&self) -> Option<&dyn PropertyClass> {
        let list = self.property_list();
        list.base_value(self)
    }

    fn base_mut(&mut self) -> Option<&mut dyn PropertyClass> {
        let list = self.property_list();
        list.base_value_mut(self)
    }
}
