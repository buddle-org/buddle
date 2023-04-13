//! A Rust implementation of KingsIsle's Data Management Layer.
//!
//! This crate provides traits for defining DML services and messages and
//! allow for easy serialization and deserialization of native Rust types.

#![deny(rust_2018_idioms, rustdoc::broken_intra_doc_links)]
#![forbid(unsafe_code)]

pub use bytes;

mod access_level;
pub use access_level::AccessLevel;

mod encoding;
pub use encoding::BinaryEncoding;

mod message;
pub use message::{DispatchFuture, Message};

mod protocol;
pub use protocol::Protocol;
