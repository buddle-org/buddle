use syn::Path;

/// Gets the default path to the `buddle_object_property`
/// crate.
pub fn default_crate_path() -> Path {
    syn::parse2(quote!(::buddle_object_property)).unwrap()
}
