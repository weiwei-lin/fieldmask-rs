#![allow(dead_code)]

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    Attribute, Expr, Generics, Ident, Meta, Path, Token, Type, Visibility, braced, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::{Brace, Paren},
};

struct Wrap<T>(pub T);

impl<T: Parse> Parse for Wrap<Punctuated<T, Token![,]>> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self(input.parse_terminated(T::parse, Token![,])?))
    }
}

/// Represents the input.
pub enum Input {
    UnitEnum(ItemUnitEnum),
    TupleEnum(ItemTupleEnum),
    Struct(ItemStruct),
}

impl Input {
    pub fn get_message_info(&self) -> MessageInfo {
        match &self {
            Input::UnitEnum(input) => input.get_info(),
            Input::TupleEnum(input) => input.get_info(),
            Input::Struct(input) => input.get_info(),
        }
    }
}

impl Parse for Input {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;

        let vis = input.parse()?;

        let lookahead = input.lookahead1();
        if lookahead.peek(Token![struct]) {
            assert_no_attr(&attrs)?;

            let struct_token = input.parse()?;
            let ident = input.parse()?;
            let generics = {
                let mut generics: Generics = input.parse()?;
                generics.where_clause = input.parse()?;
                generics
            };

            let content;
            let brace_token = braced!(content in input);
            let fields = content.parse_terminated(NamedField::parse, Token![,])?;

            return Ok(Self::Struct(ItemStruct {
                attrs,
                vis,
                struct_token,
                ident,
                generics,
                brace_token,
                fields,
            }));
        }
        if lookahead.peek(Token![enum]) {
            let enum_token = input.parse()?;
            let ident = input.parse()?;
            let generics = {
                let mut generics: Generics = input.parse()?;
                generics.where_clause = input.parse()?;
                generics
            };

            let content;
            let brace_token = braced!(content in input);
            let first_variant: EnumVariant = content.parse()?;
            let _: Option<Token![,]> = content.parse()?;
            match first_variant {
                EnumVariant::Unit(first_variant) => {
                    let attr_iter = attrs
                        .iter()
                        .filter(|attr| attr.path().is_ident("fieldmask"))
                        .map(|attr| attr.parse_args())
                        .collect::<syn::Result<Vec<_>>>()?
                        .into_iter()
                        .flat_map(|attrs: Wrap<Punctuated<UnitEnumAttribute, Token![,]>>| attrs.0);

                    let mut variants =
                        content.parse_terminated(UnitEnumVariant::parse, Token![,])?;
                    variants.insert(0, first_variant);

                    let mut normalize_some_default = false;
                    for attr in attr_iter {
                        match attr {
                            UnitEnumAttribute::NormalizeSomeDefault { .. } => {
                                if normalize_some_default {
                                    return Err(syn::Error::new_spanned(
                                        attr,
                                        "duplicated normalize_some_default attribute",
                                    ));
                                }
                                normalize_some_default = true;
                            }
                        }
                    }

                    return Ok(Self::UnitEnum(ItemUnitEnum {
                        attrs,
                        normalize_some_default,
                        vis,
                        enum_token,
                        ident,
                        generics,
                        brace_token,
                        variants,
                    }));
                }
                EnumVariant::Tuple(first_variant) => {
                    assert_no_attr(&attrs)?;

                    let mut variants =
                        content.parse_terminated(TupleEnumVariant::parse, Token![,])?;
                    variants.insert(0, first_variant);
                    return Ok(Self::TupleEnum(ItemTupleEnum {
                        attrs,
                        vis,
                        enum_token,
                        ident,
                        generics,
                        brace_token,
                        variants,
                    }));
                }
            }
        }

        Err(lookahead.error())
    }
}

/// Represents the declaration of a unit enum.
pub struct ItemUnitEnum {
    pub attrs: Vec<Attribute>,
    pub normalize_some_default: bool,
    pub vis: Visibility,
    pub enum_token: Token![enum],
    pub ident: Ident,
    pub generics: Generics,
    pub brace_token: Brace,
    pub variants: Punctuated<UnitEnumVariant, Token![,]>,
}

impl ItemUnitEnum {
    pub fn get_info(&self) -> MessageInfo {
        let ident = &self.ident;
        let generics = &self.generics;

        MessageInfo {
            ident,
            generics,
            fields: vec![],
        }
    }
}

