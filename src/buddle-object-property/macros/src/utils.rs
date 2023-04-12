use quote::quote;
use syn::Path;

pub fn default_crate_path() -> Path {
    syn::parse2(quote!(::buddle_object_property)).unwrap()
}
