use std::{
    any::TypeId,
    ptr::{self, DynMetadata},
};

use bitflags::bitflags;

use super::PropertyList;
use crate::{type_info::TypeInfo, PropertyClass, Type};

bitflags! {
    /// The configuration bits for [`Property`] values.
    pub struct PropertyFlags: u32 {
        const SAVE = 1 << 0;
        const COPY = 1 << 1;
        const PUBLIC = 1 << 2;
        const TRANSMIT = 1 << 3;
        const PRIVILEGED_TRANSMIT = 1 << 4;
        const PERSIST = 1 << 5;
        const DEPRECATED = 1 << 6;
        const NOSCRIPT = 1 << 7;
        const DELTA_ENCODE = 1 << 8;
        const BLOB = 1 << 9;

        const NOEDIT = 1 << 16;
        const FILENAME = 1 << 17;
        const COLOR = 1 << 18;
        const BITS = 1 << 20;
        const ENUM = 1 << 21;
        const LOCALIZED = 1 << 22;
        const STRING_KEY = 1 << 23;
        const OBJECT_ID = 1 << 24;
        const REFERENCE_ID = 1 << 25;
        const OBJECT_NAME = 1 << 27;
        const HAS_BASECLASS = 1 << 28;
    }
}

/// Description of a property in a [`PropertyClass`].
///
/// Properties, being reflected fields of Rust structs,
/// have the following details exposed for reflective
/// access:
///
/// - their unique name in the class compound
///
/// - [`TypeInfo`] for their storage type
///
/// - an individual set of [`PropertyFlags`]
#[derive(Clone, Debug)]
pub struct Property {
    name: &'static str,
    hash: u32,
    flags: PropertyFlags,

    type_info: &'static TypeInfo,
    base_info: Option<&'static PropertyList>,

    offset: usize,
    meta: DynMetadata<dyn Type>,
}

impl Property {
    /// Creates a new property description from the
    /// relevant details.
    ///
    /// # Safety
    ///
    /// - `T` must be the actual Rust type of the property
    ///   in the struct.
    ///
    /// - `offset` must be a valid and `T`-aligned offset
    ///   into the containing class struct of the property.
    ///
    ///   This involves accounting for the lack of layout
    ///   stability guarantees in repr(Rust) types.
    ///
    /// - `name` must be chosen so that [`Property::hash`]
    ///   does not clash with other property hashes in
    ///   that class.
    pub const unsafe fn new<T: Type>(
        name: &'static str,
        flags: PropertyFlags,
        base: bool,
        type_info: &'static TypeInfo,
        offset: usize,
    ) -> Self {
        let base_info = if base {
            match type_info {
                TypeInfo::Class(list) => Some(list),
                _ => unreachable!(),
            }
        } else {
            None
        };

        Self {
            name,
            hash: 0, // FIXME: Produce a real hash.
            flags,

            type_info,
            base_info,

            offset,
            meta: ptr::metadata::<dyn Type>(ptr::null::<T>()),
        }
    }

    /// Creates a special base class property.
    ///
    /// # Safety
    ///
    /// The same conditions as for [`Property::new`] apply.
    #[allow(unsafe_op_in_unsafe_fn)]
    pub const unsafe fn new_base<T: PropertyClass>(
        type_info: &'static TypeInfo,
        offset: usize,
    ) -> Self {
        // SAFETY: Caller upholds the invariants.
        Self::new::<T>("super", PropertyFlags::empty(), true, type_info, offset)
    }

    /// Gets the name of the property.
    #[inline]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Gets the dictionary hash of the property.
    ///
    /// The resulting value can be assumed to uniquely
    /// reference a property in its property class.
    #[inline]
    pub const fn hash(&self) -> u32 {
        self.hash
    }

    /// Gets the [`PropertyFlags`] for the property.
    #[inline]
    pub const fn flags(&self) -> PropertyFlags {
        self.flags
    }

    /// Indicates whether this property is the base class
    /// of its containing type.
    #[inline]
    pub const fn is_base(&self) -> bool {
        self.base_info.is_some()
    }