/// Represents an attribute for a unit enum.
#[derive(PartialEq)]
enum UnitEnumAttribute {
    NormalizeSomeDefault { repr: Path },
}

impl Parse for UnitEnumAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let meta: Meta = input.parse()?;
        match meta {
            Meta::Path(p) if p.is_ident("normalize_some_default") => {
                Ok(Self::NormalizeSomeDefault { repr: p })
            }
            _ => Err(syn::Error::new_spanned(meta, "invalid meta")),
        }
    }
}

impl ToTokens for UnitEnumAttribute {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::NormalizeSomeDefault { repr } => repr.to_tokens(tokens),
        }
    }
}

/// Represents the declaration of a tuple enum.
pub struct ItemTupleEnum {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub enum_token: Token![enum],
    pub ident: Ident,
    pub generics: Generics,
    pub brace_token: Brace,
    pub variants: Punctuated<TupleEnumVariant, Token![,]>,
}

impl ItemTupleEnum {
    pub fn get_info(&self) -> MessageInfo {
        let ident = &self.ident;
        let generics = &self.generics;

        MessageInfo {
            ident,
            generics,
            fields: self
                .variants
                .iter()
                .map(|v| MessageField {
                    ident: &v.ident,
                    ty: &v.ty,
                    is_flatten: false,
                })
                .collect::<Vec<_>>(),
        }
    }
}

/// Represents the declaration of a struct.
pub struct ItemStruct {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub struct_token: Token![struct],
    pub ident: Ident,
    pub generics: Generics,
    pub brace_token: Brace,
    pub fields: Punctuated<NamedField, Token![,]>,
}

impl ItemStruct {
    pub fn get_info(&self) -> MessageInfo {
        let ident = &self.ident;
        let generics = &self.generics;
        let fields = self
            .fields
            .iter()
            .map(|f| MessageField {
                ident: &f.ident,
                ty: &f.ty,
                is_flatten: f.is_flatten,
            })
            .collect::<Vec<_>>();
        MessageInfo {
            ident,
            generics,
            fields,
        }
    }
}

/// Represents the declaration of a variant in an enum.
pub enum EnumVariant {
    Unit(UnitEnumVariant),
    Tuple(TupleEnumVariant),
}

impl Parse for EnumVariant {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let ident: Ident = input.parse()?;

        if input.peek(Paren) {
            Ok(Self::Tuple(TupleEnumVariant::parse_content(
                attrs, ident, input,
            )?))
        } else {
            Ok(Self::Unit(UnitEnumVariant::parse_content(
                attrs, ident, input,
            )?))
        }
    }
}

/// Represents the declaration of a variant in a unit enum.
pub struct UnitEnumVariant {
    pub attrs: Vec<Attribute>,
    pub ident: Ident,
    pub discriminant: Option<(Token![=], Expr)>,
}

impl UnitEnumVariant {
    fn parse_content(attrs: Vec<Attribute>, ident: Ident, input: ParseStream) -> syn::Result<Self> {
        assert_no_attr(&attrs)?;

        let discriminant = if input.peek(Token![=]) {
            let eq_token = input.parse()?;
            let discriminant = input.parse()?;
            Some((eq_token, discriminant))
        } else {
            None
        };

        Ok(UnitEnumVariant {
            attrs,
            ident,
            discriminant,
        })
    }
}

impl Parse for UnitEnumVariant {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let ident: Ident = input.parse()?;
        Self::parse_content(attrs, ident, input)
    }
}

/// Represents the declaration of a variant in a tuple enum.
pub struct TupleEnumVariant {
    pub attrs: Vec<Attribute>,
    pub ident: Ident,
    pub paren_token: Paren,
    pub tuple_attrs: Vec<Attribute>,
    pub ty: Type,
}

impl TupleEnumVariant {
    fn parse_content(attrs: Vec<Attribute>, ident: Ident, input: ParseStream) -> syn::Result<Self> {
        assert_no_attr(&attrs)?;

        let content;
        let paren_token = parenthesized!(content in input);
        let tuple_attrs = content.call(Attribute::parse_outer)?;
        let ty = content.parse()?;

        if !content.is_empty() {
            let _punt: Token![,] = content.parse()?;
            if !content.is_empty() {
                return Err(content.error("there can be at most one item in the tuple variant"));
            }
        }

        Ok(TupleEnumVariant {
            attrs,
            ident,
            paren_token,
            tuple_attrs,
            ty,
        })
    }
}

