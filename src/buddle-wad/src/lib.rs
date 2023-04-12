//! Library for parsing and interacting with KIWAD archives.
//!
//! The core of this library is the [`Archive`] type which provides
//! facilities to load KIWAD archive files into memory and grants
//! access to raw file data by name.
//!
//! Since most of the files are compressed, the [`Interner`] type
//! provides a means to access uncompressed contents of archive
//! files on demand.

#![deny(
    rust_2018_idioms,
    rustdoc::broken_intra_doc_links,
    unsafe_op_in_unsafe_fn
)]

mod archive;
pub use archive::Archive;

pub mod crc;

mod interner;
pub use interner::*;

pub mod types;
