use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token, Attribute, Error, Ident, LitStr, Result, Token,
};

mod kw {
    syn::custom_keyword!(op_crate);
    syn::custom_keyword!(on_pre_save);
    syn::custom_keyword!(on_post_save);
    syn::custom_keyword!(on_pre_load);
    syn::custom_keyword!(on_post_load);
    syn::custom_keyword!(base);
    syn::custom_keyword!(name);
    syn::custom_keyword!(info);
    syn::custom_keyword!(flags);
}

pub struct Attrs<'a> {
    pub op_crate: Option<CrateAttr<'a>>,
    pub object: Option<ObjectAttr<'a>>,
    pub property: Option<PropertyAttr<'a>>,
    pub option: Option<OptionAttr<'a>>,
}

pub fn get(input: &[Attribute]) -> Result<Attrs<'_>> {
    let mut attrs = Attrs {
        op_crate: None,
        object: None,
        property: None,
        option: None,
    };

    for attr in input {
        if attr.path.is_ident("object_property_crate") {
            parse_crate_attr(&mut attrs, attr)?;
        } else if attr.path.is_ident("object") {
            parse_object_attr(&mut attrs, attr)?;
        } else if attr.path.is_ident("property") {
            parse_property_attr(&mut attrs, attr)?;
        } else if attr.path.is_ident("option") {
            parse_option_attr(&mut attrs, attr)?;
        }
    }

    Ok(attrs)
}

/// #[op_crate = ...]
///
/// Only on structs and enums.
pub struct CrateAttr<'a> {
    pub original: &'a Attribute,
    pub krate: syn::Path,
}

fn parse_crate_attr<'a>(attrs: &mut Attrs<'a>, attr: &'a Attribute) -> Result<()> {
    if attrs.op_crate.is_some() {
        return Err(Error::new_spanned(
            attr,
            "duplicate #[op_crate] attribute found",
        ));
    }

    attrs.op_crate = Some(CrateAttr {
        original: attr,
        krate: attr.parse_args_with(parse_mod_path::<kw::op_crate>)?,
    });
    Ok(())
}

/// #[object(..)]
///
/// Only on structs.
pub struct ObjectAttr<'a> {
    pub original: &'a Attribute,
    name: Option<LitStr>,
    pub on_pre_save: Option<syn::Path>,
    pub on_post_save: Option<syn::Path>,
    pub on_pre_load: Option<syn::Path>,
    pub on_post_load: Option<syn::Path>,
}

impl<'a> ObjectAttr<'a> {
    pub fn name(&self) -> Option<String> {
        self.name.as_ref().map(|name| name.value())
    }
}

fn parse_object_attr<'a>(attrs: &mut Attrs<'a>, attr: &'a Attribute) -> Result<()> {
    if attrs.object.is_none() {
        attrs.object = Some(ObjectAttr {
            original: attr,
            name: None,
            on_pre_save: None,
            on_post_save: None,
            on_pre_load: None,
            on_post_load: None,
        });
    }
    let object = attrs.object.as_mut().unwrap();

    attr.parse_args_with(|input: ParseStream<'_>| {
        let mut first = true;
        while !input.is_empty() {
            if !first {
                input.parse::<Token![,]>()?;
            }

            let look = input.lookahead1();
            if look.peek(kw::name) {
                if object.name.is_some() {
                    return Err(Error::new_spanned(
                        attr,
                        "duplicate #[object(name)] attribute found",
                    ));
                }

                let AttrWrapper::<kw::name, LitStr> { value: name, .. } = input.parse()?;
                object.name = Some(name);
            } else if look.peek(kw::on_pre_save) {
                if object.on_pre_save.is_some() {
                    return Err(Error::new_spanned(
                        attr,
                        "duplicate #[object(on_pre_save)] attribute found",
                    ));
                }

                object.on_pre_save = Some(parse_mod_path::<kw::on_pre_save>(input)?);
            } else if look.peek(kw::on_post_save) {
                if object.on_post_save.is_some() {
                    return Err(Error::new_spanned(
                        attr,
                        "duplicate #[object(on_post_save)] attribute found",
                    ));
                }

                object.on_post_save = Some(parse_mod_path::<kw::on_post_save>(input)?);
            } else if look.peek(kw::on_pre_load) {
                if object.on_pre_load.is_some() {
                    return Err(Error::new_spanned(
                        attr,
                        "duplicate #[object(on_pre_load)] attribute found",
                    ));
                }

                object.on_pre_load = Some(parse_mod_path::<kw::on_pre_load>(input)?);
            } else if look.peek(kw::on_post_load) {
                if object.on_post_load.is_some() {
                    return Err(Error::new_spanned(
                        attr,
                        "duplicate #[object(on_post_load)] attribute found",
                    ));
                }

                object.on_post_load = Some(parse_mod_path::<kw::on_post_load>(input)?);
            } else {
                return Err(look.error());
            }

            first = false;
        }

        Ok(())
    })
}

