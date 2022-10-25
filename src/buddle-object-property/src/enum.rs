use crate::Type;

/// A reflected Rust enum with unit variants.
///
/// This approximates C++ enums with variants represented
/// as human-readable strings or unique [`u32`] values.
pub trait Enum: Type {
    /// Gets the name of this variant as a string.
    fn variant(&self) -> &'static str;

    /// Gets a variant from its string representation.
    ///
    /// Returns [`None`] if `variant` does not map to
    /// any defined variant.
    fn from_variant(variant: &str) -> Option<Self>
    where
        Self: Sized;

    /// Gets the value of this variant.
    fn value(&self) -> u32;

    /// Gets a variant from its numeric representation.
    ///
    /// Returns [`None`] if `value` does not map to any
    /// defined variant.
    fn from_value(value: u32) -> Option<Self>
    where
        Self: Sized;
}
