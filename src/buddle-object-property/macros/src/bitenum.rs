use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    Path, Token,
};

use crate::utils::default_crate_path;

pub struct Input {
    krate: Option<Path>,
    item: Bits,
}

impl Parse for Input {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let krate = if input.peek(Token![#]) && input.peek2(Token![!]) {
            // #![crate = kronos_object_property]

            input.parse::<Token![#]>()?;
            input.parse::<Token![!]>()?;

            let content;
            syn::bracketed!(content in input);

            content.parse::<Token![crate]>()?;
            content.parse::<Token![=]>()?;

            Some(content.parse()?)
        } else {
            None
        };

        Ok(Self {
            krate,
            item: input.parse()?,
        })
    }
}

struct Bits {
    attrs: Vec<syn::Attribute>,
    vis: syn::Visibility,
    ident: syn::Ident,
    flags: Punctuated<Flag, Token![;]>,
}

impl Parse for Bits {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;

        let vis = input.parse()?;
        input.parse::<Token![struct]>()?;
        let ident = input.parse()?;

        let content;
        syn::braced!(content in input);

        Ok(Self {
            attrs,
            vis,
            ident,
            flags: content.parse_terminated(Flag::parse)?,
        })
    }
}

struct Flag {
    attrs: Vec<syn::Attribute>,
    ident: syn::Ident,
    value: syn::Expr,
}

impl Parse for Flag {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;

        input.parse::<syn::Visibility>()?;
        input.parse::<Token![const]>()?;

        let ident = input.parse()?;
        input.parse::<Token![=]>()?;
        let value = input.parse()?;

        Ok(Self {
            attrs,
            ident,
            value,
        })
    }
}

pub fn expand(input: Input) -> Result<TokenStream> {
    let path = input.krate.unwrap_or_else(default_crate_path);

    let Bits {
        attrs,
        vis,
        ident: ty,
        flags,
    } = &input.item;

    let bits: Vec<_> = flags.iter().collect();
    let bit_attrs = bits.iter().map(|f| &f.attrs);
    let bit_idents: Vec<_> = bits.iter().map(|f| &f.ident).collect();
    let bit_names = bits.iter().map(|f| f.ident.to_string());
    let bit_exprs = bits.iter().map(|f| &f.value);

    Ok(quote! {
        #path::__private::bitflags! {
            #(#attrs)*
            #[repr(transparent)]
            #vis struct #ty: ::std::primitive::u32 {
                #(
                    #(#bit_attrs)*
                    const #bit_idents = #bit_exprs;
                )*
            }
        }

        // SAFETY: Enums are always leaf types by nature.
        unsafe impl #path::type_info::Reflected for #ty {
            const TYPE_NAME: &'static ::std::primitive::str = Self::TYPE_INFO.type_name();

            const TYPE_INFO: &'static #path::type_info::TypeInfo =
                &#path::type_info::TypeInfo::leaf::<#ty>(
                    ::std::option::Option::None
                );
        }

        impl #path::Type for #ty {
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

        impl #path::Enum for #ty {
            fn variant(&self) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                use ::std::fmt::Write;

                let mut out = ::std::string::String::new();
                ::std::write!(&mut out, "{:?}", self).unwrap();

                ::std::borrow::Cow::Owned(out)
            }

            fn update_variant(
                &mut self,
                variant: &::std::primitive::str,
            ) -> ::std::primitive::bool {
                let mut new = Self::empty();
                for bit in variant.split('|').map(|b| b.trim()) {
                    match bit {
                        #(#bit_names => new |= #ty::#bit_idents,)*

                        _ => return false,
                    }
                }

                *self = new;
                true
            }

            fn value(&self) -> ::std::primitive::u32 {
                Self::bits(self)
            }

            fn update_value(&mut self, value: ::std::primitive::u32) -> ::std::primitive::bool {
                *self = match Self::from_bits(value) {
                    ::std::option::Option::Some(v) => v,

                    _ => return false,
                };

                true
            }
        }
    })
}
