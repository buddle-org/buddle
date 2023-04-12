use std::marker::PhantomData;

use bitvec::prelude::*;
use buddle_utils::mem::align_up;
use funty::Integral;

use crate::util::IntCast;

macro_rules! write_bytes_to_bitslice {
    ($bs:ident, $buf:expr) => {
        // SAFETY: The iterator is consumed while only ever holding
        // onto one `slot` at the same time.
        unsafe { $bs.chunks_exact_mut(u8::BITS as _).remove_alias() }
            .zip($buf)
            .for_each(|(slot, byte)| slot.store_be(byte));
    };
}

macro_rules! impl_write_literal {
    ($($(#[$doc:meta])* $write_fn:ident($ty:ty)),* $(,)?) => {
        $(
            $(#[$doc])*
            #[inline]
            pub fn $write_fn(&mut self, v: $ty) {
                self.realign_to_byte();

                let len = self.inner.len();
                self.inner.resize(len + <$ty>::BITS as usize, false);

                // SAFETY: `len` was the former end of the buffer before reallocation;
                // therefore it denotes where the newly allocated memory starts.
                let bs = unsafe { self.inner.get_unchecked_mut(len..) };
                write_bytes_to_bitslice!(bs, v.to_le_bytes());
            }
        )*
    };
}

/// A reserved length prefix to be committed to the buffer later.
pub struct LengthPrefix<I> {
    start: usize,
    pos: usize,
    _i: PhantomData<I>,
}

/// A buffer which supports bit-based serialization of data.
///
/// Quantities of multiple bytes (except byte slices) are always written
/// in little-endian byte ordering. Individual bit writing starts with
/// the LSB of the byte, working towards the MSB.
#[derive(Clone, Debug, Default)]
pub struct BitWriter {
    inner: BitVec<u8, Lsb0>,
}

impl BitWriter {
    /// Creates a new, empty [`BitWriter`].
    pub fn new() -> Self {
        Self {
            inner: BitVec::new(),
        }
    }

    /// Returns the number of bits in the buffer.
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Indicates whether the buffer is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Gets a view of the buffer's storage as a byte slice.
    #[inline]
    pub fn view(&self) -> &[u8] {
        self.inner.as_raw_slice()
    }

    /// Consumes the [`BitWriter`] and returns a [`Vec`] of bytes.
    #[inline]
    pub fn into_vec(self) -> Vec<u8> {
        self.inner.into_vec()
    }

    /// Writes a single bit to the buffer.
    #[inline]
    pub fn write_bit(&mut self, b: bool) {
        self.inner.push(b);
    }

    /// Writes all bits in `buf` to the buffer.
    #[inline]
    pub fn write_bits(&mut self, buf: &BitSlice<u8, Lsb0>) {
        self.inner.extend_from_bitslice(buf);
    }

    /// Writes a given number of bits from `value` to the buffer.
    #[inline]
    pub fn write_bitint<I: Integral>(&mut self, value: I, bits: usize) {
        let len = self.inner.len();
        self.inner.resize(len + bits, false);

        let bs = unsafe { self.inner.get_unchecked_mut(len..) };
        bs.store_le(value);
    }

    #[inline]
    fn realign_to_byte(&mut self) {
        let pad_bits = align_up(self.inner.len(), u8::BITS as _) - self.inner.len();
        self.inner.resize(self.inner.len() + pad_bits, false);
    }

    /// Writes the bytes in `buf` to the buffer.
    ///
    /// This will force-align the buffer to full byte boundaries before
    /// writing; effectively filling remaining bits with zeroes.
    #[inline]
    pub fn write_bytes(&mut self, buf: &[u8]) {
        self.realign_to_byte();

        let len = self.inner.len();
        self.inner.resize(len + buf.len() * 8, false);

        // SAFETY: `len` was the former end of the buffer before reallocation,
        // therefore it denotes where the newly allocated memory starts.
        let bs = unsafe { self.inner.get_unchecked_mut(len..) };
        write_bytes_to_bitslice!(bs, buf.iter().copied());
    }

    /// Reserves a length prefix of a given literal type `I` in the buffer.
    ///
    /// After calling this method, the bits written to the buffer will be
    /// counted until [`BitWriter::place_length_prefix`] is called.
    pub fn reserve_length_prefix<I: Integral>(&mut self) -> LengthPrefix<I>
    where
        usize: IntCast<I>,
        I::Bytes: IntoIterator<Item = u8>,
    {
        // Back up the current bit position for calculation of the length.
        // We also count padding bits inserted towards the length value.
        let start = self.len();

        // Pad to full bytes to prepare for writing an `I` value.
        self.realign_to_byte();

        // Reserve a placeholder for the length prefix.
        let pos = self.len();
        self.inner.resize(pos + I::BITS as usize, false);

        LengthPrefix {
            start,
            pos,
            _i: PhantomData,
        }
    }

    /// Applies a previously reserved [`LengthPrefix`] by storing the
    /// amount of newly written bits.
    pub fn write_length_prefix<I: Integral>(&mut self, len: LengthPrefix<I>)
    where
        usize: IntCast<I>,
        I::Bytes: IntoIterator<Item = u8>,
    {
        let LengthPrefix { start, pos, .. } = len;

        // Calculate the length in bits.
        let prefix: I = (self.inner.len() - start).cast_as();

        // SAFETY: Only we can issue valid `LengthPrefix`es, so `pos`
        // can be trusted to point at valid memory.
        let bs = unsafe { self.inner.get_unchecked_mut(pos..) };
        write_bytes_to_bitslice!(bs, prefix.to_le_bytes());
    }

    /// Writes a given [`bool`] value to the buffer.
    ///
    /// Booleans are represented as single bits and do not force a realign
    /// to full byte boundaries.
    #[inline]
    pub fn bool(&mut self, v: bool) {
        self.write_bit(v);
    }

    impl_write_literal! {
        /// Writes a given [`u8`] value to the buffer.
        ///
        /// This will force-align the buffer to full byte boundaries before
        /// writing; effectively filling remaining bits with zeroes.
        u8(u8),
        /// Writes a given [`i8`] value to the buffer.
        ///
        /// This will force-align the buffer to full byte boundaries before
        /// writing; effectively filling remaining bits with zeroes.
        i8(i8),

        /// Writes a given [`u16`] value to the buffer.
        ///
        /// This will force-align the buffer to full byte boundaries before
        /// writing; effectively filling remaining bits with zeroes.
        u16(u16),
        /// Writes a given [`i16`] value to the buffer.
        ///
        /// This will force-align the buffer to full byte boundaries before
        /// writing; effectively filling remaining bits with zeroes.
        i16(i16),

        /// Writes a given [`u32`] value to the buffer.
        ///
        /// This will force-align the buffer to full byte boundaries before
        /// writing; effectively filling remaining bits with zeroes.
        u32(u32),
        /// Writes a given [`i32`] value to the buffer.
        ///
        /// This will force-align the buffer to full byte boundaries before
        /// writing; effectively filling remaining bits with zeroes.
        i32(i32),

        /// Writes a given [`u64`] value to the buffer.
        ///
        /// This will force-align the buffer to full byte boundaries before
        /// writing; effectively filling remaining bits with zeroes.
        u64(u64),
        /// Writes a given [`i64`] value to the buffer.
        ///
        /// This will force-align the buffer to full byte boundaries before
        /// writing; effectively filling remaining bits with zeroes.
        i64(i64),
    }

    /// Writes the bits of a given [`f32`] value to the buffer.
    ///
    /// This will force-align the buffer to full byte boundaries before
    /// writing; effectively filling remaining bits with zeroes.
    #[inline]
    pub fn f32(&mut self, v: f32) {
        self.u32(v.to_bits());
    }

    /// Writes the bits of a given [`f64`] value to the buffer.
    ///
    /// This will force-align the buffer to full byte boundaries before
    /// writing; effectively filling remaining bits with zeroes.
    #[inline]
    pub fn f64(&mut self, v: f64) {
        self.u64(v.to_bits());
    }
}
