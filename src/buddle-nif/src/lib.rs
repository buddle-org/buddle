//! NetImmerse File (NIF) parsing library for use with
//! Wizard101 assets.
//!
//! In particular, this crate targets versions `20.2.0.7`,
//! `20.2.0.8`, and `20.6.0.0` of the format.

#![feature(slice_take)]

pub mod basic;
use self::basic::FileVersion;

pub mod bitflags;

pub mod bitfields;

pub mod compounds;
use self::compounds::{Footer, Header};

pub mod enums;

pub mod objects;
use self::objects::NiObject;

mod parse;

use binrw::BinResult;
use binrw::{
    io::{Read, Seek},
    BinRead, BinReaderExt,
};
pub use half::f16;

const SUPPORTED_VERSIONS: [FileVersion; 5] = [
    FileVersion(20, 1, 0, 3),
    FileVersion(20, 2, 0, 7),
    FileVersion(20, 2, 0, 8),
    FileVersion(20, 3, 0, 9),
    FileVersion(20, 6, 0, 0),
];

/// Representation of a NIF file in all its glory.
#[derive(Clone, Debug, PartialEq, BinRead)]
pub struct Nif {
    /// The NIF [`Header`], directly deserialized from the
    /// input source.
    #[br(assert(SUPPORTED_VERSIONS.contains(&header.version)))]
    pub header: Header,
    /// Every [`NiObject`] block encoded in the file, directly
    /// deserialized from the input source.
    #[br(args(&header), parse_with = parse::blocks)]
    pub blocks: Vec<NiObject>,
    /// The terminating NIF [`Footer`], directly deserialized
    /// from the input source.
    pub footer: Footer,
}

impl Nif {
    /// Attempts to parse a NIF file from a given input source.
    pub fn parse<R: Read + Seek>(reader: &mut R) -> BinResult<Self> {
        reader.read_le()
    }

    /// Gets a list of the root [`NiObject`] references for this
    /// data tree.
    pub fn root_objects(&self) -> Vec<&NiObject> {
        self.footer
            .roots
            .iter()
            .filter_map(|r| r.get(&self.blocks))
            .collect()
    }

    /// Gets a list of [`NiObject`] references to child nodes
    /// referenced by the given object, if any.
    pub fn children_for(&self, obj: &NiObject) -> Option<Vec<&NiObject>> {
        obj.children(&self.blocks)
    }

    /// Gets a list of [`NiObject`] properties referenced by
    /// the given object, if any.
    pub fn properties_for(&self, obj: &NiObject) -> Option<Vec<&NiObject>> {
        obj.properties(&self.blocks)
    }

    /// Gets a list of [`NiObject`] extra data referenced by
    /// the given object, if any.
    pub fn extra_data_for(&self, obj: &NiObject) -> Option<Vec<&NiObject>> {
        obj.extra_data(&self.blocks)
    }
}
