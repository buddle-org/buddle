//! Serialization support for `PropertyClass`es.
//!
//! NOTE: This is not related to the popular `serde` crate
//! as its design choices are clashing with the requirements
//! for reflective serialization we have.

use crate::PropertyClass;

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
/// Only `Serializer`s and `Deserializer`s create instances
/// of this type when they start with a root object.
#[derive(Clone, Copy)]
pub struct Baton(pub(super) ());

/// Serializes a [`PropertyClass`] to the given
/// serializer.
///
/// This may be used for effortless and consistent
/// implementations of [`Type::serialize`][crate::Type::serialize] for
/// custom types.
pub fn serialize_class<T: PropertyClass>(
    serializer: &mut dyn ser::DynSerializer,
    v: &T,
    baton: Baton,
) -> Result<()> {
    serializer.identity(Some(v.property_list()), baton)?;
    serializer.class(v, baton)
}

/// Deserializes a [`PropertyClass`] from the given
/// deserializer.
///
/// This may be used for effortless and consistent
/// implementations of [`Type::deserialize`][crate::Type::deserialize] for
/// custom types.
pub fn deserialize_class<T: PropertyClass>(
    deserializer: &mut dyn de::DynDeserializer,
    v: &mut T,
    baton: Baton,
) -> Result<()> {
    let list = deserializer.identity(baton)?;
    let list = list.as_ref();

    if list.map(|l| l.is::<T>()).unwrap_or(false) {
        deserializer.class(v, baton)
    } else {
        Err(Error::custom(format_args!(
            "Type mismatch - {} serialized, {} instantiated",
            list.map(|l| l.type_name()).unwrap_or("nothing"),
            v.type_info().type_name()
        )))
    }
}
