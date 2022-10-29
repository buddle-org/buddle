use super::result::*;
use crate::{Container, Enum, PropertyClass};

mod sealed {
    pub trait Sealed {}
}

/// A type that supports serialization.
pub trait Serialize {
    /// Serializes `self` to the given `serializer`.
    fn serialize(&self, serializer: &mut dyn DynSerializer) -> Result<()>;
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
/// marshalling of types.
pub trait Layout {
    /// Serializes a [`PropertyClass`] object into the
    /// described format.
    ///
    /// NOTE: In some cases, the value of a [`PropertyClass`]
    /// may not be available. It is then recommended to
    /// pass [`None`] to indicate that.
    fn class(&mut self, m: &mut dyn Marshal, v: Option<&dyn PropertyClass>) -> Result<()>;

    /// Serializes a [`Container`] object into the
    /// described format.
    fn container(&mut self, m: &mut dyn Marshal, v: &dyn Container) -> Result<()>;

    /// Serializes an [`Enum`] variant into the
    /// described format.
    fn enum_variant(&mut self, m: &mut dyn Marshal, v: &dyn Enum) -> Result<()>;
}

/// Type-erased [`Serializer`] that can be passed to
/// object-safe traits without losing functionality.
pub trait DynSerializer: sealed::Sealed {
    /// Gets the serializer's [`Marshal`] layer for
    /// marshalling primitives.
    fn marshal(&mut self) -> &mut dyn Marshal;

    /// Serializes a [`PropertyClass`] object into the
    /// described format.
    ///
    /// NOTE: In some cases, the value of a [`PropertyClass`]
    /// may not be available. It is then recommended to
    /// pass [`None`] to indicate that.
    fn class(&mut self, v: Option<&dyn PropertyClass>) -> Result<()>;

    /// Serializes a [`Container`] object into the
    /// described format.
    fn container(&mut self, v: &dyn Container) -> Result<()>;

    /// Serializes an [`Enum`] variant into the
    /// described format.
    fn enum_variant(&mut self, v: &dyn Enum) -> Result<()>;
}

/// A serializer for reflected values that wraps
/// [`Marshal`] and [`Layout`] strategies.
pub struct Serializer<M, L> {
    marshal: M,
    layout: L,
}

impl<M, L> Serializer<M, L> {
    /// Creates a new serializer from the given data.
    pub const fn new(marshal: M, layout: L) -> Self {
        Self { marshal, layout }
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

impl<M: Marshal, L: Layout> Serializer<M, L> {}

impl<M, L> sealed::Sealed for Serializer<M, L> {}

impl<M: Marshal, L: Layout> DynSerializer for Serializer<M, L> {
    fn marshal(&mut self) -> &mut dyn Marshal {
        self.marshal()
    }

    fn class(&mut self, v: Option<&dyn PropertyClass>) -> Result<()> {
        self.layout.class(&mut self.marshal, v)
    }

    fn container(&mut self, v: &dyn Container) -> Result<()> {
        self.layout.container(&mut self.marshal, v)
    }

    fn enum_variant(&mut self, v: &dyn Enum) -> Result<()> {
        self.layout.enum_variant(&mut self.marshal, v)
    }
}
