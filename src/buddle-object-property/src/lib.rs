//! An implementation of the ObjectProperty reflection and
//! serialization system.
//!
//! This is inspired by the original work of [Richard Lyle]
//! as part of the [Medusa] project.
//!
//! [Richard Lyle]: https://github.com/rlyle
//! [Medusa]: https://github.com/palestar/medusa

#![allow(incomplete_features)]
#![deny(
    rust_2018_idioms,
    rustdoc::broken_intra_doc_links,
    unsafe_op_in_unsafe_fn
)]
#![feature(
    // Compile-time type info for static reflection.
    const_option_ext,
    const_type_id,
    const_type_name,

    // Better macro ergonomics.
    decl_macro,

    // Enables reflected access to `dyn Type` properties.
    pointer_byte_offsets,
    ptr_metadata,

    // Coercion of `dyn PropertyClass` to `dyn Type`.
    trait_upcasting
)]

#[doc(inline)]
pub use buddle_object_property_macros::*;

mod container;
pub use self::container::*;

pub mod cpp;

mod r#enum;
pub use self::r#enum::*;

mod impls;

pub mod path;

mod property_class;
pub use self::property_class::*;

pub mod serde;

pub mod type_info;

mod r#type;
pub use self::r#type::*;

#[doc(hidden)]
pub mod __private {
    pub use bitflags::bitflags;

    /// Wrapper around [`std::any::type_name`] for codegen.
    ///
    /// Doesn't require enabling the `const_type_name`
    /// nightly feature in user code.
    pub const fn type_name<T: ?Sized>() -> &'static str {
        std::any::type_name::<T>()
    }

    /// Computes the offset to a struct field for
    /// pointer access.
    pub macro offset_of($ty:path, $field:ident) {{
        // Allocate an uninitialized `$ty` and get a pointer
        // to its designated storage.
        //
        // This pointer must never be dereferenced.
        let uninit = ::std::mem::MaybeUninit::<$ty>::uninit();
        let parent = uninit.as_ptr();

        // This protects against deref coercion by statically
        // enforcing that `$field` is an actual field in `$ty`.
        #[allow(clippy::unneeded_field_pattern)]
        let $ty { $field: _, .. };

        // Craft a pointer to `$field` without creating a
        // reference to the uninitialized `uninit`.
        //
        // The resulting `child` pointer will inherit `parent`'s
        // provenance which is required for subsequent pointer
        // arithmetic operations.
        #[allow(unused_unsafe)] // Macro may be used in unsafe block.
        let child = unsafe { ::std::ptr::addr_of!((*parent).$field) };

        // Finally compute the offset from `parent` to `child`.
        //
        // The pointers share the same provenance and are in bounds
        // of the same allocated object (see deref coercion above).
        //
        // Further, subtracting `parent` from `child` will always
        // produce a non-negative value.
        #[allow(unused_unsafe)] // Macro may be used in unsafe block.
        unsafe {
            let offset = child.cast::<u8>().offset_from(parent.cast::<u8>());
            debug_assert!(offset >= 0);
            offset as usize
        }
    }}
}
