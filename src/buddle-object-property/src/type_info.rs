//! Statically constructible type information for reflected
//! Rust types.

use std::any::TypeId;

use crate::Type;

mod value;
pub use self::value::*;

/// Statically accessible [`TypeInfo`] for reflected types.
///
/// # Safety
///
/// `TYPE_INFO` must be an accurate description of the type
/// this trait is implemented for to avoid memory corruption
/// (and obvious semantic breakage).
///
/// [`PropertyClass`][crate::property_class::PropertyClass]es
/// must always have [`PropertyList`] returned, whereas all
/// other types use [`ValueInfo`].
pub unsafe trait Reflected {
    /// A reference to the [`TypeInfo`] object.
    const TYPE_INFO: &'static TypeInfo;
}

/// Provides object-safe [`TypeInfo`] for reflected types.
///
/// It is preferred to implement [`Reflected`] as this trait
/// comes for free with it.
///
/// # Safety
///
/// The same conditions as for [`Reflected`] apply.
pub unsafe trait DynReflected {
    /// Gets the [`TypeInfo`] object for `self`.
    fn type_info(&self) -> &'static TypeInfo;
}

// SAFETY: `Reflected` implementor upholds the invariants.
unsafe impl<T: Reflected> DynReflected for T {
    fn type_info(&self) -> &'static TypeInfo {
        Self::TYPE_INFO
    }
}

/// Type information for a reflected Rust type.
#[derive(Clone, Debug)]
pub enum TypeInfo {
    /// Type info for a leaf value type.
    ///
    /// This basically covers every type that is not a
    /// `PropertyClass`.
    Leaf(ValueInfo),
}

impl TypeInfo {
    /// Creates new [`ValueInfo`]-backed type information.
    ///
    /// This is a shortcut for calling [`ValueInfo::new`]
    /// and wrapping the result in [`TypeInfo::Value`].
    #[inline]
    pub const fn leaf<T: Type>(name: Option<&'static str>) -> Self {
        Self::Leaf(ValueInfo::new::<T>(name))
    }

    /// Gets the name of the type.
    #[inline]
    pub const fn type_name(&self) -> &'static str {
        match self {
            TypeInfo::Leaf(value) => value.type_name(),
        }
    }

    /// Gets the hash of the type's name.
    #[inline]
    pub const fn type_hash(&self) -> u32 {
        match self {
            TypeInfo::Leaf(value) => value.type_hash(),
        }
    }

    /// Gets the [`TypeId`] for the type.
    #[inline]
    pub const fn type_id(&self) -> TypeId {
        match self {
            TypeInfo::Leaf(value) => value.type_id(),
        }
    }

    /// Checks if the type `T` matches the reflected tpye.
    pub fn is<T: ?Sized + 'static>(&self) -> bool {
        match self {
            TypeInfo::Leaf(value) => value.is::<T>(),
        }
    }
}
