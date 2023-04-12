use std::{
    any::TypeId,
    ptr::{self, DynMetadata},
};

use super::{Property, PropertyAccess};
use crate::{property_class::PropertyClass, r#type::Type, type_info::ValueInfo};

/// A [`PropertyClass`] layout description for reflective introspection
/// and access.
#[derive(Clone, Debug)]
pub struct PropertyList {
    type_info: ValueInfo,

    base: Option<Property>,
    properties: &'static [Property],
    default_fn: fn() -> Box<dyn PropertyClass>,

    meta: DynMetadata<dyn PropertyClass>,
}

impl PropertyList {
    /// Creates a new property list for a type `T`.
    ///
    /// This stores and exposes [`Property`] values for every struct field.
    ///
    /// Users should prefer the `#[derive(Type)]` macro for constructing
    /// instances of this type.
    ///
    /// # Safety
    ///
    /// The following arguments must be correctly supplied.
    ///
    /// - `name` optionally allows for choosing a custom type name for hashing.
    ///
    ///   Defaults to Rust's [`std::any::type_name`] otherwise.
    ///
    /// - `base` is an optional "base class" [`Property`] for simulated
    ///   inheritance trees.
    ///
    /// - `properties` is an array of unique [`Property`] objects for fields
    ///   in the struct type.
    ///
    ///   This array excludes the base class property.
    pub const unsafe fn new<T: PropertyClass>(
        name: Option<&'static str>,
        base: Option<Property>,
        properties: &'static [Property],
    ) -> Self {
        debug_assert!(base.as_ref().map(Property::is_base).unwrap_or(true));
        Self {
            type_info: ValueInfo::new::<T>(name),

            base,
            properties,
            default_fn: T::make_default,

            meta: ptr::metadata::<dyn PropertyClass>(ptr::null::<T>()),
        }
    }

    /// Gets the name of the class type.
    pub const fn type_name(&self) -> &'static str {
        self.type_info.type_name()
    }

    /// Gets the dictionary hash of the class type.
    pub const fn type_hash(&self) -> u32 {
        self.type_info.type_hash()
    }

    /// Gets the [`TypeId`] of the represented type.
    pub const fn type_id(&self) -> TypeId {
        self.type_info.type_id()
    }

    /// Checks if `T` matches the represented class type.
    pub fn is<T: 'static>(&self) -> bool {
        self.type_info.is::<T>()
    }

    /// Creates a default-initialized instance of the represented
    /// [`PropertyClass`] type.
    pub fn make_default(&self) -> Box<dyn PropertyClass> {
        (self.default_fn)()
    }

    /// Gets the [`PropertyList`] for the base class type, if one exists.
    pub const fn base_list(&self) -> Option<&'static PropertyList> {
        self.base.as_ref().and_then(Property::base_list)
    }

    /// Gets the number of properties in the list.
    ///
    /// This excludes the base class property, if one exists.
    pub const fn property_count(&self) -> usize {
        self.properties.len()
    }

    /// Gets the base class property, if one exists.
    pub fn base(&self) -> Option<PropertyAccess<'_>> {
        self.base.as_ref().map(|p| p.make_access(self.type_id()))
    }

    /// Attempts to find a property with a specific name.
    pub fn property(&self, name: &str) -> Option<PropertyAccess<'_>> {
        self.properties
            .iter()
            .find(|p| p.name() == name)
            .map(|p| p.make_access(self.type_id()))
    }

    /// Attempts to find a property for a given hash.
    ///
    /// NOTE: This does not scan [`PropertyList`]s of base types for the
    /// requested property.
    pub fn property_for(&self, hash: u32) -> Option<PropertyAccess<'_>> {
        self.properties
            .iter()
            .find(|p| p.hash() == hash)
            .map(|p| p.make_access(self.type_id()))
    }

    /// Attempts to find a property at a specified index.
    ///
    /// NOTE: This does not scan [`PropertyList`]s of base types for the
    /// requested property.
    pub fn property_at(&self, id: usize) -> Option<PropertyAccess<'_>> {
        self.properties
            .get(id)
            .map(|p| p.make_access(self.type_id()))
    }

    /// Gets an immutable reference to the base class value, if one exists.
    pub fn base_value<'a>(&self, obj: &'a dyn PropertyClass) -> Option<&'a dyn PropertyClass> {
        // Get the base class property and make sure we can retrieve the value
        // through the given `obj`.
        let base = self.base()?;
        let property = base.value(obj.type_info().type_id())?;

        // SAFETY: Coming from a reference, we get a valid data pointer and
        // correct lifetime for the resulting value.
        let value = unsafe { property.value(obj as *const dyn PropertyClass as *const ()) };

        // SAFETY: `property` is already a base type property, otherwise we would
        // have bailed earlier. Thus, we don't need to check for None.
        let base_list = unsafe { property.base_list().unwrap_unchecked() };

        // Build a `PropertyClass` pointer from `value`.
        let ptr: *const dyn PropertyClass = ptr::from_raw_parts(
            value as *const dyn Type as *const (),
            // SAFETY: `value` is accessed through the base property, therefore
            // `base_list` stores the correct metadata for it.
            base_list.meta,
        );

        // SAFETY: The value is only borrowed immutably.
        Some(unsafe { &*ptr })
    }

    /// Gets a mutable reference to the base class value, if one exists.
    pub fn base_value_mut<'a>(
        &self,
        obj: &'a mut dyn PropertyClass,
    ) -> Option<&'a mut dyn PropertyClass> {
        // Get the base class property and make sure we can retrieve the value
        // through the given `obj`.
        let base = self.base()?;
        let property = base.value(obj.type_info().type_id())?;

        // Introduce a new scope so that all outstanding references get dropped
        // before we dereference the resulting pointer.
        let ptr: *mut dyn PropertyClass = unsafe {
            // SAFETY: Coming from a reference, we get a valid data pointer and
            // correct lifetime for the resulting value.
            let value = property.value_mut(obj as *mut dyn PropertyClass as *mut ());

            // SAFETY: `property` is already a base type property, otherwise we
            // would have bailed earlier. Thus, we don't need to check for None.
            let base_list = property.base_list().unwrap_unchecked();

            ptr::from_raw_parts_mut(
                value as *mut dyn Type as *mut (),
                // SAFETY: `value` is accessed through the base property, therefore
                // `base_list` stores the correct metadata for it.
                base_list.meta,
            )
        };

        // SAFETY: The value is not borrowed anymore.
        Some(unsafe { &mut *ptr })
    }

    /// Returns an [`Iterator`] over all the [`Property`] objects in the list.
    ///
    /// NOTE: This does not yield properties from the base type, if one exists.
    pub fn iter_properties(&self) -> impl Iterator<Item = PropertyAccess<'_>> {
        self.properties
            .iter()
            .map(|p| p.make_access(self.type_id()))
    }
}
