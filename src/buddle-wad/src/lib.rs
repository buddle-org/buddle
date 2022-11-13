//! Library for parsing and working with KIWAD archives.

#![deny(rust_2018_idioms, rustdoc::broken_intra_doc_links)]
#![forbid(unsafe_op_in_unsafe_fn)]

mod archive;
pub use self::archive::Archive;

pub mod crc;

mod interner;
pub use self::interner::*;

pub mod types;

mod parse;
