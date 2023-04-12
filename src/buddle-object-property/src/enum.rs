use std::borrow::Cow;

use crate::r#type::Type;

/// A reflected Rust enum with numbered unit variants.
///
/// This approximates C++ enums with variants represented as human-readable
/// strings or unique [`u32`] values.
pub trait Enum: Type {
    /// Gets the name of this variant as a string.
    fn variant(&self) -> Cow<'static, str>;

    /// Updates the value of `self` to the given string variant.
    ///
    /// No-op if the variant does not exist.
    fn update_variant(&mut self, variant: &str) -> bool;

    /// Gets the value of this variant.
    fn value(&self) -> u32;

    /// Updates the value of `self` to the given variant value.
    ///
    /// No-op if the variant does not exist.
    fn update_value(&mut self, value: u32) -> bool;
}
