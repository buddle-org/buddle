//! Result and error for serialization and deserializaton.

use core::fmt::{self, Display};

/// A [`Result`][std::result::Result] produced by this
/// module's operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Error when serialization or deserialization through a
/// trait object fails.
#[derive(Debug)]
pub struct Error {
    msg: String,
}

impl Error {
    /// Creates a new error from a custom message.
    pub fn custom<T: Display>(msg: T) -> Self {
        Self {
            msg: msg.to_string(),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.msg.fmt(f)
    }
}

impl std::error::Error for Error {}
