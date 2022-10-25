//! An implementation of the ObjectProperty reflection and
//! serialization system.
//!
//! This is inspired by the original work of [Richard Lyle]
//! as part of the [Medusa] project.
//!
//! [Richard Lyle]: https://github.com/rlyle
//! [Medusa]: https://github.com/palestar/medusa

#![deny(
    rust_2018_idioms,
    rustdoc::broken_intra_doc_links,
    unsafe_op_in_unsafe_fn
)]
#![feature(
    // Better macro ergonomics.
    decl_macro
)]

#[doc(hidden)]
pub mod __private {
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
