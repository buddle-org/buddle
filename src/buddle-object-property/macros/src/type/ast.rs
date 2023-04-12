use quote::quote;
use syn::{
    Data, DataEnum, DataStruct, DeriveInput, Error, Expr, Generics, Ident, Member, Path, Result,
    Type,
};

use super::attrs::{self, Attrs};

pub enum Input<'a> {
    Struct(Struct<'a>),
    Enum(Enum<'a>),
}

impl<'a> Input<'a> {
    pub fn from_syn(node: &'a DeriveInput) -> Result<Self> {
        match &node.data {
            Data::Struct(data) => Struct::from_syn(node, data).map(Input::Struct),
            Data::Enum(data) => Enum::from_syn(node, data).map(Input::Enum),
            Data::Union(_) => Err(Error::new_spanned(
                node,
                "union reflection is not supported",
            )),
        }
    }

    pub fn validate(&self) -> Result<()> {
        match self {
            Input::Struct(input) => input.validate(),
            Input::Enum(input) => input.validate(),
        }
    }
}

pub struct Struct<'a> {
    pub original: &'a DeriveInput,
    pub attrs: Attrs<'a>,
    pub ident: Ident,
    pub generics: &'a Generics,
    pub fields: Vec<Field<'a>>,
}

pub struct Field<'a> {
    pub original: &'a syn::Field,
    pub attrs: Attrs<'a>,
    pub ident: Ident,
    pub ty: &'a Type,
}

impl Field<'_> {
    pub fn is_base(&self) -> bool {
        // If this field has a `#[property(base)]` attribute configured,
        // it holds the base class of the containing type.
        self.attrs
            .property
            .as_ref()
            .map(|p| p.base)
            .unwrap_or(false)
    }
}

pub struct Enum<'a> {
    pub original: &'a DeriveInput,
    pub attrs: Attrs<'a>,
    pub ident: Ident,
    pub generics: &'a Generics,
    pub variants: Vec<Variant<'a>>,
}

pub struct Variant<'a> {
    pub original: &'a syn::Variant,
    pub attrs: Attrs<'a>,
    pub ident: Ident,
    pub discriminant: Expr,
}

impl<'a> Struct<'a> {
    fn from_syn(node: &'a DeriveInput, data: &'a DataStruct) -> Result<Self> {
        Ok(Self {
            original: node,
            attrs: attrs::get(&node.attrs)?,
            ident: node.ident.clone(),
            generics: &node.generics,
            fields: Field::multiple_from_syn(&data.fields)?,
        })
    }

    fn validate(&self) -> Result<()> {
        require_no_property(&self.attrs)?;
        require_no_option(&self.attrs)?;

        for field in &self.fields {
            field.validate()?;
        }

        Ok(())
    }

    pub fn on_pre_load(&self) -> Option<&Path> {
        self.attrs
            .object
            .as_ref()
            .and_then(|o| o.on_pre_load.as_ref())
    }

    pub fn on_post_load(&self) -> Option<&Path> {
        self.attrs
            .object
            .as_ref()
            .and_then(|o| o.on_post_load.as_ref())
    }

    pub fn on_pre_save(&self) -> Option<&Path> {
        self.attrs
            .object
            .as_ref()
            .and_then(|o| o.on_pre_save.as_ref())
    }

    pub fn on_post_save(&self) -> Option<&Path> {
        self.attrs
            .object
            .as_ref()
            .and_then(|o| o.on_post_save.as_ref())
    }
}

impl<'a> Enum<'a> {
    fn from_syn(node: &'a DeriveInput, data: &'a DataEnum) -> Result<Self> {
        Ok(Self {
            original: node,
            attrs: attrs::get(&node.attrs)?,
            ident: node.ident.clone(),
            generics: &node.generics,
            variants: data
                .variants
                .iter()
                .map(Variant::from_syn)
                .collect::<Result<_>>()?,
        })
    }

