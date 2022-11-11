//! Utilities for working with colors.

/// An RGBA color.
// https://github.com/palestar/medusa/blob/develop/Standard/Color.h
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Color {
    /// The red channel value.
    pub r: u8,
    /// The green channel value.
    pub g: u8,
    /// The blue channel value.
    pub b: u8,
    /// The red channel value.
    pub a: u8,
}

impl Color {
    /// Creates a new RGBA color given all channel values.
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Creates a new RGB color given the channel values.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            r,
            g,
            b,
            a: u8::MAX,
        }
    }

    /// Gets the distance between two colors.
    pub fn distance_from(&self, rhs: &Self) -> f32 {
        (self.r as f32 - rhs.r as f32).powi(2)
            + (self.g as f32 - rhs.g as f32).powi(2)
            + (self.b as f32 - rhs.b as f32).powi(2)
            + (self.a as f32 - rhs.a as f32).powi(2)
    }

    /// Interpolates two colors.
    ///
    /// `fraction` is a factor that denotes how much
    /// interpolation is desired. `0.0` is the full
    /// `self` color while `1.0` is the full `rhs`.
    pub fn interpolate(&self, rhs: &Self, fraction: f32) -> Self {
        let interpolate_value = |value, other, frac| {
            let value = value as f32;
            (value + (frac * (other as f32 - value))) as u8
        };

        Self {
            r: interpolate_value(self.r, rhs.r, fraction),
            g: interpolate_value(self.g, rhs.g, fraction),
            b: interpolate_value(self.b, rhs.b, fraction),
            a: interpolate_value(self.a, rhs.a, fraction),
        }
    }
}

impl Color {
    /// Creates a black color.
    #[inline]
    pub fn black() -> Self {
        Self::rgb(u8::MIN, u8::MIN, u8::MIN)
    }

    /// Creates a white color.
    #[inline]
    pub fn white() -> Self {
        Self::rgb(u8::MAX, u8::MAX, u8::MAX)
    }

    /// Creates a red color.
    #[inline]
    pub fn red() -> Self {
        Self::rgb(u8::MAX, u8::MIN, u8::MIN)
    }

    /// Creates a new green color.
    #[inline]
    pub fn green() -> Self {
        Self::rgb(u8::MIN, u8::MAX, u8::MIN)
    }

    /// Creates a new blue color.
    #[inline]
    pub fn blue() -> Self {
        Self::rgb(u8::MIN, u8::MIN, u8::MAX)
    }
}
