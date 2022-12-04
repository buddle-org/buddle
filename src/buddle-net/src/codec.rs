//!

mod crypto;
pub use crypto::EncryptionMode;

mod decoder;
mod encoder;

pub(super) const FOOD: u16 = 0xF00D;

#[inline(always)]
pub(super) const fn is_large_frame(size: usize) -> bool {
    size > i16::MAX as _
}

/// A tokio-based codec for reading and writing [`Frame`]s
/// to network sockets.
///
/// [`Frame`]: crate::frame::Frame
pub struct Codec {
    mode: EncryptionMode,
}

impl Codec {
    /// Creates a new codec with the given [`EncryptionMode`].
    pub const fn new(mode: EncryptionMode) -> Self {
        Self { mode }
    }
}
