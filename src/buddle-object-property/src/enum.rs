use std::borrow::Cow;

use crate::Type;

/// A reflected Rust enum with unit variants.
///
/// This approximates C++ enums with variants represented
/// as human-readable strings or unique [`u32`] values.
pub trait Enum: Type {
    /// Gets the name of this variant as a string.
    fn variant(&self) -> Cow<'static, str>;

    /// Updates the value of `self` to the new variant
    /// given as a string.
    ///
    /// A [`bool`] indicates whether a variant matching
    /// the string exists. If this returns `false`, self
    /// was not modified.
    fn update_variant(&mut self, variant: &str) -> bool;

    /// Gets a variant from its string representation.
    ///
    /// Returns [`None`] if `variant` does not map to
    /// any defined variant.
    fn from_variant(variant: &str) -> Option<Self>
    where
        Self: Sized;

    /// Gets the value of this variant.
    fn value(&self) -> u32;

    /// Updates the value of `self` to the new variant
    /// given as a an integer.
    ///
    /// A [`bool`] indicates whether a variant matching
    /// the string exists. If this returns `false`, self
    /// was not modified.
    fn update_value(&mut self, value: u32) -> bool;

    /// Gets a variant from its numeric representation.
    ///
    /// Returns [`None`] if `value` does not map to any
    /// defined variant.
    fn from_value(value: u32) -> Option<Self>
    where
        Self: Sized;
}
