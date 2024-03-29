//! Procedural macros for use with [`buddle-object-property`].
//!
//! There is no need to directly add this crate to application
//! dependencies as these macros are already re-exported by
//! [`buddle-object-property`].
//!
//! [`buddle-object-property`]: ../buddle_object_property/

#![deny(rust_2018_idioms, rustdoc::broken_intra_doc_links)]
#![forbid(unsafe_code)]

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod bitenum;
mod r#type;
mod utils;

/// TODO: Document this.
#[proc_macro_derive(Type, attributes(op_crate, object, property, option))]
pub fn derive_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    r#type::derive(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// TODO: Document this.
#[proc_macro]
pub fn bitenum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as bitenum::Input);
    bitenum::expand(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
