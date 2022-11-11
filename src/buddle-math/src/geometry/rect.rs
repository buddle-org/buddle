use num_traits::Num;

use crate::{Point, Size};

/// A rectangular shape in two-dimensional space.
// https://github.com/palestar/medusa/blob/develop/Standard/Rect.h
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect<T> {
    /// The location of the left edge.
    pub left: T,
    /// The location of the top edge.
    pub top: T,
    /// The location of the right edge.
    pub right: T,
    /// The location of the bottom edge.
    pub bottom: T,
}

impl<T> Rect<T> {
    /// Creates a new rectangle given its edges.
    pub const fn new(left: T, top: T, right: T, bottom: T) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }
}

impl<T: Copy + From<u8> + PartialOrd + Num> Rect<T> {
    /// Creates a rectangle from coordinates and its size.
    pub fn from_coords_and_size(x: T, y: T, size: &Size<T>) -> Self {
        Self {
            left: x,
            top: y,
            right: x + size.width - T::one(),
            bottom: y + size.height - T::one(),
        }
    }

    /// Creates a rectangle from a point and a size.
    pub fn from_point_and_size(point: &Point<T>, size: &Size<T>) -> Self {
        Self::from_coords_and_size(point.x, point.y, size)
    }

    /// Checks whether the rectangle is valid.
    pub fn is_valid(&self) -> bool {
        self.left <= self.right && self.top <= self.bottom
    }

    /// Gets the width of the shape.
    pub fn width(&self) -> T {
        (self.right - self.left) + T::one()
    }

    /// Gets the height of the shape.
    pub fn height(&self) -> T {
        (self.bottom - self.top) + T::one()
    }

    /// Gets the size of the shape.
    pub fn size(&self) -> Size<T> {
        Size::new(self.width(), self.height())
    }

    /// Checks if a given set of coordinates is inside
    /// the shape.
    pub fn contains_coords(&self, x: T, y: T) -> bool {
        x >= self.left && x <= self.right && y >= self.top && y <= self.bottom
    }

    /// Checks if a given point is inside the shape.
    pub fn contains_point(&self, point: &Point<T>) -> bool {
        self.contains_coords(point.x, point.y)
    }

    /// Checks if a given rectangle is inside the shape.
    pub fn contains_rect(&self, rect: &Self) -> bool {
        self.left <= rect.left
            && self.right >= rect.right
            && self.top <= rect.top
            && self.bottom >= rect.bottom
    }

    /// Gets the center point of the shape.
    pub fn center(&self) -> Point<T> {
        Point::new(
            (self.left + self.right) / T::from(2),
            (self.top + self.bottom) / T::from(2),
        )
    }

    /// Gets the upper left point of the shape.
    pub fn upper_left(&self) -> Point<T> {
        Point::new(self.left, self.top)
    }

    /// Gets the upper right point of the shape.
    pub fn upper_right(&self) -> Point<T> {
        Point::new(self.right, self.top)
    }

    /// Gets the lower left point of the shape.
    pub fn lower_left(&self) -> Point<T> {
        Point::new(self.left, self.bottom)
    }

    /// Gets the lower right point of the shape.
    pub fn lower_right(&self) -> Point<T> {
        Point::new(self.right, self.bottom)
    }
}
