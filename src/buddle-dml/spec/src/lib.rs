//! A crate to parse Data Management Layer protocol specifications in XML format.

#![deny(rust_2018_idioms, rustdoc::broken_intra_doc_links)]
#![forbid(unsafe_code)]

use roxmltree::Document;

mod field;
pub use field::Field;

mod protocol;
pub use protocol::Protocol;

mod record;
pub use record::Record;

/// Parses a DML protocol from its XML description given as a string.
pub fn parse_protocol(input: &str) -> anyhow::Result<Protocol> {
    let proto = Document::parse(input)?;
    Protocol::parse(proto)
}
