//! Provides buffers for bit-level serialization and deserialization
//! of data.
//!
//! All the operations on types of this crate write data from LSB of
//! a byte towards the MSB. The exception are whole units of bytes,
//! which will be written in proper little-endian ordering.

#![deny(
    rust_2018_idioms,
    rustdoc::broken_intra_doc_links,
    unsafe_op_in_unsafe_fn
)]

mod reader;
pub use reader::BitReader;

mod writer;
pub use writer::{BitWriter, LengthPrefix};

mod util;
