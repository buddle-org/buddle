//! Serialization support for [`PropertyClass`]es.
//!
//! NOTE: This is not related to the popular `serde` crate
//! as its design choices are clashing with the requirements
//! for reflective serialization we have.

mod result;
pub use self::result::*;

mod ser;
pub use self::ser::*;
