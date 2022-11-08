use std::{
    marker::PhantomData,
    sync::{Arc, Weak},
};

use crate::{PropertyClass, PropertyClassExt};

/// A simulated C++ pointer which can be serialized.
///
/// A `Ptr<T>` can hold any [`PropertyClass`] value
/// where `T` is a base of the actual stored type.
#[repr(transparent)]
pub struct Ptr<T> {
    // Invariant: Must be derived from `T`.
    pub(crate) value: Option<Box<dyn PropertyClass>>,
    _t: PhantomData<Box<T>>,
}

impl<T: PropertyClass> Ptr<T> {
    /// Creates a new pointer to the given `value`.
    ///
    /// If `value` is not derived from `T`, this
    /// will return [`None`].
    pub fn try_new(value: Box<dyn PropertyClass>) -> Option<Self> {
        if value.base_as::<T>().is_some() {
            Some(Self {
                value: Some(value),
                _t: PhantomData,
            })
        } else {
            None
        }
    }

    /// Creates a new pointer to the given `value`.
    ///
    /// # Panics
    ///
    /// Panics if `value` is not derived from `T`.
    pub fn new(value: Box<dyn PropertyClass>) -> Self {
        Self::try_new(value).unwrap()
    }

    /// Creates a new pointer initialized to null.
    pub const fn null() -> Self {
        Self {
            value: None,
            _t: PhantomData,
        }
    }

    /// Whether the pointer is null, i.e. does not
    /// point to any value.
    pub const fn is_null(&self) -> bool {
        self.value.is_none()
    }

    /// Gets the raw value of the stored object.
    pub fn get_raw(&self) -> Option<&dyn PropertyClass> {
        self.value.as_deref()
    }

    /// Gets the inner value as a `T` reference,
    /// if the pointer is non-null.
    #[inline]
    pub fn get(&self) -> Option<&T> {
        self.value.as_ref().map(|p| p.base_as::<T>().unwrap())
    }

    /// Gets the inner value as a mutable `T`
    /// reference, if the pointer is non-null.
    #[inline]
    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.value.as_mut().map(|p| p.base_as_mut::<T>().unwrap())
    }

    /// Gets the inner value downcasted to `U`,
    /// if that's the actual pointer type.
    ///
    /// If the pointer is null or `U` does not
    /// match the stored type, [`None`] will be
    /// returned.
    pub fn get_downcast<U: PropertyClass>(&self) -> Option<&U> {
        self.value.as_ref().and_then(|p| p.as_type().downcast_ref())
    }

    /// Gets the inner value downcasted to `U`,
    /// if that's the actual pointer type.
    ///
    /// If the pointer is null or `U` does not
    /// match the stored type, [`None`] will be
    /// returned.
    pub fn get_downcast_mut<U: PropertyClass>(&mut self) -> Option<&mut U> {
        self.value
            .as_mut()
            .and_then(|p| p.as_type_mut().downcast_mut())
    }
}

// TODO: Clone, Copy, Debug traits?

impl<T: PropertyClass> Default for Ptr<T> {
    fn default() -> Self {
        Self::null()
    }
}

/// A simulated C++ shared pointer which can be
/// serialized.
///
/// This has the reference counting semantics of
/// Rust's [`Arc`] type.
///
/// A `SharedPtr<T>` can hold any [`PropertyClass`] value
/// where `T` is a base of the actual stored type.
#[repr(transparent)]
pub struct SharedPtr<T> {
    // Invariant: Must be derived from `T`.
    pub(crate) value: Option<Arc<dyn PropertyClass>>,
    _t: PhantomData<Box<T>>,
}

impl<T: PropertyClass> SharedPtr<T> {
    /// Creates a new pointer to the given `value`.
    ///
    /// If `value` is not derived from `T`, this
    /// will return [`None`].
    pub fn try_new(value: Arc<dyn PropertyClass>) -> Option<Self> {
        if value.base_as::<T>().is_some() {
            Some(Self {
                value: Some(value),
                _t: PhantomData,
            })
        } else {
            None
        }
    }

