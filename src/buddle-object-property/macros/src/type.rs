use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, Path, Result};

use crate::utils::default_crate_path;

mod ast;
use ast::Input;

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

fn derive_struct(input: ast::Struct<'_>, path: &Path) -> Result<TokenStream> {
    let ty = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let (name, name_option) = input
        .attrs
        .object
        .as_ref()
        .and_then(|o| o.name())
        .map(|name| (quote!(#name), quote!(Some(#name))))
        .unwrap_or_else(|| (quote!(#path::__private::type_name::<Self>()), quote!(None)));

    let base = {
        let mut fields = input.fields.iter().filter(|f| f.is_base());

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
                    ::std::option::Option::Some(#path::type_info::Property::new::<#base_ty>(
                        "super",
                        #path::type_info::PropertyFlags::empty(),
                        true,
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
        .filter(|f| f.attrs.property.is_some() && !f.is_base())
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

    Ok(quote! {
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

            // SAFETY: Structs that are PropertyClasses always get a PropertyList
            // since they are non-leaf types. We correctly reflect all properties.
            unsafe impl #impl_generics #path::type_info::Reflected for #ty #ty_generics
                #where_clause
            {
                const TYPE_NAME: &'static ::std::primitive::str = #name;

                const TYPE_INFO: &'static #path::type_info::TypeInfo = unsafe {
                    &#path::type_info::TypeInfo::Class(
                        #path::type_info::PropertyList::new::<#ty #ty_generics>(
                            ::std::option::Option::#name_option,
                            #base,
                            &__PROPERTIES,
                        )
                    )
                };
            }
        };

        impl #impl_generics #path::Type for #ty #ty_generics #where_clause {
            #path::impl_type_methods!(Class);

            #[inline]
            fn serialize(&mut self, ser: &mut #path::serde::Serializer<'_>) {
                ser.serialize(self);
            }

            #[inline]
            fn deserialize(
                &mut self,
                de: &mut #path::serde::Deserializer<'_>,
            ) -> #path::__private::Result<()> {
                de.deserialize_class(self)
            }
        }

        impl #impl_generics #path::PropertyClass for #ty #ty_generics #where_clause {
            fn make_default() -> ::std::boxed::Box<dyn #path::PropertyClass>
            where
                Self: ::std::marker::Sized,
            {
                <::std::boxed::Box::<Self> as ::std::default::Default>::default()
            }

            fn base(&self) -> ::std::option::Option<&dyn #path::PropertyClass> {
                let list = self.property_list();
                list.base_value(self)
            }

            fn base_mut(&mut self) -> ::std::option::Option<&mut dyn #path::PropertyClass> {
                let list = self.property_list();
                list.base_value_mut(self)
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

    Ok(quote! {
        // SAFETY: Enums are always leaf types by nature.
        unsafe impl #impl_generics #path::type_info::Reflected for #ty #ty_generics
            #where_clause
        {
            const TYPE_NAME: &'static ::std::primitive::str = Self::TYPE_INFO.type_name();

            const TYPE_INFO: &'static #path::type_info::TypeInfo =
                &#path::type_info::TypeInfo::leaf::<#ty #ty_generics>(
                    ::std::option::Option::None
                );
        }

        impl #impl_generics #path::Type for #ty #ty_generics #where_clause {
            #path::impl_type_methods!(Enum);

            #[inline]
            fn serialize(&mut self, ser: &mut #path::serde::Serializer<'_>) {
                ser.serialize_enum(self);
            }

            #[inline]
            fn deserialize(
                &mut self,
                de: &mut #path::serde::Deserializer<'_>,
            ) -> #path::__private::Result<()> {
                de.deserialize_enum(self)
            }
        }

        impl #impl_generics #path::Enum for #ty #ty_generics #where_clause {
            fn variant(&self) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                match self {
                    #(#ty::#idents => ::std::borrow::Cow::Borrowed(#names),)*
                }
            }

            fn update_variant(
                &mut self,
                variant: &::std::primitive::str,
            ) -> ::std::primitive::bool {
                *self = match variant {
                    #(#names => #ty::#idents,)*

                    _ => return false,
                };

                true
            }

            fn value(&self) -> ::std::primitive::u32 {
                match self {
                    #(#ty::#idents => #discrims,)*
                }
            }

            fn update_value(&mut self, value: ::std::primitive::u32) -> ::std::primitive::bool {
                *self = match value {
                    #(d if d == #discrims => #ty::#idents,)*

                    _ => return false,
                };

                true
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
