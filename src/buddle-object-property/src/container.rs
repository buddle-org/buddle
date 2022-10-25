use crate::Type;

/// A dynamically growable container that stores sequences
/// of values.
pub trait Container: Type {
    /// Returns an immutable reference to an element
    /// at the given index.
    fn get(&self, idx: usize) -> Option<&dyn Type>;

    /// Returns a mutable reference to an element
    /// at the given index.
    fn get_mut(&mut self, idx: usize) -> Option<&mut dyn Type>;

    /// Appends a new element to the back.
    fn push(&mut self, value: Box<dyn Type>);

    /// Removes an element from the back.
    fn pop(&mut self) -> Option<Box<dyn Type>>;

    /// Clears the container, removing all elements but
    /// preserving the capacity allocation.
    fn clear(&mut self);

    /// Reserves memory for `capacity` more elements.
    fn reserve(&mut self, capacity: usize);

    /// Returns the number of elements inside the container.
    fn len(&self) -> usize;

    /// Indicates whether the container is empty.
    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator over the [`Type`] objects in
    /// the container.
    fn iter(&self) -> ContainerIter<'_>;
}

/// An [`Iterator`] that produces immutable references to
/// container elements.
///
/// The values will be returned in the same order they are
/// stored.
pub struct ContainerIter<'c> {
    container: &'c dyn Container,
    index: usize,
}

impl<'c> ContainerIter<'c> {
    pub(crate) fn new(container: &'c dyn Container) -> Self {
        Self {
            container,
            index: 0,
        }
    }
}

impl<'c> Iterator for ContainerIter<'c> {
    type Item = &'c dyn Type;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let value = self.container.get(self.index);
        self.index += 1;
        value
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.container.len();
        (size, Some(size))
    }
}

impl<'c> ExactSizeIterator for ContainerIter<'c> {}
