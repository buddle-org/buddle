use anyhow::bail;
use bitvec::{domain::Domain, prelude::*};
use buddle_utils::mem::align_down;
use funty::Integral;

#[cold]
#[inline(never)]
fn premature_eof() -> anyhow::Error {
    anyhow::anyhow!("premature EOF while trying to read data")
}

macro_rules! impl_read_literal {
    ($($(#[$doc:meta])* $read_fn:ident() -> $ty:ty),* $(,)?) => {
        $(
            $(#[$doc])*
            #[inline]
            pub fn $read_fn(&mut self) -> anyhow::Result<$ty> {
                self.realign_to_byte();
                self.read_bitint::<$ty>(<$ty>::BITS as _)
            }
        )*
    };
}

/// A buffer which supports bit-based deserialization of data.
///
/// Quantities of multiple bytes (except byte slices) are always read
/// in little-endian byte ordering. Individual bit reading starts with
/// the LSB of the byte, working towards the MSB.
#[derive(Clone, Debug, Default)]
pub struct BitReader<'de> {
    inner: &'de BitSlice<u8, Lsb0>,
}

impl<'de> BitReader<'de> {
    /// Creates a new reader over the given byte slice.
    #[inline]
    pub fn new(buf: &'de [u8]) -> Self {
        Self {
            inner: buf.view_bits(),
        }
    }

    /// Returns the number of bits remaining in the buffer.
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Indicates whether the buffer is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Reads a single bit from the buffer, if possible.
    #[inline]
    pub fn read_bit(&mut self) -> anyhow::Result<bool> {
        let (first, remainder) = self.inner.split_first().ok_or_else(premature_eof)?;
        self.inner = remainder;

        Ok(*first)
    }

    /// Reads `n` bits from the buffer, if possible.
    #[inline]
    pub fn read_bits(&mut self, n: usize) -> anyhow::Result<&'de BitSlice<u8, Lsb0>> {
        if n <= self.inner.len() {
            // SAFETY: We checked that `n` is in bounds.
            let (chunk, remainder) = unsafe { self.inner.split_at_unchecked(n) };
            self.inner = remainder;

            Ok(chunk)
        } else {
            Err(premature_eof())
        }
    }

    /// Reads a given number of bits from the buffer into an integer,
    /// if possible.
    #[inline]
    pub fn read_bitint<I: Integral>(&mut self, bits: usize) -> anyhow::Result<I> {
        if 0 < bits && bits <= I::BITS as _ {
            self.read_bits(bits).map(|bs| bs.load_le())
        } else {
            bail!("requested bits overflow capacity of target type");
        }
    }

    #[inline]
    fn realign_to_byte(&mut self) {
        let pad_bits = self.inner.len() - align_down(self.inner.len(), u8::BITS as _);
        // SAFETY: `pad_bits` is always <= `self.inner.len()`.
        self.inner = unsafe { self.inner.split_at_unchecked(pad_bits).1 };
    }

    /// Reads `n` bytes from the buffer, if possible.
    ///
    /// This will force-align the buffer to full byte boundaries before
    /// reading; effectively discarding the remaining bits until then.
    #[inline]
    pub fn read_bytes(&mut self, n: usize) -> anyhow::Result<&'de [u8]> {
        self.realign_to_byte();
        self.read_bits(n * u8::BITS as usize)
            .map(|bs| match bs.domain() {
                // SAFETY: Since we're starting at byte boundary and only reading
                // full bytes, we don't have to consider any partial elements.
                Domain::Region { body, .. } => body,
                Domain::Enclave(..) => unsafe { std::hint::unreachable_unchecked() },
            })
    }

    /// Reads a [`bool`] value from the buffer, if possible.
    ///
    /// Booleans are represented as individual bits and do not force a
    /// realign to full byte boundaries.
    #[inline]
    pub fn bool(&mut self) -> anyhow::Result<bool> {
        self.read_bit()
    }

    // fn $read_fn(&mut self) -> Option<$ty>
    impl_read_literal! {
        /// Reads a [`u8`] value from the buffer, if possible.
        ///
        /// This will force-align the buffer to full byte boundaries before
        /// reading; effectively discarding the remaining bits until then.
        u8() -> u8,
        /// Reads a [`i8`] value from the buffer, if possible.
        ///
        /// This will force-align the buffer to full byte boundaries before
        /// reading; effectively discarding the remaining bits until then.
        i8() -> i8,

        /// Reads a [`u16`] value from the buffer, if possible.
        ///
        /// This will force-align the buffer to full byte boundaries before
        /// reading; effectively discarding the remaining bits until then.
        u16() -> u16,
        /// Reads a [`i16`] value from the buffer, if possible.
        ///
        /// This will force-align the buffer to full byte boundaries before
        /// reading; effectively discarding the remaining bits until then.
        i16() -> i16,

        /// Reads a [`u32`] value from the buffer, if possible.
        ///
        /// This will force-align the buffer to full byte boundaries before
        /// reading; effectively discarding the remaining bits until then.
        u32() -> u32,
        /// Reads a [`i32`] value from the buffer, if possible.
        ///
        /// This will force-align the buffer to full byte boundaries before
        /// reading; effectively discarding the remaining bits until then.
        i32() -> i32,

        /// Reads a [`u64`] value from the buffer, if possible.
        ///
        /// This will force-align the buffer to full byte boundaries before
        /// reading; effectively discarding the remaining bits until then.
        u64() -> u64,
    }

    /// Reads a [`f32`] value from the buffer, if possible.
    ///
    /// This will force-align the buffer to full byte boundaries before
    /// reading; effectively discarding the remaining bits until then.
    #[inline]
    pub fn f32(&mut self) -> anyhow::Result<f32> {
        self.u32().map(f32::from_bits)
    }

    /// Reads a [`f64`] value from the buffer, if possible.
    ///
    /// This will force-align the buffer to full byte boundaries before
    /// reading; effectively discarding the remaining bits until then.
    #[inline]
    pub fn f64(&mut self) -> anyhow::Result<f64> {
        self.u64().map(f64::from_bits)
    }
}
