use std::{
    marker::PhantomData,
    sync::{Arc, Weak},
};

use crate::{PropertyClass, PropertyClassExt};

macro_rules! impl_ptr {
    ($ty:ty, $ptr:ty) => {
        impl<T: PropertyClass> $ty {
            /// Creates a new pointer to a default-initialized
            /// value.
            pub fn new() -> Self
            where
                T: Default,
            {
                Self {
                    value: Some(<$ptr>::default()),
                    _t: PhantomData,
                }
            }

            /// Creates a new pointer initialized to null.
            #[inline]
            pub const fn null() -> Self {
                Self {
                    value: None,
                    _t: PhantomData,
                }
            }

            /// Whether the pointer is null, i.e. does not
            /// point to any value.
            #[inline]
            pub const fn is_null(&self) -> bool {
                self.value.is_none()
            }
        }

        impl<T: PropertyClass> Default for $ty {
            fn default() -> Self {
                Self::null()
            }
        }
    };
}

/// A simulated C++ pointer which can be serialized.
///
/// A `Ptr<T>` can hold any [`PropertyClass`] value
/// where `T` is a base of the actual stored type.
#[repr(transparent)]
pub struct Ptr<T: PropertyClass> {
    value: Option<Box<dyn PropertyClass>>,
    _t: PhantomData<T>,
}

impl<T: PropertyClass> Ptr<T> {
    /// Creates a new pointer to the given `value`.
    pub fn new_with_value(value: T) -> Self {
        Self {
            value: Some(Box::<T>::from(value)),
            _t: PhantomData,
        }
    }

    /// Sets the pointed-to value to the supplied one.
    pub fn set(&mut self, value: T) {
        self.value = Some(Box::<T>::from(value));
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
}

impl_ptr!(Ptr<T>, Box<T>);

/// A simulated C++ pointer with the semantics of a
/// `std::shared_ptr` which can be serialized.
///
/// A `SharedPtr<T>` can hold any [`PropertyClass`] value
/// where `T` is a base of the actual stored type.
#[repr(transparent)]
pub struct SharedPtr<T: PropertyClass> {
    value: Option<Arc<dyn PropertyClass>>,
    _t: PhantomData<T>,
}

impl<T: PropertyClass> SharedPtr<T> {
    /// Creates a new pointer to the given `value`.
    pub fn new_with_value(value: T) -> Self {
        Self {
            value: Some(Arc::<T>::from(value)),
            _t: PhantomData,
        }
    }

    /// Sets the pointed-to value to the supplied one.
    pub fn set(&mut self, value: T) {
        self.value = Some(Arc::<T>::from(value));
    }

    /// Gets the inner value as a `T` reference,
    /// if the pointer is non-null.
    #[inline]
    pub fn get(&self) -> Option<&T> {
        self.value.as_ref().map(|p| p.base_as::<T>().unwrap())
    }

    /// Gets the value as a [`WeakPtr`] without strong
    /// refcounting semantics.
    pub fn downgrade(&self) -> WeakPtr<T> {
        WeakPtr {
            value: self.value.as_ref().map(Arc::downgrade),
            _t: PhantomData,
        }
    }
}

impl_ptr!(SharedPtr<T>, Arc<T>);

/// A simulated C++ pointer with the semantics of a
/// `std::weak_ptr` which can be serialized.
///
/// A `WeakPtr<T>` can hold any [`PropertyClass`] value
/// where `T` is a base of the actual stored type.
#[repr(transparent)]
pub struct WeakPtr<T: PropertyClass> {
    value: Option<Weak<dyn PropertyClass>>,
    _t: PhantomData<T>,
}

impl<T: PropertyClass> WeakPtr<T> {
    /// Tries to upgrade the weak pointer into a [`SharedPtr`]
    /// if there are still strong references.
    pub fn upgrade(&self) -> Option<SharedPtr<T>> {
        self.value.as_ref().and_then(|value| {
            Some(SharedPtr {
                value: Some(value.upgrade()?),
                _t: PhantomData,
            })
        })
    }
}
