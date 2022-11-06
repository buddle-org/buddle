use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use once_cell::race::OnceBox;
use usync::RwLock;

use super::TypeInfo;

/// A storage cell for [`TypeInfo`] of non-generic types,
/// allowing static storage of instances.
///
/// If your type **is** generic, use [`GenericTypeInfoCell`]
/// instead. Using this type will lead to unexpected results.
pub struct NonGenericTypeInfoCell(OnceBox<TypeInfo>);

impl NonGenericTypeInfoCell {
    /// Creates a new, empty cell for non-generic type info.
    pub const fn new() -> Self {
        Self(OnceBox::new())
    }

    /// Returns a reference to the [`TypeInfo`] stored.
    ///
    /// If no [`TypeInfo`] is written for the cell yet, a new
    /// one will be created as needed.
    pub fn get_or_init<F>(&self, f: F) -> &TypeInfo
    where
        F: FnOnce() -> TypeInfo,
    {
        self.0.get_or_init(|| Box::new(f()))
    }
}

/// A storage cell for [`TypeInfo`] of generic types,
/// allowing static storage of instances.
///
/// If your type is non-generic, [`NonGenericTypeInfoCell`]
/// will serve better performance for the same gain.
pub struct GenericTypeInfoCell(OnceBox<RwLock<HashMap<TypeId, &'static TypeInfo>>>);

impl GenericTypeInfoCell {
    /// Creates a new, empty cell for generic type info.
    pub const fn new() -> Self {
        Self(OnceBox::new())
    }

    /// Returns a reference to the [`TypeInfo`] stored.
    ///
    /// This method will return a correct reference based on
    /// the given `T`. If no info is yet registered for the
    /// type, a new one will be lazily created and stored.
    pub fn get_or_insert<T, F>(&self, f: F) -> &TypeInfo
    where
        T: Any + ?Sized,
        F: FnOnce() -> TypeInfo,
    {
        let type_id = TypeId::of::<T>();

        let mapping = self.0.get_or_init(Box::default);
        if let Some(info) = mapping.read().get(&type_id) {
            return info;
        }

        mapping.write().entry(type_id).or_insert_with(|| {
            // We leak the allocation to obtain a `&'static` reference.
            // This should be fine as we expect it to remain statically
            // active for the lifetime of the entire application anyway.
            Box::leak(Box::new(f()))
        })
    }
}
