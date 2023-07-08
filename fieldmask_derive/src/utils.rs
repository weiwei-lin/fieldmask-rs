use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::{Brace, Paren},
    Attribute, Expr, Generics, Ident, Meta, NestedMeta, Token, Type, Visibility,
};

#[cfg(feature = "prost")]
use syn::Lit;

struct Wrap<T>(pub T);

impl<T: Parse> Parse for Wrap<Punctuated<T, Token![,]>> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self(input.parse_terminated(T::parse)?))
    }
}

pub enum Item {
    Struct(ItemStruct),
    Enum(ItemEnum),
}

pub enum ItemType {
    Struct,
    Enum,
    UnitEnum,
}

pub struct ItemStruct {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub struct_token: Token![struct],
    pub ident: Ident,
    pub generics: Generics,
    pub brace_token: Brace,
    pub fields: Punctuated<NamedField, Token![,]>,
}

pub struct ItemEnum {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub enum_token: Token![enum],
    pub ident: Ident,
    pub generics: Generics,
    pub brace_token: Brace,
    pub variants: Punctuated<EnumVariant, Token![,]>,
}

pub struct NamedField {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub ident: Ident,
    pub colon_token: Token![:],
    pub ty: Type,
    pub is_flatten: bool,
}

pub struct SingleTupleVariant {
    pub attrs: Vec<Attribute>,
    pub ident: Ident,
    pub paren_token: Paren,
    pub tuple_attrs: Vec<Attribute>,
    pub ty: Type,
}

pub struct UnitVariant {
    pub attrs: Vec<Attribute>,
    pub ident: Ident,
    pub discriminant: Option<(Token![=], Expr)>,
}

pub enum EnumVariant {
    Tuple(SingleTupleVariant),
    Unit(UnitVariant),
}

impl Parse for EnumVariant {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let _visibility: Visibility = input.parse()?;
        let ident: Ident = input.parse()?;

        if input.peek(Paren) {
            let content;
            Ok(Self::Tuple(SingleTupleVariant {
                attrs,
                ident,
                paren_token: parenthesized!(content in input),
                tuple_attrs: content.call(Attribute::parse_outer)?,
                ty: {
                    let _vis: Visibility = content.parse()?;
                    let ty = content.parse()?;
                    if !content.is_empty() {
                        let _punt: Token![,] = content.parse()?;
                        if !content.is_empty() {
                            return Err(
                                content.error("there can be at most one item in the tuple variant")
                            );
                        }
                    }
                    ty
                },
            }))
        } else {
            Ok(Self::Unit(UnitVariant {
                attrs,
                ident,
                discriminant: if input.peek(Token![=]) {
                    let eq_token = input.parse()?;
                    let discriminant = input.parse()?;
                    Some((eq_token, discriminant))
                } else {
                    None
                },
            }))
        }
    }
}

impl Parse for Item {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;

        let vis = input.parse()?;

        let lookahead = input.lookahead1();
        if lookahead.peek(Token![struct]) {
            let content;
            Ok(Self::Struct(ItemStruct {
                attrs,
                vis,
                struct_token: input.parse()?,
                ident: input.parse()?,
                generics: {
                    let mut generics: Generics = input.parse()?;
                    generics.where_clause = input.parse()?;
                    generics
                },
                brace_token: braced!(content in input),
                fields: content.parse_terminated(NamedField::parse)?,
            }))
        } else if lookahead.peek(Token![enum]) {
            let content;
            Ok(Self::Enum(ItemEnum {
                attrs,
                vis,
                enum_token: input.parse()?,
                ident: input.parse()?,
                generics: {
                    let mut generics: Generics = input.parse()?;
                    generics.where_clause = input.parse()?;
                    generics
                },
                brace_token: braced!(content in input),
                variants: content.parse_terminated(EnumVariant::parse)?,
            }))
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(PartialEq)]
enum NamedFieldAttribute {
    Flatten,
}

impl Parse for NamedFieldAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let meta: NestedMeta = input.parse()?;
        match meta {
            NestedMeta::Meta(Meta::Path(p)) if p.is_ident("flatten") => Ok(Self::Flatten),
            _ => Err(syn::Error::new_spanned(meta, "invalid meta")),
        }
    }
}

