//! Serialization support for `PropertyClass`es.
//!
//! NOTE: This is not related to the popular `serde` crate.

use crate::type_info::PropertyFlags;

mod deserializer;
pub use deserializer::*;

mod serializer;
pub use serializer::*;

mod type_tag;
pub use type_tag::*;

bitflags::bitflags! {
    /// Configuration flags to customize serialization behavior.
    #[repr(transparent)]
    pub struct SerializerFlags: u8 {
        /// [`SerializerFlags`] will be included in the output.
        const STATEFUL_FLAGS = 1 << 0;
        /// Length prefixes for strings and containers will be compressed
        /// into a compact representation for small values.
        const COMPACT_LENGTH_PREFIXES = 1 << 1;
        /// Enum variants will be serialized as a human-readable variant
        /// string rather than the integer value.
        const HUMAN_READABLE_ENUMS = 1 << 2;
        /// Whether the serialized state should be compressed with zlib.
        const COMPRESS = 1 << 3;
        /// Properties with the `DELTA_ENCODE` bit must always have their
        /// values serialized.
        const FORBID_DELTA_ENCODE = 1 << 4;
    }
}

/// Serialization configuration.
#[derive(Clone, Copy, Debug)]
pub struct Config {
    /// The [`SerializerFlags`] to apply.
    pub flags: SerializerFlags,
    /// The [`PropertyFlags`] for masking the properties of an object
    /// that should be serialized.
    pub property_mask: PropertyFlags,
    /// Whether the shallow encoding strategy is used for data.
    pub shallow: bool,
    /// A recursion limit for nested data to avoid stack overflows.
    pub recursion_limit: u8,
}

impl Config {
    /// Creates the default serializer configuration.
    ///
    /// No serializer flags, shallow mode, `TRANSMIT | PRIVILEGED_TRANSMIT`
    /// property mask, recursion limit of `128`.
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            flags: SerializerFlags::empty(),
            property_mask: PropertyFlags::TRANSMIT.union(PropertyFlags::PRIVILEGED_TRANSMIT),
            shallow: true,
            recursion_limit: u8::MAX / 2,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
