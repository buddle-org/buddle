use proc_macro2::TokenStream;
use syn::{spanned::Spanned, Data, Error, Path, Result, Visibility};

use crate::utils::default_crate_path;

mod ast;
use self::ast::Input;

mod attrs;

pub fn derive(node: syn::DeriveInput) -> Result<TokenStream> {
    let input = Input::from_syn(&node)?;
    input.validate()?;

    let path = crate_path(&input);
    match input {
        Input::Struct(input) => derive_struct(input, &path),
        Input::Enum(input) => derive_enum(input, &path),
    }
}

macro_rules! spanned_trait {
    ($trait:path, $input:expr, $path:ident) => {{
        let vis_span = match &$input.vis {
            Visibility::Public(vis) => Some(vis.pub_token.span()),
            Visibility::Crate(vis) => Some(vis.crate_token.span()),
            Visibility::Restricted(vis) => Some(vis.pub_token.span()),
            Visibility::Inherited => None,
        };
        let data_span = match &$input.data {
            Data::Struct(data) => data.struct_token.span(),
            Data::Enum(data) => data.enum_token.span(),
            Data::Union(_) => unreachable!(),
        };
        let first_span = vis_span.unwrap_or(data_span);
        let last_span = $input.ident.span();

        let path = quote_spanned!(first_span => #$path::);
        let ty = quote_spanned!(last_span => $trait);

        quote!(#path #ty)
    }};
}

fn derive_struct(input: ast::Struct<'_>, path: &Path) -> Result<TokenStream> {
    let ty = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let name = input
        .attrs
        .object
        .as_ref()
        .and_then(|o| o.name())
        .map(|name| quote!(Some(#name)))
        .unwrap_or_else(|| quote!(None));

    let base = {
        let mut fields = input
            .fields
            .iter()
            .filter(|f| f.attrs.property.as_ref().map(|p| p.base).unwrap_or(false));

        let base = fields.next();
        if fields.next().is_some() {
            return Err(Error::new_spanned(
                input.ident,
                "only one base class property allowed",
            ));
        }

        if let Some(base) = base {
            let ident = &base.ident;
            let base_ty = base.ty;
            let info = base.info(path);
            quote! {
                unsafe {
                    ::std::option::Option::Some(#path::type_info::Property::new_base::<#base_ty>(
                        #info,
                        #path::__private::offset_of!(#ty #ty_generics, #ident),
                    ))
                }
            }
        } else {
            quote!(::std::option::Option::None)
        }
    };

    let fields: Vec<_> = input
        .fields
        .iter()
        .filter(|f| f.attrs.property.as_ref().map(|p| !p.base).unwrap_or(false))
        .collect();
    let idents = fields.iter().map(|f| &f.ident);
    let tys = fields.iter().map(|f| f.ty);
    let names = fields.iter().map(|f| f.name());
    let field_count = fields.len();
    let flags = fields.iter().map(|f| f.flags());
    let infos = fields.iter().map(|f| f.info(path));
    let on_pre_load = input.on_pre_load();
    let on_post_load = input.on_post_load();
    let on_pre_save = input.on_pre_save();
    let on_post_save = input.on_post_save();

    let reflected = spanned_trait!(type_info::Reflected, input.original, path);
    let type_trait = spanned_trait!(Type, input.original, path);
    let property_class = spanned_trait!(PropertyClass, input.original, path);

    Ok(quote! {
        // SAFETY: Structs that are PropertyClasses always
        // get a PropertyList since they are non-leaf types.
        // We correctly reflect all the properties in the list.
        const _: () = {
            const __PROPERTIES: [#path::type_info::Property; #field_count] = [
                #(
                    unsafe {
                        #path::type_info::Property::new::<#tys>(
                            #names,
                            #path::type_info::PropertyFlags::empty()
                                #(.union(#path::type_info::PropertyFlags::#flags))*,
                            false,
                            #infos,
                            #path::__private::offset_of!(#ty #ty_generics, #idents),
                        )
                    }
                ),*
            ];

            unsafe impl #impl_generics #reflected for #ty #ty_generics
                #where_clause
            {
                const TYPE_INFO: &'static #path::type_info::TypeInfo = unsafe {
                    &#path::type_info::TypeInfo::Class(
                        #path::type_info::PropertyList::new::<#ty #ty_generics>(
                            ::std::option::Option::#name,
                            #base,
                            &__PROPERTIES,
                        )
                    )
                };
            }
        };

        impl #impl_generics #type_trait for #ty #ty_generics #where_clause {
            #[inline]
            fn as_any(&self) -> &dyn ::std::any::Any {
                self
            }

            #[inline]
            fn as_any_mut(&mut self) -> &mut dyn ::std::any::Any {
                self
            }

            #[inline]
            fn as_type(&self) -> &dyn #path::Type {
                self
            }

            #[inline]
            fn as_type_mut(&mut self) -> &mut dyn #path::Type {
                self
            }

            #[inline]
            fn type_ref(&self) -> #path::TypeRef<'_> {
                #path::TypeRef::Class(self)
            }

            #[inline]
            fn type_mut(&mut self) -> #path::TypeMut<'_> {
                #path::TypeMut::Class(self)
            }

            #[inline]
            fn set(
                &mut self,
                value: ::std::boxed::Box<dyn #path::Type>,
            ) -> ::std::result::Result<(), ::std::boxed::Box<dyn #path::Type>> {
                *self = *value.downcast()?;
                ::std::result::Result::Ok(())
            }
        }

        impl #impl_generics #property_class for #ty #ty_generics #where_clause {
            fn make_default() -> ::std::boxed::Box<dyn #property_class>
            where
                Self: ::std::marker::Sized,
            {
                <::std::boxed::Box::<Self> as ::std::default::Default>::default()
            }

            fn on_pre_save(&mut self) {
                let _ = #on_pre_save(self);
            }

            fn on_post_save(&mut self) {
                let _ = #on_post_save(self);
            }

            fn on_pre_load(&mut self) {
                let _ = #on_pre_load(self);
            }

            fn on_post_load(&mut self) {
                let _ = #on_post_load(self);
            }
        }
    })
}

