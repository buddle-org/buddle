use std::mem::size_of;

use buddle_bytes_ext::CheckedBuf;
use bytes::{Buf, BufMut, Bytes, BytesMut};

/// Defines binary encoding and decoding for DML messages and supported
/// primitive types.
pub trait BinaryEncoding {
    /// Gets the binary size of this type in bytes.
    fn binary_size(&self) -> usize;

    /// Tries to read a `Self` value out of the given `source`.
    fn read(source: &mut Bytes) -> Option<Self>
    where
        Self: Sized;

    /// Writes `self` to the given `dest` buffer.
    ///
    /// # Panics
    ///
    /// This may panic if `dest` lacks capacity to store [`BinaryEncoding::binary_size`]
    /// more bytes.
    fn write(&self, dest: &mut BytesMut);
}

macro_rules! impl_primitive_encoding {
    ($(($ty:ty, $read:ident, $write:ident)),* $(,)?) => {
        $(
            impl BinaryEncoding for $ty {
                fn binary_size(&self) -> usize {
                    size_of::<$ty>()
                }

                fn read(source: &mut Bytes) -> Option<Self>
                where
                    Self: Sized,
                {
                    source.$read()
                }

                fn write(&self, dest: &mut BytesMut) {
                    dest.$write(*self)
                }
            }
        )*
    };
}

impl_primitive_encoding! {
    (u8, try_get_u8, put_u8), // UBYT
    (i8, try_get_i8, put_i8), // BYT
    (i16, try_get_i16_le, put_i16_le), // SHRT
    (i32, try_get_i32_le, put_i32_le), // INT
    (u32, try_get_u32_le, put_u32_le), // UINT
    (u64, try_get_u64_le, put_u64_le), // GID
    (f32, try_get_f32_le, put_f32_le), // FLT
    (f64, try_get_f64_le, put_f64_le), // DBL
}

impl BinaryEncoding for Vec<u8> {
    fn binary_size(&self) -> usize {
        size_of::<u16>() + self.len()
    }

    fn read(source: &mut Bytes) -> Option<Self>
    where
        Self: Sized,
    {
        let len = source.try_get_u16_le()? as usize;
        (source.remaining() >= len).then(|| {
            let mut str = vec![0; len];
            source.copy_to_slice(&mut str);
            str
        })
    }

    fn write(&self, dest: &mut BytesMut) {
        dest.put_u16_le(self.len().try_into().expect("string too long to encode"));
        dest.put_slice(self);
    }
}

const WCHAR_SIZE: usize = size_of::<u16>();

impl BinaryEncoding for Vec<u16> {
    fn binary_size(&self) -> usize {
        size_of::<u16>() + self.len()
    }

    fn read(source: &mut Bytes) -> Option<Self>
    where
        Self: Sized,
    {
        let len = source.try_get_u16_le()? as usize;
        (source.remaining() >= len * WCHAR_SIZE).then(|| {
            let mut wstr = Vec::with_capacity(len);
            (0..len).for_each(|_| wstr.push(source.get_u16_le()));
            wstr
        })
    }

    fn write(&self, dest: &mut BytesMut) {
        dest.put_u16_le(self.len().try_into().expect("string too large to encode"));
        self.iter().for_each(|&wchar| dest.put_u16_le(wchar));
    }
}
