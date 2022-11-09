use proc_macro2::TokenStream;
use syn::{
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    Path, Token,
};

use crate::utils::default_crate_path;

pub struct Input {
    pub krate: Option<Path>,
    pub item: Bits,
}

impl Parse for Input {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let krate = if input.peek(Token![#]) && input.peek2(Token![!]) {
            // #![crate = buddle_object_property]

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

        Ok(Input {
            krate,
            item: input.parse()?,
        })
    }
}

pub struct Bits {
    pub attrs: Vec<syn::Attribute>,
    pub vis: syn::Visibility,
    pub ident: syn::Ident,
    pub flags: Punctuated<Flag, Token![;]>,
}

impl Parse for Bits {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let vis = input.parse()?;
        input.parse::<Token![struct]>()?;
        let ident = input.parse()?;

        let content;
        syn::braced!(content in input);
        let flags = content.parse_terminated(Flag::parse)?;

        Ok(Self {
            attrs,
            vis,
            ident,
            flags,
        })
    }
}

pub struct Flag {
    pub attrs: Vec<syn::Attribute>,
    pub ident: syn::Ident,
    pub value: syn::Expr,
}

impl Parse for Flag {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;

        input.parse::<syn::Visibility>()?; // We tolerate explicit visibility.
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
        // We capture the correct type the derive macro was
        // used on and go with the default Rust type name.
        unsafe impl #path::type_info::Reflected for #ty {
            const TYPE_NAME: &'static str = Self::TYPE_INFO.type_name();

            const TYPE_INFO: &'static #path::type_info::TypeInfo =
                &#path::type_info::TypeInfo::leaf::<#ty>(
                    ::std::option::Option::None
                );
        }

        impl #path::Type for #ty {
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
            fn as_boxed_type(self: ::std::boxed::Box<Self>) -> ::std::boxed::Box<dyn #path::Type> {
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
            fn type_owned(self: ::std::boxed::Box<Self>) -> #path::TypeOwned {
                #path::TypeOwned::Enum(self)
            }

            #[inline]
            fn set(
                &mut self,
                value: ::std::boxed::Box<dyn #path::Type>,
            ) -> ::std::result::Result<(), ::std::boxed::Box<dyn #path::Type>> {
                *self = *value.downcast()?;
                ::std::result::Result::Ok(())
            }

            fn serialize(
                &self,
                serializer: &mut dyn #path::serde::ser::DynSerializer,
                baton: #path::serde::Baton,
            ) -> #path::serde::Result<()> {
                serializer.enum_variant(self, baton)
            }

            fn deserialize(
                &mut self,
                deserializer: &mut dyn #path::serde::de::DynDeserializer,
                baton: #path::serde::Baton,
            ) -> #path::serde::Result<()> {
                deserializer.enum_variant(self, baton)
            }
        }

        impl #path::Enum for #ty {
            fn variant(&self) -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                use ::std::fmt::Write;

                let mut out = String::new();
                ::std::write!(&mut out, "{:?}", self).unwrap();

                ::std::borrow::Cow::Owned(out)
            }

            fn update_variant(
                &mut self,
                variant: &::std::primitive::str,
            ) -> ::std::primitive::bool {
                if let ::std::option::Option::Some(value) = Self::from_variant(variant) {
                    *self = value;
                    true
                } else {
                    false
                }
            }

            fn from_variant(variant: &::std::primitive::str) -> ::std::option::Option<Self>
            where
                Self: ::std::marker::Sized
            {
                let mut flags = Self::empty();

                for bit in variant.split('|').map(|b| b.trim()) {
                    match bit {
                        #(#bit_names => flags |= #ty::#bit_idents,)*

                        _ => return ::std::option::Option::None,
                    }
                }

                ::std::option::Option::Some(flags)
            }

            fn value(&self) -> ::std::primitive::u32 {
                Self::bits(self)
            }

            fn update_value(
                &mut self,
                value: ::std::primitive::u32,
            ) -> ::std::primitive::bool {
                if let ::std::option::Option::Some(value) = Self::from_value(value) {
                    *self = value;
                    true
                } else {
                    false
                }
            }

            fn from_value(value: ::std::primitive::u32) -> ::std::option::Option<Self>
            where
                Self: ::std::marker::Sized
            {
                Self::from_bits(value)
            }
        }
    })
}