impl Parse for TupleEnumVariant {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let ident: Ident = input.parse()?;
        Self::parse_content(attrs, ident, input)
    }
}

/// Represents the declaration of a named field in a struct.
pub struct NamedField {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub ident: Ident,
    pub colon_token: Token![:],
    pub ty: Type,
    pub is_flatten: bool,
}

impl Parse for NamedField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;

        #[allow(unused_assignments)]
        let mut is_flatten = false;

        #[cfg(feature = "prost")]
        {
            is_flatten = attrs
                .iter()
                .filter(|attr| attr.path().is_ident("prost"))
                .map(|attr| attr.parse_args())
                .collect::<syn::Result<Vec<_>>>()?
                .iter()
                .flat_map(|attrs: &Wrap<Punctuated<ProstFieldAttribute, Token![,]>>| &attrs.0)
                .any(|meta| matches!(meta, ProstFieldAttribute::OneOf));
        }

        let attr_iter = attrs
            .iter()
            .filter(|attr| attr.path().is_ident("fieldmask"))
            .map(|attr| attr.parse_args())
            .collect::<syn::Result<Vec<_>>>()?
            .into_iter()
            .flat_map(|attrs: Wrap<Punctuated<NamedFieldAttribute, Token![,]>>| attrs.0);

        for attr in attr_iter {
            match attr {
                NamedFieldAttribute::Flatten { .. } => {
                    if is_flatten {
                        return Err(syn::Error::new_spanned(
                            attr,
                            "duplicated flatten attribute",
                        ));
                    }
                    is_flatten = true;
                }
            }
        }

        Ok(NamedField {
            attrs,
            vis: input.parse()?,
            ident: input.parse()?,
            colon_token: input.parse()?,
            ty: input.parse()?,
            is_flatten,
        })
    }
}

/// Represents an attribute for a named field in a struct.
#[derive(PartialEq)]
enum NamedFieldAttribute {
    Flatten { repr: Path },
}

impl Parse for NamedFieldAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let meta: Meta = input.parse()?;
        match meta {
            Meta::Path(p) if p.is_ident("flatten") => Ok(Self::Flatten { repr: p }),
            _ => Err(syn::Error::new_spanned(meta, "invalid meta")),
        }
    }
}

impl ToTokens for NamedFieldAttribute {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Flatten { repr } => repr.to_tokens(tokens),
        }
    }
}

/// Represents a prost attribute for a named field in a struct.
#[derive(PartialEq)]
#[non_exhaustive]
#[cfg(feature = "prost")]
enum ProstFieldAttribute {
    OneOf,
    Other,
}

#[cfg(feature = "prost")]
impl Parse for ProstFieldAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let meta: Meta = input.parse()?;
        match meta {
            Meta::NameValue(m) if m.path.is_ident("oneof") => Ok(Self::OneOf),
            _ => Ok(Self::Other),
        }
    }
}

/// The metadata of a field in a message.
pub struct MessageField<'a> {
    pub ident: &'a Ident,
    pub ty: &'a Type,
    pub is_flatten: bool,
}

/// The metadata of a message.
pub struct MessageInfo<'a> {
    pub ident: &'a Ident,
    pub generics: &'a Generics,
    /// The fields of the message.
    /// Note that unit enum is considered a single value so it does not have any field.
    pub fields: Vec<MessageField<'a>>,
}

fn assert_no_attr(attrs: &[Attribute]) -> syn::Result<()> {
    let _attr_iter = attrs
        .iter()
        .filter(|attr| attr.path().is_ident("fieldmask"))
        .map(|attr| attr.parse_args())
        .collect::<syn::Result<Vec<_>>>()?
        .into_iter()
        .flat_map(|attrs: Wrap<Punctuated<NoAttribute, Token![,]>>| attrs.0);
    Ok(())
}

/// Represents an attribute for an item that should not have any attribute.
#[derive(PartialEq)]
enum NoAttribute {}

impl Parse for NoAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let meta: Meta = input.parse()?;
        Err(syn::Error::new_spanned(meta, "invalid meta"))
    }
}

impl ToTokens for NoAttribute {
    fn to_tokens(&self, _tokens: &mut TokenStream) {}
}