fn derive_enum(input: ast::Enum<'_>, path: &Path) -> Result<TokenStream> {
    let ty = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let idents: Vec<_> = input.variants.iter().map(|v| &v.ident).collect();
    let names: Vec<_> = input.variants.iter().map(|v| v.name()).collect();
    let discrims: Vec<_> = input.variants.iter().map(|v| &v.discriminant).collect();

    let reflected = spanned_trait!(type_info::Reflected, input.original, path);
    let type_trait = spanned_trait!(Type, input.original, path);
    let enum_trait = spanned_trait!(Enum, input.original, path);

    Ok(quote! {
        // SAFETY: Enums are always leaf types by nature.
        // We capture the correct type the derive macro was
        // used on and go with the default Rust type name.
        unsafe impl #impl_generics #reflected for #ty #ty_generics
            #where_clause
        {
            const TYPE_INFO: &'static #path::type_info::TypeInfo =
                &#path::type_info::TypeInfo::leaf::<#ty #ty_generics>(
                    ::std::option::Option::None
                );
        }

        impl #impl_generics #type_trait for #ty #ty_generics #where_clause {
            #[inline]
            fn as_any(&self) -> &dyn ::std::any::Any {
                self
            }

            #[inline]
            fn as_any_mut(&mut self) -> &mut dyn ::std::any::Any {
                self
            }

            #[inline]
            fn as_type(&self) -> &dyn #path::Type {
                self
            }

            #[inline]
            fn as_type_mut(&mut self) -> &mut dyn #path::Type {
                self
            }

            #[inline]
            fn type_ref(&self) -> #path::TypeRef<'_> {
                #path::TypeRef::Enum(self)
            }

            #[inline]
            fn type_mut(&mut self) -> #path::TypeMut<'_> {
                #path::TypeMut::Enum(self)
            }

            #[inline]
            fn set(
                &mut self,
                value: ::std::boxed::Box<dyn #path::Type>,
            ) -> ::std::result::Result<(), ::std::boxed::Box<dyn #path::Type>> {
                *self = *value.downcast()?;
                ::std::result::Result::Ok(())
            }
        }

        impl #impl_generics #enum_trait for #ty #ty_generics #where_clause {
            fn variant(&self) -> &'static ::std::primitive::str {
                match () {
                    #(() if ::std::matches!(self, #ty::#idents) => #names,)*

                    // For the sake of catching bugs, we want to panic
                    // debug mode if this branch is ever executed.
                    #[cfg(debug_assertions)]
                    _ => ::std::unreachable!(),

                    // SAFETY: We exhaustively covered all enum variants
                    // since the proc macro captures all of them. We have
                    // also employed a safety check in debug mode, so we
                    // want to discard this branch in release mode.
                    #[cfg(not(debug_assertions))]
                    _ => unsafe { ::std::hint::unreachable_unchecked() },
                }
            }

            fn from_variant(variant: &::std::primitive::str) -> ::std::option::Option<Self>
            where
                Self: ::std::marker::Sized
            {
                match variant {
                    #(#names => ::std::option::Option::Some(#ty::#idents),)*
                    _ => ::std::option::Option::None,
                }
            }

            fn value(&self) -> ::std::primitive::u32 {
                match self {
                    #(#ty::#idents => #discrims,)*
                }
            }

            fn from_value(value: ::std::primitive::u32) -> ::std::option::Option<Self>
            where
                Self: ::std::marker::Sized
            {
                match value {
                    #(d if d == #discrims => ::std::option::Option::Some(#ty::#idents),)*
                    _ => ::std::option::Option::None,
                }
            }
        }
    })
}

fn crate_path(input: &Input<'_>) -> Path {
    let attrs = match input {
        Input::Struct(data) => &data.attrs,
        Input::Enum(data) => &data.attrs,
    };

    attrs
        .op_crate
        .as_ref()
        .map(|attr| attr.krate.clone())
        .unwrap_or_else(default_crate_path)
}
