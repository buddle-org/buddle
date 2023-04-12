use binrw::{
    io::{Read, Seek},
    BinResult, Endian, Error,
};

use crate::{compounds::Header, objects::NiObject};

pub fn blocks<R: Read + Seek>(
    reader: &mut R,
    endian: Endian,
    (header,): (&Header,),
) -> BinResult<Vec<NiObject>> {

    let mut blocks = Vec::with_capacity(header.num_blocks as usize);

    for idx in &header.block_type_index {
        match header.block_types.get(idx.0 as usize) {
            Some(block) => blocks.push(NiObject::read_options(
                reader,
                endian,
                &block.data,
                header.version,
            )?),
            None => {
                return Err(Error::Custom {
                    pos: reader.stream_position()?,
                    err: Box::new("referenced block does not exist in header"),
                });
            }
        }
    }

    Ok(blocks)
}
