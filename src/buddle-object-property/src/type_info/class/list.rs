use std::{
    any::TypeId,
    ptr::{self, DynMetadata},
};

use super::{Property, PropertyAccess};
use crate::{type_info::ValueInfo, PropertyClass, Type};

/// A type description of a [`PropertyClass`] layout for
/// reflective interaction.
#[derive(Clone, Debug)]
pub struct PropertyList {
    type_info: ValueInfo,

    base: Option<Property>,
    properties: &'static [Property],

    class_meta: DynMetadata<dyn PropertyClass>,
}

impl PropertyList {
    /// Creates a new property list for a class `T`.
    ///
    /// This stores and exposes [`Property`] values for
    /// every field of the class.
    ///
    /// Users should prefer the `#[derive(Type)]` macro
    /// for constructing instances of this type.
    ///
    /// # Arguments
    ///
    /// - `name` optionally allows for choosing a custom
    ///   type name for the property class to be hashed.
    ///
    ///   Defaults to Rust's [`std::any::type_name`] when
    ///   a value is missing.
    ///
    /// - A "base class" [`Property`] for emulated
    ///   inheritance trees.
    ///
    /// - An array of unique [`Property`] objects which
    ///   occupy actual pointer-accessible fields in the
    ///   `T` struct.
    ///
    ///   This array excludes the base class property, if
    ///   any exists for the class.
    ///
    /// # Safety
    ///
    /// The arguments need to be supplied correctly.
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

            class_meta: ptr::metadata::<dyn PropertyClass>(ptr::null::<T>()),
        }
    }

    /// Gets the name of the class type.
    #[inline]
    pub const fn type_name(&self) -> &'static str {
        self.type_info.type_name()
    }

    /// Gets the hash of the class type name.
    #[inline]
    pub const fn type_hash(&self) -> u32 {
        self.type_info.type_hash()
    }

    /// Gets the [`TypeId`] of the represented type.
    #[inline]
    pub const fn type_id(&self) -> TypeId {
        self.type_info.type_id()
    }

    /// Checks if `T` matches the represented class type.
    #[inline]
    pub fn is<T: ?Sized + 'static>(&self) -> bool {
        self.type_info.is::<T>()
    }

    /// Gets the [`PropertyList`] for the base class type,
    /// if one exists.
    #[inline]
    pub const fn base_list(&self) -> Option<&'static PropertyList> {
        // FIXME: Once it's possible, use a closure instead.
        #[inline(always)]
        const fn const_get_list(property: &Property) -> Option<&'static PropertyList> {
            property.base_list()
        }

        self.base.as_ref().and_then(const_get_list)
    }

    /// Gets the number of properties in the list.
    ///
    /// NOTE: This does not include base classes if
    /// one exists.
    #[inline]
    pub const fn property_count(&self) -> usize {
        self.properties.len()
    }

    /// Gets the base class property, if one exists.
    #[inline]
    pub fn base(&self) -> Option<PropertyAccess<'_>> {
        self.base
            .as_ref()
            .map(|value| value.make_access(self.type_id()))
    }

    /// Attempts to find a property with a specific name.
    ///
    /// NOTE: This does not scan [`PropertyList`]s of base
    /// classes for the requested type.
    #[inline]
    pub fn property(&self, name: &str) -> Option<PropertyAccess<'_>> {
        self.properties
            .iter()
            .find(|p| p.name() == name)
            .map(|value| value.make_access(self.type_id()))
    }

    /// Attempts to find a property for a given hash.
    ///
    /// NOTE: This does not scan [`PropertyList`]s of base
    /// classes for the requested type.
    #[inline]
    pub fn property_for(&self, hash: u32) -> Option<PropertyAccess<'_>> {
        self.properties
            .iter()
            .find(|p| p.hash() == hash)
            .map(|value| value.make_access(self.type_id()))
    }

    /// Attempts to get a property at a specified index.
    ///
    /// NOTE: This does not scan [`PropertyList`]s of base
    /// classes for the requested type.
    #[inline]
    pub fn property_at(&self, idx: usize) -> Option<PropertyAccess<'_>> {
        self.properties
            .get(idx)
            .map(|value| value.make_access(self.type_id()))
    }

    /// Gets an immutable reference to the base class value,
    /// if one exists.
    #[inline]
    pub fn base_value<'p>(&self, obj: &'p dyn PropertyClass) -> Option<&'p dyn PropertyClass> {
        // Get the base class property and make sure we can
        // retrieve the value through the given `obj`.
        let base = self.base()?;
        let property = base.value(obj.type_info().type_id())?;

        // SAFETY: Coming from a reference, we get a valid data
        // pointer and correct lifetime for the resulting value.
        let value = unsafe { property.value(obj as *const dyn PropertyClass as *const ()) };

        // SAFETY: `property` is already a base type property,
        // otherwise we would have bailed earlier. Thus, we
        // don't need to check for None.
        let base_list = unsafe { property.base_list().unwrap_unchecked() };

        // Build a `PropertyClass` pointer from value.
        let ptr: *const dyn PropertyClass = ptr::from_raw_parts(
            value as *const dyn Type as *const (),
            // SAFETY: `value` is accessed through the base property,
            // therefore base_list's meta stores the correct vtable.
            base_list.class_meta,
        );

        // SAFETY: We can safely dereference the new pointer
        // since the value is only borrowed immutably.
        Some(unsafe { &*ptr })
    }

    /// Gets a mutable reference to the base class value,
    /// if one exists.
    #[inline]
    pub fn base_value_mut<'p>(
        &self,
        obj: &'p mut dyn PropertyClass,
    ) -> Option<&'p mut dyn PropertyClass> {
        // Get the base class property and make sure we can
        // retrieve the value through the given `obj`.
        let base = self.base()?;
        let property = base.value(obj.type_info().type_id())?;

        // Build a PropertyClass pointer for the base.
        //
        // We explicitly introduce a scope here so that
        // all outstanding data references get dropped
        // before we dereference the pointer.
        let ptr: *mut dyn PropertyClass = unsafe {
            // SAFETY: Coming from a reference, we get a valid data
            // pointer and correct lifetime for the resulting value.
            let value = property.value_mut(obj as *mut dyn PropertyClass as *mut ());

            // SAFETY: `property` is already a base type property,
            // otherwise we would have bailed earlier. Thus, we
            // don't need to check for None.
            let base_list = property.base_list().unwrap_unchecked();

            ptr::from_raw_parts_mut(
                value as *mut dyn Type as *mut (),
                // SAFETY: `value` is accessed through the base property,
                // therefore base_list's meta stores the correct vtable.
                base_list.class_meta,
            )
        };

        // SAFETY: We can safely dereference the new pointer
        // since the value is not borrowed anymore.
        Some(unsafe { &mut *ptr })
    }

    /// Returns an [`Iterator`] over all the [`Property`]
    /// objects in the list.
    ///
    /// NOTE: This does not yield properties from base
    /// class [`PropertyList`]s. Users are advised to
    /// check for these with [`PropertyList::base_list`].
    pub fn iter_properties(&self) -> impl Iterator<Item = PropertyAccess<'_>> {
        self.properties
            .iter()
            .map(|value| value.make_access(self.type_id()))
    }
}
