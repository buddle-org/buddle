use std::{
    any::{Any, TypeId},
    fmt::Debug,
};

use crate::{
    container::Container,
    property_class::PropertyClass,
    r#enum::Enum,
    serde::{Deserializer, Serializer},
    type_info::DynReflected,
};

/// An immutable reference to a value categorized by varying data types.
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

/// A mutable reference to a value categorized by varying data types.
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
    /// Gets the inner value as a boxed [`Type`] object.
    pub fn into_type(self) -> Box<dyn Type> {
        match self {
            TypeOwned::Class(v) => v,
            TypeOwned::Container(v) => v,
            TypeOwned::Enum(v) => v,
            TypeOwned::Value(v) => v,
        }
    }
}

/// A reflected Rust type in the *ObjectProperty* system.
///
/// # Correctness
///
/// While not directly causing memory unsafety, the following
/// invariants must be met by implementations of this trait.
///
/// It is generally recommended to just leave the work to the
/// `#[derive(Type)]` macro unless there is a reason not to.
///
/// - [`Type::as_any`] and [`Type::as_any_mut`] should always return `self`.
///
/// - [`Type::type_ref`], [`Type::type_mut`] and [`Type::type_owned`] should
///   return the most concrete variant `self` can be represented as.
pub trait Type: Any + Sync + Send + Debug + DynReflected + 'static {
    /// Gets the value as an [`Any`] reference.
    fn as_any(&self) -> &dyn Any;

    /// Gets the value as an [`Any`] reference.
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Gets `self` as a [`TypeRef`].
    fn type_ref(&self) -> TypeRef<'_>;

    /// Gets `self` as a [`TypeMut`].
    fn type_mut(&mut self) -> TypeMut<'_>;

    /// Gets `self` as [`TypeOwned`].
    fn type_owned(self: Box<Self>) -> TypeOwned;

    /// Attempts to perform a type-checked assignment of `value` to `self`.
    ///
    /// If the types are incompatible with each other, then `value` will be
    /// passed back in the [`Err`] variant of the [`Result`].
    fn set(&mut self, value: Box<dyn Type>) -> Result<(), Box<dyn Type>>;

    /// Serializes `self` to the given [`Serializer`].
    ///
    /// Serialization is infallible so this method does not return anything.
    fn serialize(&mut self, ser: &mut Serializer<'_>);

    /// Deserializes `self` from the given [`Deserializer`] in-place.
    fn deserialize(&mut self, de: &mut Deserializer<'_>) -> anyhow::Result<()>;
}

impl dyn Type {
    /// Checks if this value is a `T` object.
    #[inline]
    pub fn is<T: Type>(&self) -> bool {
        self.type_id() == TypeId::of::<T>()
    }

    // In debug builds we can enforce this trait's implementation invariants
    // to a certain degree at runtime. This helps with spotting bugs.
    #[cfg(debug_assertions)]
    fn debug_check_invariants(&self) {
        let type_id = self.type_id();

        assert_eq!(
            type_id,
            self.as_any().type_id(),
            "TypeId mismatch between self and Any; make Type::as_any(_mut) return self"
        );
    }

    /// Downcasts the value into a concrete type if it is a `T` object.
    #[inline]
    pub fn downcast_ref<T: Type>(&self) -> Option<&T> {
        #[cfg(debug_assertions)]
        self.debug_check_invariants();

        self.as_any().downcast_ref()
    }

    /// Downcasts the value into a concrete type if it is a `T` object.
    #[inline]
    pub fn downcast_mut<T: Type>(&mut self) -> Option<&mut T> {
        #[cfg(debug_assertions)]
        self.debug_check_invariants();

        self.as_any_mut().downcast_mut()
    }

    /// Consumes `self` and returns it as a `T` object, if it is one.
    ///
    /// When this is not the case, `self` will be passed back as the [`Err`]
    /// variant of the [`Result`].
    pub fn downcast<T: Type>(self: Box<dyn Type>) -> Result<Box<T>, Box<dyn Type>> {
        match self.is::<T>() {
            true => unsafe {
                // SAFETY: The TypeId of the boxed value matches the ID of type T.
                let ptr = Box::into_raw(self);
                Ok(Box::from_raw(ptr.cast::<T>()))
            },
            false => Err(self),
        }
    }
}
