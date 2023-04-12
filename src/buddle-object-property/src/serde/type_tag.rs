use super::{Deserializer, Serializer};
use crate::property_class::PropertyClass;

/// Type tagging for [`PropertyClass`].
///
/// Implementations of this trait provide the functionality of reading
/// and writing unique identifiers and creating default-initialized
/// object instances from them dynamically.
pub trait TypeTag {
    /// Reads a tag from the given [`Deserializer`] and returns the
    /// corresponding [`PropertyClass`], if found.
    fn read_tag(&self, de: &mut Deserializer<'_>)
        -> anyhow::Result<Option<Box<dyn PropertyClass>>>;

    /// Reads a tag from the given [`Deserializer`] and validates it
    /// against the given `obj`.
    ///
    /// Returns an error on type mismatch.
    fn validate_tag(
        &self,
        de: &mut Deserializer<'_>,
        obj: &dyn PropertyClass,
    ) -> anyhow::Result<()>;

    /// Writes a type tag for `obj` to the given [`Serializer`].
    fn write_tag(&self, ser: &mut Serializer<'_>, obj: Option<&dyn PropertyClass>);
}