/// #[property(..)]
///
/// Only on struct fields.
pub struct PropertyAttr<'a> {
    pub original: &'a Attribute,
    name: Option<LitStr>,
    pub base: bool,
    pub info: Option<syn::Expr>,
    pub flags: Option<Punctuated<Ident, Token![|]>>,
}

impl<'a> PropertyAttr<'a> {
    pub fn name(&self) -> Option<String> {
        self.name.as_ref().map(|name| name.value())
    }
}

fn parse_property_attr<'a>(attrs: &mut Attrs<'a>, attr: &'a Attribute) -> Result<()> {
    if attrs.property.is_none() {
        attrs.property = Some(PropertyAttr {
            original: attr,
            name: None,
            base: false,
            info: None,
            flags: None,
        });
    }
    let property = attrs.property.as_mut().unwrap();

    // Allow the `#[property]` notation to be accepted.
    if attr.tokens.is_empty() {
        return Ok(());
    }

    attr.parse_args_with(|input: ParseStream<'_>| {
        let mut first = true;
        while !input.is_empty() {
            if !first {
                input.parse::<Token![,]>()?;
            }

            let look = input.lookahead1();
            if look.peek(kw::base) {
                if property.base {
                    return Err(Error::new_spanned(
                        attr,
                        "duplicate #[property(base)] attribute found",
                    ));
                }

                input.parse::<kw::base>()?;
                property.base = true;
            } else if look.peek(kw::name) {
                if property.name.is_some() {
                    return Err(Error::new_spanned(
                        attr,
                        "duplicate #[property(name)] attribute found",
                    ));
                }

                let AttrWrapper::<kw::name, LitStr> { value: name, .. } = input.parse()?;
                property.name = Some(name);
            } else if look.peek(kw::info) {
                if property.info.is_some() {
                    return Err(Error::new_spanned(
                        attr,
                        "duplicate #[property(info)] attribute found",
                    ));
                }

                let AttrWrapper::<kw::info, syn::Expr> { value: info, .. } = input.parse()?;
                property.info = Some(info);
            } else if look.peek(kw::flags) {
                if property.flags.is_some() {
                    return Err(Error::new_spanned(
                        attr,
                        "duplicate #[property(flags)] attribute found",
                    ));
                }

                input.parse::<kw::flags>()?;
                let content;
                parenthesized!(content in input);
                let bits: Punctuated<Ident, Token![|]> = content.parse_terminated(Ident::parse)?;
                property.flags = Some(bits);
            } else {
                return Err(look.error());
            }

            first = false;
        }
        Ok(())
    })
}

/// #[option(..)]
///
/// Only on enum variants.
pub struct OptionAttr<'a> {
    pub original: &'a Attribute,
    name: Option<LitStr>,
}

impl<'a> OptionAttr<'a> {
    pub fn name(&self) -> Option<String> {
        self.name.as_ref().map(|name| name.value())
    }
}

fn parse_option_attr<'a>(attrs: &mut Attrs<'a>, attr: &'a Attribute) -> Result<()> {
    if attrs.option.is_none() {
        attrs.option = Some(OptionAttr {
            original: attr,
            name: None,
        });
    }
    let option = attrs.option.as_mut().unwrap();

    attr.parse_args_with(|input: ParseStream<'_>| {
        let look = input.lookahead1();
        if look.peek(kw::name) {
            if option.name.is_some() {
                return Err(Error::new_spanned(
                    attr,
                    "duplicate #[option(name)] attribute found",
                ));
            }

            let AttrWrapper::<kw::name, LitStr> { value: name, .. } = input.parse()?;
            option.name = Some(name);

            Ok(())
        } else {
            Err(look.error())
        }
    })
}

struct AttrWrapper<K, V> {
    #[allow(unused)] // Usually we don't care about the ident.
    pub ident: K,
    pub value: V,
}

impl<K: Parse, V: Parse> Parse for AttrWrapper<K, V> {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let ident = input.parse()?;
        let value = if input.peek(token::Paren) {
            // #[ident(value)]
            let value;
            parenthesized!(value in input);
            value.parse()?
        } else {
            // #[ident = value]
            input.parse::<Token![=]>()?;
            input.parse()?
        };

        Ok(Self { ident, value })
    }
}

fn parse_mod_path<K: Parse>(input: ParseStream<'_>) -> Result<syn::Path> {
    input.parse::<K>()?;
    if input.peek(token::Paren) {
        // #[ident(path)]
        let value;
        parenthesized!(value in input);
        value.call(syn::Path::parse_mod_style)
    } else {
        // #[ident = path]
        input.parse::<Token![=]>()?;
        input.call(syn::Path::parse_mod_style)
    }
}
