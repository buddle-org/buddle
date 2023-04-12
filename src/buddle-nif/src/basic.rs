use std::marker::PhantomData;

use binrw::{
    io::{Read, Seek},
    BinRead, BinResult, Endian,
};

use crate::{objects::NiObject, parse};

/// A 16-bit integer, which is used in the header to refer to a
/// particular object type in a object type string array.
#[derive(Clone, Copy, Debug, Default, PartialEq, BinRead)]
pub struct BlockTypeIndex(pub u16);

/// A 32-bit integer that stores the version in hexadecimal format.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, BinRead)]
#[br(map = Self::from_u32)]
pub struct FileVersion(pub u8, pub u8, pub u8, pub u8);

impl FileVersion {
    pub fn from_u32(value: u32) -> Self {
        let major = ((value >> 24) & 0xFF) as u8;
        let minor = ((value >> 16) & 0xFF) as u8;
        let micro = ((value >> 8) & 0xFF) as u8;
        let patch = (value & 0xFF) as u8;

        Self(major, minor, micro, patch)
    }
}

/// A 16-bit floating point number.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct f16(pub crate::f16);

impl BinRead for f16 {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        a: Self::Args<'_>,
    ) -> BinResult<Self> {
        u16::read_options(reader, endian, a).map(|v| Self(crate::f16::from_bits(v)))
    }
}

/// A variable length string that encodes a version.
#[derive(Clone, Debug, Default, PartialEq, BinRead)]
#[br(magic = b"Gamebryo File Format, Version ")]
pub struct HeaderString(#[br(parse_with = parse::version)] pub u32);

/// A variable length string that ends with a newline character.
#[derive(Clone, Debug, Default, PartialEq, BinRead)]
pub struct LineString(#[br(parse_with = parse::line_string)] pub String);

/// A signed 32-bit integer, used to refer to another object.
#[derive(Clone, Copy, Debug, Default, PartialEq, BinRead)]
pub struct Ptr<T: 'static>(pub u32, PhantomData<T>);

impl<T: 'static> Ptr<T> {
    /// Gets the referenced type as a raw [`NiObject`] out of
    /// the full block list.
    pub fn get<'b>(&self, blocks: &'b [NiObject]) -> &'b NiObject {
        &blocks[self.0 as usize]
    }
}

/// A signed 32-bit integer, used to refer to another object.
#[derive(Clone, Copy, Debug, Default, PartialEq, BinRead)]
pub struct Ref<T: 'static>(pub i32, PhantomData<T>);

impl<T: 'static> Ref<T> {
    /// Gets the referenced type as a raw [`NiObject`] out of
    /// the full block list.
    pub fn get<'b>(&self, blocks: &'b [NiObject]) -> Option<&'b NiObject> {
        (self.0 >= 0).then(|| &blocks[self.0 as usize])
    }
}

/// A 32-bit unsigned integer, used to refer to strings.
#[derive(Clone, Copy, Debug, Default, PartialEq, BinRead)]
pub struct StringOffset(pub u32);

/// A 32-bit unsigned integer, used to refer to strings.
#[derive(Clone, Copy, Debug, Default, PartialEq, BinRead)]
pub struct NiFixedString(pub u32);
