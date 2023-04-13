use std::{any::Any, fmt, net::SocketAddr};

use crate::{access_level::AccessLevel, encoding::BinaryEncoding, protocol::Protocol};

// FIXME: Do not require heap allocation of the futures.
// https://github.com/rust-lang/rust/issues/107011

/// The future produced by [`Message::dispatch_request`].
pub type DispatchFuture<'a> = futures::future::BoxFuture<'a, anyhow::Result<()>>;

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

    /// Dispatches this message as a [`Request`][crate::Request] to its
    /// designated handler.
    ///
    /// Extra state can be passed to the handler through `extra`, which
    /// will be downcasted into the expected type.
    ///
    /// Implementors should use the [`Handler`][crate::Handler] trait to
    /// support as many variations of handler functions as possible.
    ///
    /// # Panics
    ///
    /// Panics when downcasting `extra` into the correct type fails.
    ///
    /// This issue is best avoided by only passing one uniform type for
    /// all messages to this function.
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    fn dispatch_request(
        self: Box<Self>,
        addr: SocketAddr,
        extra: &mut dyn Any,
    ) -> DispatchFuture<'_>;
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
