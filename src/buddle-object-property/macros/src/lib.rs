//! Procedural macros for use with [`buddle-object-property`].
//!
//! There is no need to directly add this crate to application
//! dependencies as these macros are already re-exported by
//! [`buddle-object-property`].
//!
//! [`buddle-object-property`]: ../buddle_object_property/

#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

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
