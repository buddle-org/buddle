//! Commonly shared code for the Buddle project.

#![deny(rust_2018_idioms, rustdoc::broken_intra_doc_links)]
#![feature(optimize_attribute)]
#![forbid(unsafe_code)]

pub mod color;

pub mod hash;

/// Creates a bitmask spanning the `$numbits` least
/// significant bits of the inferred type.
///
/// # Panics
///
/// This macro conveniently works with all primitive
/// integer types.
///
/// However, this bears a certain risk: For an inferred
/// target integer type, when `$numbits > $int::BITS`,
/// the operation will trigger a panic/produce incorrect
/// results.
#[macro_export]
macro_rules! bitmask {
    ($numbits:expr) => {
        ((1 << ($numbits - 1)) + ((1 << ($numbits - 1)) - 1))
    };
}
