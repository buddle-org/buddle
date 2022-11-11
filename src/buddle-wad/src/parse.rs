use binrw::{
    io::{Read, Seek, SeekFrom},
    BinRead, BinResult, ReadOptions, VecArgs,
};

pub fn parse_file_name<R: Read + Seek>(
    reader: &mut R,
    options: &ReadOptions,
    (len,): (usize,),
) -> BinResult<String> {
    // Read all the string bytes and chop off the null terminator.
    // This Vec is expected to end with a null terminator we don't
    // actually need, so we subtract 1 from len to skip it.
    let out = Vec::read_options(
        reader,
        options,
        VecArgs::builder().count(len.saturating_sub(1)).finalize(),
    )?;
    let new_pos = reader.seek(SeekFrom::Current(1))?;

    String::from_utf8(out).map_err(|e| binrw::Error::Custom {
        pos: new_pos - len as u64,
        err: Box::new(e),
    })
}