    /// Creates a new pointer to the given `value`.
    ///
    /// # Panics
    ///
    /// Panics if `value` is not derived from `T`.
    pub fn new(value: Arc<dyn PropertyClass>) -> Self {
        Self::try_new(value).unwrap()
    }

    /// Creates a new pointer initialized to null.
    pub const fn null() -> Self {
        Self {
            value: None,
            _t: PhantomData,
        }
    }

    /// Downgrades the pointer into a [`WeakPtr`].
    ///
    /// The resulting pointer is not strongly reference
    /// counted and needs to be upgraded back into a
    /// [`SharedPtr`] to access its value.
    pub fn downgrade(&self) -> WeakPtr<T> {
        WeakPtr {
            value: self.value.as_ref().map(Arc::downgrade),
            _t: PhantomData,
        }
    }

    /// Whether the pointer is null, i.e. does not
    /// point to any value.
    pub const fn is_null(&self) -> bool {
        self.value.is_none()
    }

    /// Gets the raw value of the stored object.
    pub fn get_raw(&self) -> Option<&dyn PropertyClass> {
        self.value.as_deref()
    }

    /// Gets the inner value as a `T` reference,
    /// if the pointer is non-null.
    #[inline]
    pub fn get(&self) -> Option<&T> {
        self.value.as_ref().map(|p| p.base_as::<T>().unwrap())
    }

    /// Gets the inner value as a mutable `T`
    /// reference, if the pointer is non-null.
    ///
    /// If other mutable borrows of the pointer
    /// exist, this method will return [`None`].
    #[inline]
    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.value
            .as_mut()
            .and_then(|p| Arc::get_mut(p)?.base_as_mut::<T>())
    }

    /// Gets the inner value downcasted to `U`,
    /// if that's the actual pointer type.
    ///
    /// If the pointer is null or `U` does not
    /// match the stored type, [`None`] will be
    /// returned.
    pub fn get_downcast<U: PropertyClass>(&self) -> Option<&U> {
        self.value.as_ref().and_then(|p| p.as_type().downcast_ref())
    }

    /// Gets the inner value downcasted to `U`,
    /// if that's the actual pointer type.
    ///
    /// If other mutable borrows of the pointer
    /// exist, this method will return [`None`].
    ///
    /// If the pointer is null or `U` does not
    /// match the stored type, [`None`] will be
    /// returned.
    pub fn get_downcast_mut<U: PropertyClass>(&mut self) -> Option<&mut U> {
        self.value
            .as_mut()
            .and_then(|p| Arc::get_mut(p)?.as_type_mut().downcast_mut())
    }
}

// TODO: Clone, Copy, Debug traits?

impl<T: PropertyClass> Default for SharedPtr<T> {
    fn default() -> Self {
        Self::null()
    }
}

/// A simulated C++ weak pointer which can be
/// serialized.
///
/// This has the reference counting semantics of
/// Rust's [`Weak`] type.
///
/// A `WeakPtr<T>` can hold any [`PropertyClass`] value
/// where `T` is a base of the actual stored type.
#[repr(transparent)]
pub struct WeakPtr<T> {
    // Invariant: Must be derived from `T`.
    pub(crate) value: Option<Weak<dyn PropertyClass>>,
    _t: PhantomData<Box<T>>,
}

impl<T: PropertyClass> WeakPtr<T> {
    /// Upgrades the weak pointer into a [`SharedPtr`],
    /// if any strong references are still alive.
    ///
    /// This operation returns [`Some`] if the pointer
    /// value was null or if strong references were
    /// still intact.
    pub fn upgrade(&self) -> Option<SharedPtr<T>> {
        self.value
            .as_ref()
            .and_then(|value| {
                Some(SharedPtr {
                    value: Some(value.upgrade()?),
                    _t: PhantomData,
                })
            })
            .or_else(|| {
                Some(SharedPtr {
                    value: None,
                    _t: PhantomData,
                })
            })
    }
}
