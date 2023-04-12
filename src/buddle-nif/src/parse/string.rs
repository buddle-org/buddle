use binrw::{
    io::{self, Read, Seek},
    BinRead, BinResult, Error, Endian,
};

pub(crate) fn line_string_impl<R: Read + Seek>(
    reader: &mut R,
    _: Endian,
    _: (),
) -> BinResult<(u64, String)> {
    // Store current stream position for potential later error handling.
    let pos = reader.stream_position()?;

    // Consume bytes from the input until we find a newline character.
    let data: Vec<u8> = reader
        .bytes()
        .filter_map(Result::ok)
        .take_while(|&b| b != b'\n')
        .collect();

    // Make sure the data is a valid Rust string and return it as such.
    String::from_utf8(data)
        .map(|s| (pos, s))
        .map_err(|e| Error::Custom {
            pos,
            err: Box::new(e),
        })
}

pub fn line_string<R: Read + Seek>(
    reader: &mut R,
    endian: Endian,
    a: (),
) -> BinResult<String> {
    line_string_impl(reader, endian, a).map(|(_, data)| data)
}

pub fn sized_string<R: Read + Seek>(
    reader: &mut R,
    endian: Endian,
    _: (),
) -> BinResult<String> {
    // Read the length prefix of the string data.
    let len = u32::read_options(reader, endian, ())? as usize;

    // Read the actual data.
    read_string_impl(reader, len)
}

pub fn sized_string_16<R: Read + Seek>(
    reader: &mut R,
    endian: Endian,
    _: (),
) -> BinResult<String> {
    // Read the length prefix of the string data.
    let len = u16::read_options(reader, endian, ())? as usize;

    // Read the actual data.
    read_string_impl(reader, len)
}

fn read_string_impl<R: Read + Seek>(reader: &mut R, len: usize) -> BinResult<String> {
    // Store current stream position for potential later error handling.
    let pos = reader.stream_position()?;

    // Read all the data we need.
    let data = reader
        .bytes()
        .take(len)
        .map(|b| b.map(|b| if b == 1 { b'_' } else { b }))
        .collect::<io::Result<Vec<u8>>>()?;

    // Perform UTF-8 validation and create a Rust string.
    String::from_utf8(data).map_err(|e| Error::Custom {
        pos,
        err: Box::new(e),
    })
}
