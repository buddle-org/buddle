//! Compatibility layer for sharing serialized types
//! across the C++ language border.

mod ptr;
pub use self::ptr::*;

mod strings;
pub use self::strings::*;
