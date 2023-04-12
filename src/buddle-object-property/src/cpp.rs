//! Compatibility layer for sharing serialized types across the
//! C++ language border.

use crate::type_info::TypeInfo;

mod ptr;
pub use ptr::*;

mod strings;
pub use strings::*;

/// A [`TypeInfo`] for alternative representation of [`i32`] values
/// as C `long`s.
///
/// NOTE: This exists because Windows defines both ints and longs
/// to be 32-bit integers.
pub const LONG: TypeInfo = TypeInfo::leaf::<i32>(Some("long"));

/// A [`TypeInfo`] for alternative representation of [`u32`] values
/// as C `unsigned long`s.
///
/// NOTE: This exists because Windows defines both ints and longs
/// to be 32-bit integers.
pub const ULONG: TypeInfo = TypeInfo::leaf::<u32>(Some("unsigned long"));
