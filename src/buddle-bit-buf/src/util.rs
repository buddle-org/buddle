use funty::Integral;

/// Defines casting behavior from one integer type to another.
pub trait IntCast<T: Integral> {
    /// Casts `self` to target type `T`.
    ///
    /// When `self` is too large to fit `T`, bits will be chopped off.
    /// Expect this to behave like an `as` cast.
    fn cast_as(self) -> T;
}

macro_rules! impl_intcast_from_usize {
    ($($ty:ty),* $(,)*) => {
        $(
            impl IntCast<$ty> for usize {
                #[inline(always)]
                fn cast_as(self) -> $ty {
                    self as $ty
                }
            }
        )*
    };
}

impl_intcast_from_usize! {
    i8, i16, i32, i64, isize,
    u8, u16, u32, u64, usize,
}
