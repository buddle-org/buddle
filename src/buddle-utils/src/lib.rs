//! Shared code for the Buddle project.

#![deny(rust_2018_idioms, rustdoc::broken_intra_doc_links)]
#![feature(optimize_attribute)]
#![forbid(unsafe_code)]

pub use tracing;

pub mod color;

pub mod hash;

pub mod mem;
