//! This crate extends the [`bytes`] crate with fallible read
//! operations on [`bytes::Buf`]s.

#![deny(rust_2018_idioms, rustdoc::broken_intra_doc_links)]
#![forbid(unsafe_code)]

use std::mem::size_of;

use bytes::Buf;

macro_rules! read_checked {
    ($source:ident.$fn:ident() -> $ty:ty) => {
        ($source.remaining() >= size_of::<$ty>()).then(|| $source.$fn())
    };
}

/// Provides fallible read operations for arbitrary [`Buf`]s.
pub trait CheckedBuf: Buf {
    /// Attempts to get an [`i8`] from `self`.
    fn try_get_i8(&mut self) -> Option<i8> {
        read_checked!(self.get_i8() -> i8)
    }

    /// Attempts to get an [`u8`] from `self`.
    fn try_get_u8(&mut self) -> Option<u8> {
        read_checked!(self.get_u8() -> u8)
    }

    /// Attempts to get an [`i16`] from `self` in big-endian byte order.
    fn try_get_i16(&mut self) -> Option<i16> {
        read_checked!(self.get_i16() -> i16)
    }

    /// Attempts to get an [`i16`] from `self` in little-endian byte order.
    fn try_get_i16_le(&mut self) -> Option<i16> {
        read_checked!(self.get_i16_le() -> i16)
    }

    /// Attempts to get an [`u16`] from `self` in big-endian byte order.
    fn try_get_u16(&mut self) -> Option<u16> {
        read_checked!(self.get_u16() -> u16)
    }

    /// Attempts to get an [`u16`] from `self` in little-endian byte order.
    fn try_get_u16_le(&mut self) -> Option<u16> {
        read_checked!(self.get_u16_le() -> u16)
    }

    /// Attempts to get an [`i32`] from `self` in big-endian byte order.
    fn try_get_i32(&mut self) -> Option<i32> {
        read_checked!(self.get_i32() -> i32)
    }

    /// Attempts to get an [`i32`] from `self` in little-endian byte order.
    fn try_get_i32_le(&mut self) -> Option<i32> {
        read_checked!(self.get_i32_le() -> i32)
    }

    /// Attempts to get an [`u32`] from `self` in big-endian byte order.
    fn try_get_u32(&mut self) -> Option<u32> {
        read_checked!(self.get_u32() -> u32)
    }

    /// Attempts to get an [`u32`] from `self` in little-endian byte order.
    fn try_get_u32_le(&mut self) -> Option<u32> {
        read_checked!(self.get_u32_le() -> u32)
    }

    /// Attempts to get an [`i64`] from `self` in big-endian byte order.
    fn try_get_i64(&mut self) -> Option<i64> {
        read_checked!(self.get_i64() -> i64)
    }

    /// Attempts to get an [`i64`] from `self` in little-endian byte order.
    fn try_get_i64_le(&mut self) -> Option<i64> {
        read_checked!(self.get_i64_le() -> i64)
    }

    /// Attempts to get an [`u64`] from `self` in big-endian byte order.
    fn try_get_u64(&mut self) -> Option<u64> {
        read_checked!(self.get_u64() -> u64)
    }

    /// Attempts to get an [`u64`] from `self` in little-endian byte order.
    fn try_get_u64_le(&mut self) -> Option<u64> {
        read_checked!(self.get_u64_le() -> u64)
    }

    /// Attempts to get an [`f32`] from `self` in big-endian byte order.
    fn try_get_f32(&mut self) -> Option<f32> {
        read_checked!(self.get_f32() -> f32)
    }

    /// Attempts to get an [`f32`] from `self` in little-endian byte order.
    fn try_get_f32_le(&mut self) -> Option<f32> {
        read_checked!(self.get_f32_le() -> f32)
    }

    /// Attempts to get an [`f64`] from `self` in big-endian byte order.
    fn try_get_f64(&mut self) -> Option<f64> {
        read_checked!(self.get_f64() -> f64)
    }

    /// Attempts to get an [`f64`] from `self` in little-endian byte order.
    fn try_get_f64_le(&mut self) -> Option<f64> {
        read_checked!(self.get_f64_le() -> f64)
    }
}

impl<B: Buf> CheckedBuf for B {}
impl CheckedBuf for dyn Buf {}
