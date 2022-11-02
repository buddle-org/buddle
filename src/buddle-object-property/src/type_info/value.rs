use std::any::{type_name, TypeId};

use crate::Type;

/// [`TypeInfo`] for leaf types.
///
/// Leaf types in the *ObjectProperty* system are those
/// that are not `PropertyClass`es and therefore do not
/// provide nested access to child.
///
/// [`TypeInfo`]: super::TypeInfo
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ValueInfo {
    pub(crate) type_name: &'static str,
    pub(crate) type_hash: u32,
    pub(crate) type_id: TypeId,
}

impl ValueInfo {
    /// Creates new metadata for the given `T`.
    ///
    /// The `name` argument optionally allows for choosing
    /// a custom type name to hash. Defaults to Rust's
    /// [`type_name`] when the argument is missing.
    pub const fn new<T: Type>(name: Option<&'static str>) -> Self {
        let type_name = name.unwrap_or_else(type_name::<T>);
        Self {
            type_name,
            type_hash: 0, // FIXME: Properly hash this.
            type_id: TypeId::of::<T>(),
        }
    }

    /// Gets the name of the type.
    #[inline]
    pub const fn type_name(&self) -> &'static str {
        self.type_name
    }

    /// Gets the hash of the type's name.
    #[inline]
    pub const fn type_hash(&self) -> u32 {
        self.type_hash
    }

    /// Gets the [`TypeId`] for the type.
    #[inline]
    pub const fn type_id(&self) -> TypeId {
        self.type_id
    }

    /// Checks if `T` matches the reflected type.
    #[inline]
    pub fn is<T: ?Sized + 'static>(&self) -> bool {
        self.type_id == TypeId::of::<T>()
    }
}
