use std::fmt;

use crate::{access_level::AccessLevel, encoding::BinaryEncoding, protocol::Protocol};

// FIXME: Do not require heap allocation of the futures.
// https://github.com/rust-lang/rust/issues/107011

/// Generic message type that belongs to some protocol.
///
/// Messages represent structured data layouts which can be serialized.
pub trait Message: fmt::Debug + Send + BinaryEncoding + 'static {
    /// Dynamically clones this message object.
    fn dynamic_clone(&self) -> Box<dyn Message>;

    /// Gets a reference to the [`Protocol`] this message belongs to.
    fn proto(&self) -> &'static dyn Protocol;

    /// Gets the human-readable name of the message.
    fn name(&self) -> &'static str;

    /// Gets a short description of the message explaining its purpose.
    fn description(&self) -> &'static str;

    /// Gets the [`AccessLevel`] a session must meet to handle this message.
    fn access_level(&self) -> AccessLevel;

    /// Gets the unique order number of the message within a protocol.
    fn order(&self) -> u8;
}

impl Clone for Box<dyn Message> {
    fn clone(&self) -> Self {
        self.dynamic_clone()
    }
}

impl fmt::Display for dyn Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} - {}, Order {}",
            self.name(),
            self.proto(),
            self.order()
        )
    }
}
