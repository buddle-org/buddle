//! Library for parsing and working with KIWAD archives.

#![deny(rust_2018_idioms, rustdoc::broken_intra_doc_links)]
#![forbid(unsafe_op_in_unsafe_fn)]

pub mod archive;

pub mod crc;

pub mod interner;

pub mod types;

mod parse;
