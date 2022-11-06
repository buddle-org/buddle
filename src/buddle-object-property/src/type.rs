use std::any::{Any, TypeId};

use crate::{
    serde::{self, de::DynDeserializer, ser::DynSerializer, Baton},
    type_info::DynReflected,
    Container, Enum, PropertyClass,
};

/// An immutable reference to a value categorized by
/// varying data types.
pub enum TypeRef<'ty> {
    /// A property class reference.
    Class(&'ty dyn PropertyClass),
    /// A container reference.
    Container(&'ty dyn Container),
    /// An enum reference.
    Enum(&'ty dyn Enum),
    /// A regular value reference.
    Value(&'ty dyn Type),
}

/// A mutable reference to a value categorized by
/// varying data types.
pub enum TypeMut<'ty> {
    /// A property class reference.
    Class(&'ty mut dyn PropertyClass),
    /// A container reference.
    Container(&'ty mut dyn Container),
    /// An enum reference.
    Enum(&'ty mut dyn Enum),
    /// A regular value reference.
    Value(&'ty mut dyn Type),
}

/// An owned value categorized by varying data types.
pub enum TypeOwned {
    /// A property class object.
    Class(Box<dyn PropertyClass>),
    /// A container object.
    Container(Box<dyn Container>),
    /// An enum object.
    Enum(Box<dyn Enum>),
    /// A regular value object.
    Value(Box<dyn Type>),
}

impl TypeOwned {
    ///
    pub fn into_type(self) -> Box<dyn Type> {
        match self {
            Self::Class(value) => value,
            Self::Container(value) => value,
            Self::Enum(value) => value,
            Self::Value(value) => value,
        }
    }
}

/// A reflected Rust type in the *ObjectProperty* system.
///
/// # Correctness
///
/// While not directly causing memory unsafety, the
/// following invariants must be met by implementations
/// of this trait.
///
/// It is generally recommended to just leave the work
/// to the `#[derive(Type)]` macro unless there is a
/// reason not to.
///
/// - [`Type::as_any`] and [`Type::as_any_mut`] should
///   always return `self`.
///
/// - [`Type::as_type`] and [`Type::as_type_mut`] should
///   always return `self`.
pub trait Type: Any + Sync + Send + DynReflected + 'static {
    /// Gets the value as an [`Any`] reference.
    fn as_any(&self) -> &dyn Any;

    /// Gets the value as an [`Any`] reference.
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Gets the value as a [`Type`] reference.
    fn as_type(&self) -> &dyn Type;

    /// Gets the value as a [`Type`] reference.
    fn as_type_mut(&mut self) -> &mut dyn Type;

    /// Gets the value as a boxed [`Type`] object.
    fn as_boxed_type(self: Box<Self>) -> Box<dyn Type>;

    /// Gets `self` as a [`TypeRef`].
    fn type_ref(&self) -> TypeRef<'_>;

    /// Gets `self` as a [`TypeMut`].
    fn type_mut(&mut self) -> TypeMut<'_>;

    /// Gets `self` as a [`TypeOwned`].
    fn type_owned(self: Box<Self>) -> TypeOwned;

    /// Attempts to perform a type-checked assignment of
    /// `value` to `self`.
    ///
    /// If the types are incompatible with each other, then
    /// `value` will be passed back in the [`Err`] variant.
    fn set(&mut self, value: Box<dyn Type>) -> Result<(), Box<dyn Type>>;

    /// Serializes `self` to the given serializer.
    ///
    /// Types that do not support serialization do not
    /// have to override this method.
    #[allow(unused_variables)]
    fn serialize(&self, serializer: &mut dyn DynSerializer, baton: Baton) -> serde::Result<()> {
        Err(serde::Error::custom(
            "this type does not support serialization",
        ))
    }

    /// Deserializes `self` from the given deserializer.
    ///
    /// Types that do not support serialization do not
    /// have to override this method.
    #[allow(unused_variables)]
    fn deserialize(
        &mut self,
        deserializer: &mut dyn DynDeserializer,
        baton: Baton,
    ) -> serde::Result<()> {
        Err(serde::Error::custom(
            "this type does not support deserialization",
        ))
    }
}

impl dyn Type {
    /// Checks if this value is an instance of `T`.
    #[inline]
    pub fn is<T: Type>(&self) -> bool {
        self.type_id() == TypeId::of::<T>()
    }

    // In debug builds we can enforce the trait's
    // implementation invariants to a certain degree
    // at runtime. This helps with spotting bugs.
    #[cfg(debug_assertions)]
    fn debug_check_invariants(&self) {
        let type_id = self.type_id();

        assert_eq!(
            type_id,
            self.as_any().type_id(),
            "TypeId mismatch between self and Any; make Type::as_any(_mut) return self"
        );
        assert_eq!(
            type_id,
            self.as_type().type_id(),
            "TypeId mismatch between self and Any; make Type::as_type(_mut) return self"
        );
    }

    /// Downcasts the value into the concrete type if it
    /// is a `T` underneath.
    #[inline]
    pub fn downcast_ref<T: Type>(&self) -> Option<&T> {
        #[cfg(debug_assertions)]
        self.debug_check_invariants();

        self.as_any().downcast_ref()
    }

    /// Downcasts the value into the concrete type if it
    /// is a `T` underneath.
    #[inline]
    pub fn downcast_mut<T: Type>(&mut self) -> Option<&mut T> {
        #[cfg(debug_assertions)]
        self.debug_check_invariants();

        self.as_any_mut().downcast_mut()
    }

    /// Consumes `self` and casts it into a concrete `T`,
    /// if it is one underneath.
    ///
    /// When that is not the case, `self` will be returned
    /// as-is in the error variant to re-gain ownership.
    pub fn downcast<T: Type>(self: Box<dyn Type>) -> Result<Box<T>, Box<dyn Type>> {
        match self.is::<T>() {
            true => unsafe {
                // SAFETY: The TypeId of the boxed value matches the
                // ID of type T. Thus, we can cast the pointer.
                // Since it is `Sized`, it doesn't require metadata.
                let ptr = Box::into_raw(self);
                Ok(Box::from_raw(ptr.cast::<T>()))
            },
            false => Err(self),
        }
    }
}
