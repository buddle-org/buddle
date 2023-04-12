use std::{
    marker::PhantomData,
    sync::{Arc, Weak},
};

use crate::{PropertyClass, PropertyClassExt, Type};

/// A nullable, owned pointer to heap [`PropertyClass`] objects.
///
/// A [`Ptr`]`<T>` can hold any [`PropertyClass`] value where `T`
/// is a base type of the actual stored type.
#[derive(Debug)]
#[repr(transparent)]
pub struct Ptr<T> {
    // Invariant: Must be derived from `T` or `None`.
    pub(crate) value: Option<Box<dyn PropertyClass>>,

    _t: PhantomData<Box<T>>,
}

impl<T: PropertyClass> Ptr<T> {
    /// Creates a new pointer to a given type-erased [`PropertyClass`].
    ///
    /// Returns [`None`] if `value` is not derived from `T`.
    pub fn try_new(value: Box<dyn PropertyClass>) -> Result<Self, Box<dyn PropertyClass>> {
        // Invariant is met since `value` can be upcasted to `T`.
        if value.base_as::<T>().is_some() {
            Ok(Self {
                value: Some(value),
                _t: PhantomData,
            })
        } else {
            Err(value)
        }
    }

    /// Creates a new [`Ptr`] which does not point to a value.
    pub const fn null() -> Self {
        // Invariant is met since we don't have a value.
        Self {
            value: None,
            _t: PhantomData,
        }
    }

    /// Checks if this is a null pointer, i.e. doesn't point to a value.
    pub const fn is_null(&self) -> bool {
        self.value.is_none()
    }

    /// Gets an immutable reference to the raw value of the stored object.
    pub fn raw(&self) -> Option<&dyn PropertyClass> {
        self.value.as_deref()
    }

    /// Gets a mutable reference to the raw value of the stored object.
    pub fn raw_mut(&mut self) -> Option<&mut dyn PropertyClass> {
        self.value.as_deref_mut()
    }

    /// Gets an immutable reference to the stored value upcasted to the `T`
    /// base type.
    pub fn get(&self) -> Option<&T> {
        self.value.as_ref().map(|v| unsafe {
            // SAFETY: By type invariant, this can never fail.
            v.base_as::<T>().unwrap_unchecked()
        })
    }

    /// Gets a mutable reference to the stored value upcasted to the `T`
    /// base type.
    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.value.as_mut().map(|v| unsafe {
            // SAFETY: By type invariant, this can never fail.
            v.base_as_mut::<T>().unwrap_unchecked()
        })
    }

    /// Gets the inner value downcasted to `U`, if that matches the type.
    pub fn downcast<U: PropertyClass>(&self) -> Option<&U> {
        self.raw().and_then(|v| (v as &dyn Type).downcast_ref())
    }

    /// Gets the inner value downcasted to `U`, if that matches the type.
    pub fn downcast_mut<U: PropertyClass>(&mut self) -> Option<&mut U> {
        self.raw_mut()
            .and_then(|v| (v as &mut dyn Type).downcast_mut())
    }
}

// TODO: Clone, Copy traits?

impl<T: PropertyClass> Default for Ptr<T> {
    fn default() -> Self {
        Self::null()
    }
}

/// A simulated C++ shared pointer which can be serialized.
///
/// This has the reference counting semantics of Rust's [`Arc`] type.
///
/// A [`SharedPtr`]`<T>` can hold any [`PropertyClass`] value where
/// `T` is a base type of the actual stored type.
#[derive(Debug)]
#[repr(transparent)]
pub struct SharedPtr<T> {
    // Invariant: Must be derived from `T`.
    pub(crate) value: Arc<dyn PropertyClass>,

    _t: PhantomData<Arc<T>>,
}

impl<T: PropertyClass> SharedPtr<T> {
    /// Creates a new pointer to a given type-erased [`PropertyClass`].
    ///
    /// Returns [`None`] if `value` is not derived from `T`.
    pub fn try_new(value: Arc<dyn PropertyClass>) -> Result<Self, Arc<dyn PropertyClass>> {
        // Invariant is met since `value` can be upcasted to `T`.
        if value.base_as::<T>().is_some() {
            Ok(Self {
                value,
                _t: PhantomData,
            })
        } else {
            Err(value)
        }
    }

    /// Downgrades the pointer into a [`WeakPtr`].
    ///
    /// The resulting pointer is not strongly reference-counted and needs
    /// to be upgraded back into a [`SharedPtr`] to access its value.
    pub fn downgrade(&self) -> WeakPtr<T> {
        // Invariant is met since `self.value` is already checked.
        WeakPtr {
            value: Arc::downgrade(&self.value),
            _t: PhantomData,
        }
    }

    /// Gets an immutable reference to the raw value of the stored object.
    pub fn raw(&self) -> &dyn PropertyClass {
        &*self.value
    }

    /// Gets a mutable reference to the raw value of the stored object.
    pub fn raw_mut(&mut self) -> Option<&mut dyn PropertyClass> {
        Arc::get_mut(&mut self.value)
    }

    /// Gets an immutable reference to the stored value upcasted to the `T`
    /// base type.
    pub fn get(&self) -> &T {
        // SAFETY: By type invariant, this can never fail.
        unsafe { self.value.base_as::<T>().unwrap_unchecked() }
    }

    /// Gets a mutable reference to the stored value upcasted to the `T`
    /// base type.
    pub fn get_mut(&mut self) -> Option<&mut T> {
        Arc::get_mut(&mut self.value).map(|v| unsafe {
            // SAFETY: By type invariant, this can never fail.
            v.base_as_mut::<T>().unwrap_unchecked()
        })
    }

    /// Gets the inner value downcasted to `U`, if that matches the type.
    pub fn downcast<U: PropertyClass>(&self) -> Option<&U> {
        (self.raw() as &dyn Type).downcast_ref()
    }

    /// Gets the inner value downcasted to `U`, if that matches the type.
    pub fn downcast_mut<U: PropertyClass>(&mut self) -> Option<&mut U> {
        self.raw_mut()
            .and_then(|v| (v as &mut dyn Type).downcast_mut())
    }
}

// TODO: Clone, Copy traits?

#[derive(Debug)]
#[repr(transparent)]
pub struct WeakPtr<T> {
    // Invariant: Must be derived from `T`.
    pub(crate) value: Weak<dyn PropertyClass>,

    _t: PhantomData<Weak<T>>,
}

impl<T: PropertyClass> WeakPtr<T> {
    /// Upgrades the weak pointer to a [`SharedPtr`], if any strong
    /// references are still alive.
    ///
    /// This operation returns [`Some`] if the pointer value was null
    /// or if strong references were still intact.
    pub fn upgrade(&self) -> Option<SharedPtr<T>> {
        // Invariant is met since `self.value` is already checked.
        self.value.upgrade().map(|value| SharedPtr {
            value,
            _t: PhantomData,
        })
    }
}
