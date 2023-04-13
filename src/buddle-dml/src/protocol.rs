use std::fmt;

use bytes::Bytes;

use crate::message::Message;

/// Represents a DML protocol governing a set of messages.
pub trait Protocol: fmt::Debug + Sync + 'static {
    /// Gets the unique Service ID for this protocol.
    fn service_id(&self) -> u8;

    /// Gets the version number of the protocol specification.
    fn version(&self) -> i32;

    /// Gets the human-readable protocol type.
    fn proto_type(&self) -> &'static str;

    /// Gets the human-readable protocol description.
    fn proto_description(&self) -> &'static str;

    /// Reads a type-erased [`Message`] from this protocol from `buf`.
    ///
    /// When the given `order` number is not part of the protocol or
    /// reading from `buf` fails, [`None`] will be returned.
    ///
    /// Implementors may use the [`BinaryEncoding`][crate::BinaryEncoding]
    /// trait to read messages and supported types.
    fn read_message(&self, order: u8, buf: &mut Bytes) -> Option<Box<dyn Message>>;
}

impl fmt::Display for dyn Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Service {} ({})", self.service_id(), self.proto_type())
    }
}