#[derive(PartialEq)]
#[non_exhaustive]
#[cfg(feature = "prost")]
enum ProstFieldAttribute {
    OneOf(Lit),
    Other,
}

#[cfg(feature = "prost")]
impl Parse for ProstFieldAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let meta: NestedMeta = input.parse()?;
        match meta {
            NestedMeta::Meta(Meta::NameValue(m)) if m.path.is_ident("oneof") => {
                Ok(Self::OneOf(m.lit))
            }
            _ => Ok(Self::Other),
        }
    }
}

impl Parse for NamedField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;

        #[allow(unused_mut)]
        let mut is_flatten = attrs
            .iter()
            .filter(|attr| attr.path.is_ident("fieldmask"))
            .map(|attr| attr.parse_args())
            .collect::<syn::Result<Vec<_>>>()?
            .iter()
            .flat_map(|attrs: &Wrap<Punctuated<NamedFieldAttribute, Token![,]>>| &attrs.0)
            .any(|meta| *meta == NamedFieldAttribute::Flatten);

        #[cfg(feature = "prost")]
        {
            is_flatten = is_flatten
                || attrs
                    .iter()
                    .filter(|attr| attr.path.is_ident("prost"))
                    .map(|attr| attr.parse_args())
                    .collect::<syn::Result<Vec<_>>>()?
                    .iter()
                    .flat_map(|attrs: &Wrap<Punctuated<ProstFieldAttribute, Token![,]>>| &attrs.0)
                    .any(|meta| match meta {
                        ProstFieldAttribute::OneOf(_) => true,
                        _ => false,
                    });
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

pub struct Field<'a> {
    pub ident: &'a Ident,
    pub ty: &'a Type,
    pub is_flatten: bool,
}

pub struct ItemInfo<'a> {
    pub item_type: ItemType,
    pub ident: &'a Ident,
    pub generics: &'a Generics,
    pub fields: Vec<Field<'a>>,
}

impl ItemEnum {
    pub fn get_info(&self) -> ItemInfo {
        let ident = &self.ident;
        let generics = &self.generics;
        if self
            .variants
            .iter()
            .all(|v| matches!(v, EnumVariant::Tuple(_)))
        {
            let fields = self
                .variants
                .iter()
                .map(|v| match v {
                    EnumVariant::Tuple(v) => Field {
                        ident: &v.ident,
                        ty: &v.ty,
                        is_flatten: false,
                    },
                    _ => unreachable!(),
                })
                .collect::<Vec<_>>();
            ItemInfo {
                item_type: ItemType::Enum,
                ident,
                generics,
                fields,
            }
        } else if self
            .variants
            .iter()
            .all(|v| matches!(v, EnumVariant::Unit(_)))
        {
            ItemInfo {
                item_type: ItemType::UnitEnum,
                ident,
                generics,
                fields: Vec::default(),
            }
        } else {
            panic!("all enum variants must be the same type: unit or single-field tuple")
        }
    }
}

impl ItemStruct {
    pub fn get_info(&self) -> ItemInfo {
        let ident = &self.ident;
        let generics = &self.generics;
        let fields = self
            .fields
            .iter()
            .map(|f| Field {
                ident: &f.ident,
                ty: &f.ty,
                is_flatten: f.is_flatten,
            })
            .collect::<Vec<_>>();
        ItemInfo {
            item_type: ItemType::Struct,
            ident,
            generics,
            fields,
        }
    }
}

impl Item {
    pub fn get_info(&self) -> ItemInfo {
        match &self {
            Item::Enum(input) => input.get_info(),
            Item::Struct(input) => input.get_info(),
        }
    }
}
