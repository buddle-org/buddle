//! Serialization support for [`PropertyClass`]es.
//!
//! NOTE: This is not related to the popular `serde` crate
//! as its design choices are clashing with the requirements
//! for reflective serialization we have.

pub mod de;

mod result;
pub use self::result::*;

pub mod ser;

/// A baton that is passed around during serialization.
///
/// This effectively does nothing but prevent users from
/// calling methods they are not supposed to call out of
/// context.
///
/// Only [`Serializer`]s and [`Deserializer`]s create
/// instances of this type when they start with a root
/// object.
#[derive(Clone, Copy)]
pub struct Baton(pub(super) ());
