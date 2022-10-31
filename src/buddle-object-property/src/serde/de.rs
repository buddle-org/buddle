//! Deserialization facilities for reflected types.

use std::marker::PhantomData;

use super::{result::*, Baton};
use crate::{type_info::PropertyList, Container, Enum, PropertyClass};

mod sealed {
    pub trait Sealed {}
}

/// Defines the decoding of primitive types from the format.
///
/// This is the foundation for format-agnostic deserialization
/// as it allows unmarshalling of primitives without being
/// concerned about their representation.
pub trait Unmarshal {
    /// Whether primitives are unmarshalled from a text or
    /// binary format.
    fn human_readable(&self) -> bool;

    /// Unmarshals a [`bool`] value.
    fn bool(&mut self) -> Result<bool>;

    /// Unmarshals an [`i8`] value.
    fn i8(&mut self) -> Result<i8>;

    /// Unmarshals an [`i16`] value.
    fn i16(&mut self) -> Result<i16>;

    /// Unmarshals an [`i32`] value.
    fn i32(&mut self) -> Result<i32>;

    /// Unmarshals a [`u8`] value.
    fn u8(&mut self) -> Result<u8>;

    /// Unmarshals a [`u16`] value.
    fn u16(&mut self) -> Result<u16>;

    /// Unmarshals a [`u32`] value.
    fn u32(&mut self) -> Result<u32>;

    /// Unmarshals a [`u64`] value.
    fn u64(&mut self) -> Result<u64>;

    /// Unmarshals an [`f32`] value.
    fn f32(&mut self) -> Result<f32>;

    /// Unmarshals an [`f64`] value.
    fn f64(&mut self) -> Result<f64>;

    /// Unmarshals a byte string value.
    fn str(&mut self) -> Result<Vec<u8>>;

    /// Unmarshals a wide string value.
    fn wstr(&mut self) -> Result<Vec<u16>>;
}

/// Defines the handling of the data format around the
/// marshalling of primitive types.
pub trait Layout {
    /// Deserializes the *identity* of a [`PropertyClass`]
    /// from the described format.
    ///
    /// The identity is a per-class unique piece of
    /// information the deserializer can use to dynamically
    /// identify the serialized object's type.
    fn identity(&mut self, m: &mut dyn Unmarshal, baton: Baton) -> Result<&'static PropertyList>;

    /// Deserializes a [`PropertyClass`] object from the
    /// described format in-place.
    ///
    /// NOTE: This should NOT deserialize the identity of
    /// the object with [`Layout::identity`]. Instead,
    /// the deserialization logic of every [`PropertyClass`]
    /// is responsible for that.
    fn class(
        &mut self,
        m: &mut dyn Unmarshal,
        v: &mut dyn PropertyClass,
        baton: Baton,
    ) -> Result<()>;

    /// Deserializes a [`Container`] object in-place from the
    /// described format.
    fn container(
        &mut self,
        m: &mut dyn Unmarshal,
        v: &mut dyn Container,
        baton: Baton,
    ) -> Result<()>;

    /// Deserializes an [`Enum`] variant from the described
    /// format in-place.
    fn enum_variant(&mut self, m: &mut dyn Unmarshal, v: &mut dyn Enum, baton: Baton)
        -> Result<()>;
}

/// Type-erased [`Deserializer`] that can be passed to
/// object-safe traits without losing functionality.
pub trait DynDeserializer: sealed::Sealed {
    /// Gets the deserializer's [`Unmarshal`] layer for
    /// unmarshalling primitives.
    fn unmarshal(&mut self) -> &mut dyn Unmarshal;

    /// Whether this deserializer reads from human-readable
    /// (text-based) or binary input.
    fn human_readable(&self) -> bool;

    /// Deserializes the *identity* of a [`PropertyClass`]
    /// from the described format.
    ///
    /// The identity is a per-class unique piece of
    /// information the deserializer can use to dynamically
    /// identify the serialized object's type.
    fn identity(&mut self, baton: Baton) -> Result<&'static PropertyList>;

