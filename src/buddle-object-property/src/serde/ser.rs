//! Serialization facilities for reflected types.

use std::marker::PhantomData;

use super::{result::*, Baton, IdentityType};
use crate::{type_info::PropertyList, Container, Enum, PropertyClass};

mod sealed {
    pub trait Sealed {}
}

/// Defines the encoding of primitive types into the format.
///
/// This is the foundation for format-agnostic serialization
/// as it allows marshalling of primitives without being
/// concerned about their representation.
pub trait Marshal {
    /// Whether primitives are marshalled into a text or
    /// binary format.
    fn human_readable(&self) -> bool;

    /// Marshals a [`bool`] value.
    fn bool(&mut self, v: bool) -> Result<()>;

    /// Marshals an [`i8`] value.
    fn i8(&mut self, v: i8) -> Result<()>;

    /// Marshals an [`i16`] value.
    fn i16(&mut self, v: i16) -> Result<()>;

    /// Marshals an [`i32`] value.
    fn i32(&mut self, v: i32) -> Result<()>;

    /// Marshals a [`u8`] value.
    fn u8(&mut self, v: u8) -> Result<()>;

    /// Marshals a [`u16`] value.
    fn u16(&mut self, v: u16) -> Result<()>;

    /// Marshals a [`u32`] value.
    fn u32(&mut self, v: u32) -> Result<()>;

    /// Marshals a [`u64`] value.
    fn u64(&mut self, v: u64) -> Result<()>;

    /// Marshals an [`f32`] value.
    fn f32(&mut self, v: f32) -> Result<()>;

    /// Marshals an [`f64`] value.
    fn f64(&mut self, v: f64) -> Result<()>;

    /// Marshals a byte string value.
    fn str(&mut self, v: &[u8]) -> Result<()>;

    /// Marshals a wide string value.
    fn wstr(&mut self, v: &[u16]) -> Result<()>;
}

/// Defines the handling of the data format around the
/// marshalling of primitive types.
pub trait Layout {
    /// Serializes the *identity* of a [`PropertyClass`]
    /// into the described format.
    ///
    /// The identity is a per-class unique piece of
    /// information the deserializer can use to dynamically
    /// identify the serialized object's type.
    fn identity(
        &mut self,
        m: &mut dyn Marshal,
        v: Option<&'static PropertyList>,
        ty: IdentityType,
        baton: Baton,
    ) -> Result<()>;

    /// Serializes a [`PropertyClass`] object into the
    /// described format.
    ///
    /// NOTE: This should NOT serialize the identity of
    /// the object with [`Layout::identity`]. Instead,
    /// the serialization logic of every [`PropertyClass`]
    /// is responsible for that.
    fn class(&mut self, m: &mut dyn Marshal, v: &dyn PropertyClass, baton: Baton) -> Result<()>;

    /// Serializes a [`Container`] object into the
    /// described format.
    fn container(&mut self, m: &mut dyn Marshal, v: &dyn Container, baton: Baton) -> Result<()>;

    /// Serializes an [`Enum`] variant into the
    /// described format.
    fn enum_variant(&mut self, m: &mut dyn Marshal, v: &dyn Enum, baton: Baton) -> Result<()>;
}

/// Type-erased [`Serializer`] that can be passed to
/// object-safe traits without losing functionality.
pub trait DynSerializer: sealed::Sealed {
    /// Gets the serializer's [`Marshal`] layer for
    /// marshalling primitives.
    fn marshal(&mut self) -> &mut dyn Marshal;

    /// Whether this serializer produces human-readable
    /// (text-based) or binary output.
    fn human_readable(&self) -> bool;

    /// Serializes the *identity* of a [`PropertyClass`]
    /// into the described format.
    ///
    /// The identity is a per-class unique piece of
    /// information the deserializer can use to dynamically
    /// identify the serialized object's type.
    fn identity(
        &mut self,
        v: Option<&'static PropertyList>,
        ty: IdentityType,
        baton: Baton,
    ) -> Result<()>;

    /// Serializes a [`PropertyClass`] object into the
    /// described format.
    fn class(&mut self, v: &dyn PropertyClass, baton: Baton) -> Result<()>;

    /// Serializes a [`Container`] object into the
    /// described format.
    fn container(&mut self, v: &dyn Container, baton: Baton) -> Result<()>;

    /// Serializes an [`Enum`] variant into the
    /// described format.
    fn enum_variant(&mut self, v: &dyn Enum, baton: Baton) -> Result<()>;
}

/// An extension trait for adding custom pre and post
/// serialization logic to [`Serializer`].
pub trait SerializerExt: Sized {
    /// A result type produced by [`SerializerExt::post`].
    type Res;

    /// Custom logic before serialization.
    fn pre<M, L>(serializer: &mut Serializer<M, L, Self>) -> Result<()>;

    /// Custom logic after serialization.
    fn post<M, L>(serializer: Serializer<M, L, Self>) -> Result<Self::Res>;
}

/// A serializer for reflected values that wraps
/// [`Marshal`] and [`Layout`] strategies.
pub struct Serializer<M, L, Ext> {
    marshal: M,
    layout: L,
    _ext: PhantomData<Ext>,
}

impl<M, L, Ext> Serializer<M, L, Ext> {
    /// Creates a new serializer from the given data.
    pub const fn new(marshal: M, layout: L) -> Self {
        Self {
            marshal,
            layout,
            _ext: PhantomData,
        }
    }

    /// Provides mutable access to the serializer's
    /// [`Marshal`] object.
    #[inline]
    pub fn marshal(&mut self) -> &mut M {
        &mut self.marshal
    }

    /// Provides mutable access to the serializer's
    /// [`Layout`] object.
    #[inline]
    pub fn layout(&mut self) -> &mut L {
        &mut self.layout
    }
}

impl<M: Marshal, L: Layout, Ext: SerializerExt> Serializer<M, L, Ext> {
    /// Serializes the given `obj` to a persistent format.
    pub fn serialize(mut self, obj: &mut dyn PropertyClass) -> Result<Ext::Res> {
        let baton = Baton(());

        Ext::pre(&mut self)?;

        obj.on_pre_load();
        self.layout.identity(
            &mut self.marshal,
            Some(obj.property_list()),
            IdentityType::Value,
            baton,
        )?;
        self.layout.class(&mut self.marshal, obj, baton)?;
        obj.on_post_load();

        Ext::post(self)
    }
}

impl<M, L, Ext> sealed::Sealed for Serializer<M, L, Ext> {}

impl<M: Marshal, L: Layout, Ext: SerializerExt> DynSerializer for Serializer<M, L, Ext> {
    fn marshal(&mut self) -> &mut dyn Marshal {
        self.marshal()
    }

    fn human_readable(&self) -> bool {
        self.marshal.human_readable()
    }

    fn identity(
        &mut self,
        v: Option<&'static PropertyList>,
        ty: IdentityType,
        baton: Baton,
    ) -> Result<()> {
        self.layout.identity(&mut self.marshal, v, ty, baton)
    }

    fn class(&mut self, v: &dyn PropertyClass, baton: Baton) -> Result<()> {
        self.layout.class(&mut self.marshal, v, baton)
    }

    fn container(&mut self, v: &dyn Container, baton: Baton) -> Result<()> {
        self.layout.container(&mut self.marshal, v, baton)
    }

    fn enum_variant(&mut self, v: &dyn Enum, baton: Baton) -> Result<()> {
        self.layout.enum_variant(&mut self.marshal, v, baton)
    }
}
