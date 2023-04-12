use binrw::{
    io::{Read, Seek},
    BinResult, Error, Endian,
};

use super::line_string_impl;

pub fn version<R: Read + Seek>(reader: &mut R, endian: Endian, a: ()) -> BinResult<u32> {
    let (pos, version) = line_string_impl(reader, endian, a)?;

    let mut parts = version.split('.').map(|s| s.parse::<u32>());
    match (parts.next(), parts.next(), parts.next(), parts.next()) {
        (Some(Ok(a)), Some(Ok(b)), Some(Ok(c)), Some(Ok(d))) => Ok(a << 24 | b << 16 | c << 8 | d),
        _ => Err(Error::Custom {
            pos,
            err: Box::new("version string must be in format 'a.b.c.d'"),
        }),
    }
}
