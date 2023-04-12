//! Arbitrary-length integer types for spanning selected bit ranges.

#[doc(inline)]
pub use ux::{i1, i2, i24, i3, i4, i5, i6, i7, u1, u2, u24, u3, u4, u5, u6, u7};

/// Extends or unextends the sign of an arbitrary bit value
/// where `$value` is the value and `$width` its bit size.
///
/// For sign extension, `$ty` should be the signed counterpart
/// to the unsigned `$value` type. The same applies in reverse
/// for sign unextension.
pub macro sign_extend($ty:ty, $value:expr, $width:expr) {{
    let value = $value;

    // Make sure `$value` and `$ty` have matching byte sizes.
    // This prevents common cases of accidental macro misuse.
    debug_assert_eq!(std::mem::size_of::<$ty>(), std::mem::size_of_val(&value));

    let shift = (<$ty>::BITS as u8) - ($width as u8);
    (value << shift) as $ty >> shift
}}