    /// Gets the [`TypeInfo`] for the property's type.
    #[inline]
    pub const fn type_info(&self) -> &'static TypeInfo {
        self.type_info
    }

    /// Gets the [`PropertyList`] for the base type of the
    /// containing class, if the property represents one.
    #[inline]
    pub const fn base_list(&self) -> Option<&'static PropertyList> {
        self.base_info
    }

    /// Gets an immutable reference to the property's value,
    /// given a pointer to its containing object.
    ///
    /// # Safety
    ///
    /// Unless you have a specific reason against it, prefer
    /// [`PropertyClass::property`] for accessing values.
    ///
    /// - `obj` must point to an initialized and aligned
    ///   instance of the object that contains this
    ///   [`Property`] value.
    ///
    /// - The object behind `obj` must not be mutably borrowed
    ///   when this method is called, to the point where the
    ///   returned reference will be dropped.
    ///
    /// - `'t` must be inferred to not outlive the value
    ///   behind `obj`.
    #[inline]
    pub unsafe fn value<'t>(&self, obj: *const ()) -> &'t dyn Type {
        // Compute a pointer to the property's value.
        let value: *const dyn Type = ptr::from_raw_parts(
            // SAFETY: We require that `obj` is valid in this context.
            unsafe { obj.byte_add(self.offset) },
            // SAFETY: `Property::new` uses the property's type to
            // infer correct metadata.
            self.meta,
        );

        // SAFETY: Since we require that no mutable reference
        // exists, we can safely dereference the pointer.
        unsafe { &*value }
    }

    /// Gets an mutable reference to the property's value,
    /// given a pointer to its containing object.
    ///
    /// # Safety
    ///
    /// Unless you have a specific reason against it, prefer
    /// [`PropertyClass::property_mut`] for accessing values.
    ///
    /// - `obj` must point to an initialized and aligned
    ///   instance of the object that contains this
    ///   [`Property`] value.
    ///
    /// - The object behind `obj` must not be borrowed at all
    ///   when this method is called, to the point where the
    ///   returned reference will be dropped.
    ///
    /// - `'t` must be inferred to not outlive the value
    ///   behind `obj`.
    #[inline]
    pub unsafe fn value_mut<'t>(&self, obj: *mut ()) -> &'t mut dyn Type {
        // Compute a pointer to the property's value.
        let value: *mut dyn Type = ptr::from_raw_parts_mut(
            // SAFETY: We require that `obj` is valid in this context.
            unsafe { obj.byte_add(self.offset) },
            // SAFETY: `Property::new` uses the property's type to
            // infer correct metadata.
            self.meta,
        );

        // SAFETY: Since we require that no borrows exist,
        // we can safely dereference the pointer.
        unsafe { &mut *value }
    }

    pub(crate) fn make_access(&self, parent: TypeId) -> PropertyAccess<'_> {
        PropertyAccess {
            value: self,
            parent,
        }
    }
}

/// A guard that only grants access to a [`Property`] to
/// the type that actually contains it.
#[derive(Clone, Copy, Debug)]
pub struct PropertyAccess<'a> {
    value: &'a Property,
    parent: TypeId,
}

impl<'a> PropertyAccess<'a> {
    /// Gets the name of the property.
    #[inline]
    pub const fn name(&self) -> &'static str {
        self.value.name()
    }

    /// Gets the dictionary hash of the property.
    ///
    /// The resulting value can be assumed to uniquely
    /// reference a property in its property class.
    #[inline]
    pub const fn hash(&self) -> u32 {
        self.value.hash()
    }

    /// Gets the [`PropertyFlags`] for the property.
    #[inline]
    pub const fn flags(&self) -> PropertyFlags {
        self.value.flags()
    }

    /// Gets the [`TypeInfo`] for the underlying property.
    pub fn type_info(&self) -> &'static TypeInfo {
        self.value.type_info()
    }

    /// Consumes the view and grants access to the wrapped
    /// [`Property`] value.
    ///
    /// As stated in the documentation of [`PropertyAccess`],
    /// this will only return [`Some`] when `T` is the
    /// containing class type of the property.
    #[inline]
    pub fn value(self, type_id: TypeId) -> Option<&'a Property> {
        (self.parent == type_id).then_some(self.value)
    }
}