use binrw::{
    io::{Read, Seek},
    BinRead, BinResult, Endian,
};

pub fn vec_of_bools<R: Read + Seek>(
    reader: &mut R,
    endian: Endian,
    (a,): (usize,),
) -> BinResult<Vec<bool>> {
    let mut bools = Vec::with_capacity(a);

    for _ in 0..a {
        let b = u8::read_options(reader, endian, ())?;
        bools.push(b != 0);
    }

    Ok(bools)
}

pub fn var_option_vec_vec<R: Read + Seek, T>(
    reader: &mut R,
    endian: Endian,
    (cond, lengths): (bool, &[u16]),
) -> BinResult<Option<Vec<Vec<T>>>> where for <'a> T: BinRead<Args<'a> = ()> {
    if !cond {
        return Ok(None);
    }

    let mut result = Vec::with_capacity(lengths.len());
    for &len in lengths {
        let mut value = Vec::with_capacity(len as usize);
        for _ in 0..len {
            value.push(T::read_options(reader, endian, ())?);
        }
        result.push(value);
    }

    Ok(Some(result))
}

pub fn fixed_vec_vec<R: Read + Seek, T>(
    reader: &mut R,
    endian: Endian,
    (outer, inner): (usize, usize),
) -> BinResult<Vec<Vec<T>>> where for <'a> T: BinRead<Args<'a> = ()> {
    let mut result = Vec::with_capacity(outer);
    for _ in 0..outer {
        let mut value = Vec::with_capacity(inner);
        for _ in 0..inner {
            value.push(T::read_options(reader, endian, ())?);
        }
        result.push(value);
    }

    Ok(result)
}

pub fn fixed_option_vec_vec<R: Read + Seek, T>(
    reader: &mut R,
    endian: Endian,
    (cond, outer, inner): (bool, usize, usize),
) -> BinResult<Option<Vec<Vec<T>>>> where for <'a> T: BinRead<Args<'a> = ()> {
    if !cond {
        return Ok(None);
    }

    fixed_vec_vec(reader, endian, (outer, inner)).map(Some)

}