    /// Deserializes a [`PropertyClass`] object from the
    /// described format in-place.
    ///
    /// NOTE: This should NOT deserialize the identity of
    /// the object with [`Layout::identity`]. Instead,
    /// the deserialization logic of every [`PropertyClass`]
    /// is responsible for that.
    fn class(&mut self, v: &mut dyn PropertyClass, baton: Baton) -> Result<()>;

    /// Deserializes a [`Container`] object in-place from the
    /// described format.
    fn container(&mut self, v: &mut dyn Container, baton: Baton) -> Result<()>;

    /// Deserializes an [`Enum`] variant from the described
    /// format in-place.
    fn enum_variant(&mut self, v: &mut dyn Enum, baton: Baton) -> Result<()>;
}

/// An extension trait for adding custom pre and post
/// deserialization logic to a [`Deserializer`].
pub trait DeserializerExt: Sized {
    /// Custom logic before deserialization.
    fn pre<M, L>(deserializer: &mut Deserializer<M, L, Self>) -> Result<()>;

    /// Custom logic after deserialization.
    fn post<M, L>(deserializer: Deserializer<M, L, Self>) -> Result<()>;
}

/// A deserializer for reflected values that wraps
/// [`Unmarshal`] and [`Layout`] strategies.
pub struct Deserializer<M, L, Ext> {
    unmarshal: M,
    layout: L,
    _ext: PhantomData<Ext>,
}

impl<M, L, Ext> Deserializer<M, L, Ext> {
    /// Creates a new deserializer from the given data.
    pub const fn new(unmarshal: M, layout: L) -> Self {
        Self {
            unmarshal,
            layout,
            _ext: PhantomData,
        }
    }

    /// Provides mutable access to the deserializer's
    /// [`Unmarshal`] object.
    #[inline]
    pub fn unmarshal(&mut self) -> &mut M {
        &mut self.unmarshal
    }

    /// Provides mutable access to the deserializer's
    /// [`Layout`] object.
    #[inline]
    pub fn layout(&mut self) -> &mut L {
        &mut self.layout
    }
}

impl<M: Unmarshal, L: Layout, Ext: DeserializerExt> Deserializer<M, L, Ext> {
    /// Deserializes the given `obj` from a persistent format.
    pub fn deserialize(mut self) -> Result<Box<dyn PropertyClass>> {
        let baton = Baton(());

        Ext::pre(&mut self)?;

        let identity = self.layout.identity(&mut self.unmarshal, baton)?;
        let mut object = identity.make_default();

        object.on_pre_load();
        self.layout
            .class(&mut self.unmarshal, &mut *object, baton)?;
        object.on_post_load();

        Ext::post(self)?;

        Ok(object)
    }

    /// Deserializes in-place to the given `obj`.
    pub fn deserialize_in_place(mut self, obj: &mut dyn PropertyClass) -> Result<()> {
        let baton = Baton(());

        Ext::pre(&mut self)?;

        obj.on_pre_load();
        obj.deserialize(&mut self, baton)?;
        obj.on_post_load();

        Ext::post(self)
    }
}

impl<M, L, Ext> sealed::Sealed for Deserializer<M, L, Ext> {}

impl<M: Unmarshal, L: Layout, Ext: DeserializerExt> DynDeserializer for Deserializer<M, L, Ext> {
    fn unmarshal(&mut self) -> &mut dyn Unmarshal {
        self.unmarshal()
    }

    fn human_readable(&self) -> bool {
        self.unmarshal.human_readable()
    }

    fn identity(&mut self, baton: Baton) -> Result<&'static PropertyList> {
        self.layout.identity(&mut self.unmarshal, baton)
    }

    fn class(&mut self, v: &mut dyn PropertyClass, baton: Baton) -> Result<()> {
        self.layout.class(&mut self.unmarshal, v, baton)
    }

    fn container(&mut self, v: &mut dyn Container, baton: Baton) -> Result<()> {
        self.layout.container(&mut self.unmarshal, v, baton)
    }

    fn enum_variant(&mut self, v: &mut dyn Enum, baton: Baton) -> Result<()> {
        self.layout.enum_variant(&mut self.unmarshal, v, baton)
    }
}
