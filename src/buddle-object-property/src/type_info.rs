//! Statically embeddable type information for reflected Rust types.

use std::any::TypeId;

use crate::r#type::Type;

mod class;
pub use class::*;

mod value;
pub use value::*;

/// Statically accessible [`TypeInfo`] for reflected types.
///
/// # Safety
///
/// Implementors must ensure `TYPE_INFO` is an accurate description
/// of the reflected type to avoid memory corruption (and obvious
/// semantic breakage).
///
/// [`PropertyClass`][crate::PropertyClass]es must always have
/// [`PropertyList`] returned, whereas other types use [`ValueInfo`].
pub unsafe trait Reflected {
    /// The human-readable name of the type.
    const TYPE_NAME: &'static str;

    /// A reference to the associated [`TypeInfo`].
    const TYPE_INFO: &'static TypeInfo;
}

/// Provides object-safe [`TypeInfo`] for reflected values.
///
/// It is preferred to implement [`Reflected`] as implementations of
/// this trait come for free through it.
///
/// # Safety
///
/// Same conditions as for [`Reflected`] apply.
pub unsafe trait DynReflected {
    /// Gets the [`TypeInfo`] for `self`.
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
    /// Type info for a property class type.
    ///
    /// It stores properties and provides reflective access
    /// to them through the [`PropertyList`].
    Class(PropertyList),
    /// Type info for a leaf value type.
    ///
    /// This basically covers every type that is not a
    /// `PropertyClass`.
    Leaf(ValueInfo),
}

impl TypeInfo {
    /// Creates new [`ValueInfo`]-backed type info.
    ///
    /// This is a shortcut for calling [`ValueInfo::new`] and wrapping the
    /// result in [`TypeInfo::Leaf`].
    #[inline]
    pub const fn leaf<T: Type>(name: Option<&'static str>) -> Self {
        Self::Leaf(ValueInfo::new::<T>(name))
    }

    /// Gets the human-readable name of the type.
    #[inline]
    pub const fn type_name(&self) -> &'static str {
        match self {
            TypeInfo::Class(value) => value.type_name(),
            TypeInfo::Leaf(value) => value.type_name(),
        }
    }

    /// Gets the hash of the type's name.
    #[inline]
    pub const fn type_hash(&self) -> u32 {
        match self {
            TypeInfo::Class(value) => value.type_hash(),
            TypeInfo::Leaf(value) => value.type_hash(),
        }
    }

    /// Gets the [`TypeId`] for the type.
    #[inline]
    pub const fn type_id(&self) -> TypeId {
        match self {
            TypeInfo::Class(value) => value.type_id(),
            TypeInfo::Leaf(value) => value.type_id(),
        }
    }

    /// Checks if the type `T` matches the reflected type.
    pub fn is<T: 'static>(&self) -> bool {
        match self {
            TypeInfo::Class(value) => value.is::<T>(),
            TypeInfo::Leaf(value) => value.is::<T>(),
        }
    }
}
