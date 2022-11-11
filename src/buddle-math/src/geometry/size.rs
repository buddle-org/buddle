use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

/// A two-dimensional size defined by its width and height.
// https://github.com/palestar/medusa/blob/develop/Standard/Size.h
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Size<T> {
    /// The width of the size.
    pub width: T,
    /// The height of the size.
    pub height: T,
}

impl<T> Size<T> {
    /// Creates a new size given the width and height.
    pub const fn new(width: T, height: T) -> Self {
        Self { width, height }
    }
}

impl<T: Add<Output = T>> Add for Size<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            width: self.width + rhs.width,
            height: self.height + rhs.height,
        }
    }
}

impl<T: AddAssign> AddAssign for Size<T> {
    fn add_assign(&mut self, rhs: Self) {
        self.width += rhs.width;
        self.height += rhs.height;
    }
}

impl<T: Sub<Output = T>> Sub for Size<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            width: self.width - rhs.width,
            height: self.height - rhs.height,
        }
    }
}

impl<T: SubAssign> SubAssign for Size<T> {
    fn sub_assign(&mut self, rhs: Self) {
        self.width -= rhs.width;
        self.height -= rhs.height;
    }
}

impl<T: Mul<Output = T>> Mul for Size<T> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            width: self.width * rhs.width,
            height: self.height * rhs.height,
        }
    }
}

impl<T: MulAssign> MulAssign for Size<T> {
    fn mul_assign(&mut self, rhs: Self) {
        self.width *= rhs.width;
        self.height *= rhs.height;
    }
}

impl<T: Div<Output = T>> Div for Size<T> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            width: self.width / rhs.width,
            height: self.height / rhs.height,
        }
    }
}

impl<T: DivAssign> DivAssign for Size<T> {
    fn div_assign(&mut self, rhs: Self) {
        self.width /= rhs.width;
        self.height /= rhs.height;
    }
}