    fn validate(&self) -> Result<()> {
        require_no_object(&self.attrs)?;
        require_no_option(&self.attrs)?;
        require_no_property(&self.attrs)?;

        for variant in &self.variants {
            variant.validate()?;
        }

        Ok(())
    }
}

impl<'a> Field<'a> {
    fn from_syn(node: &'a syn::Field) -> Result<Self> {
        Ok(Self {
            original: node,
            attrs: attrs::get(&node.attrs)?,
            ident: if let Member::Named(member) =
                node.ident.clone().map(Member::Named).ok_or_else(|| {
                    Error::new_spanned(
                        node,
                        "tuple struct fields are unsupported in the ObjectProperty data model",
                    )
                })? {
                member
            } else {
                unreachable!()
            },
            ty: &node.ty,
        })
    }

    fn multiple_from_syn(fields: &'a syn::Fields) -> Result<Vec<Self>> {
        fields.iter().map(Field::from_syn).collect()
    }

    fn validate(&self) -> Result<()> {
        require_no_crate(&self.attrs)?;
        require_no_object(&self.attrs)?;
        require_no_option(&self.attrs)
    }

    pub fn name(&self) -> String {
        self.attrs
            .property
            .as_ref()
            .and_then(|p| p.name())
            .unwrap_or_else(|| self.ident.to_string())
    }

    pub fn info(&self, path: &syn::Path) -> syn::Expr {
        self.attrs
            .property
            .as_ref()
            .and_then(|p| p.info.clone())
            .unwrap_or_else(|| {
                let ty = self.ty;

                let expr = quote!(<#ty as #path::type_info::Reflected>::TYPE_INFO);
                syn::parse2(expr).unwrap()
            })
    }

    pub fn flags(&self) -> Vec<&Ident> {
        self.attrs
            .property
            .as_ref()
            .and_then(|p| p.flags.as_ref().map(|f| f.iter().collect()))
            .unwrap_or_default()
    }
}

impl<'a> Variant<'a> {
    fn from_syn(node: &'a syn::Variant) -> Result<Self> {
        if !node.fields.is_empty() {
            return Err(Error::new_spanned(
                node,
                "enum fields are unsupported in the ObjectProperty data model",
            ));
        }

        Ok(Self {
            original: node,
            attrs: attrs::get(&node.attrs)?,
            ident: node.ident.clone(),
            discriminant: node
                .discriminant
                .clone()
                .map(|(_, expr)| expr)
                .ok_or_else(|| {
                    Error::new_spanned(
                        node,
                        "enum variants must have expicit discriminants assigned",
                    )
                })?,
        })
    }

    fn validate(&self) -> Result<()> {
        require_no_crate(&self.attrs)?;
        require_no_object(&self.attrs)?;
        require_no_property(&self.attrs)
    }

    pub fn name(&self) -> String {
        self.attrs
            .option
            .as_ref()
            .and_then(|o| o.name())
            .unwrap_or_else(|| self.ident.to_string())
    }
}

fn require_no_crate(attrs: &Attrs<'_>) -> Result<()> {
    if let Some(krate) = &attrs.op_crate {
        return Err(Error::new_spanned(
            krate.original,
            "unexpected #[op_crate] attribute only allowed on structs and enums",
        ));
    }
    Ok(())
}

fn require_no_object(attrs: &Attrs<'_>) -> Result<()> {
    if let Some(object) = &attrs.object {
        return Err(Error::new_spanned(
            object.original,
            "unexpected #[object] attribute only allowed on structs",
        ));
    }
    Ok(())
}

fn require_no_property(attrs: &Attrs<'_>) -> Result<()> {
    if let Some(property) = &attrs.property {
        return Err(Error::new_spanned(
            property.original,
            "unexpected #[property] attribute only allowed on struct fields",
        ));
    }
    Ok(())
}

fn require_no_option(attrs: &Attrs<'_>) -> Result<()> {
    if let Some(option) = &attrs.option {
        return Err(Error::new_spanned(
            option.original,
            "unexpected #[option] attribute only allowed on enum variants",
        ));
    }
    Ok(())
}
