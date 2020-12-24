use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::{Brace, Paren},
    Attribute, Generics, Ident, Meta, NestedMeta, Token, Type, Visibility,
};

pub enum Item {
    Struct(ItemStruct),
    Enum(ItemEnum),
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
    pub variants: Punctuated<SingleTupleVariant, Token![,]>,
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
                variants: content.parse_terminated(SingleTupleVariant::parse)?,
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

struct NamedFieldAttributes {
    attrs: Punctuated<NamedFieldAttribute, Token![,]>,
}

impl Parse for NamedFieldAttributes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            attrs: input.parse_terminated(NamedFieldAttribute::parse)?,
        })
    }
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

impl Parse for NamedField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let is_flatten = attrs
            .iter()
            .filter(|attr| attr.path.is_ident("fieldmask"))
            .map(|attr| attr.parse_args())
            .collect::<syn::Result<Vec<_>>>()?
            .iter()
            .flat_map(|attrs: &NamedFieldAttributes| &attrs.attrs)
            .any(|meta| *meta == NamedFieldAttribute::Flatten);
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

impl Parse for SingleTupleVariant {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(SingleTupleVariant {
            attrs: input.call(Attribute::parse_outer)?,
            ident: {
                let _vis: Visibility = input.parse()?;
                input.parse()?
            },
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
        })
    }
}

pub struct Field<'a> {
    pub ident: &'a Ident,
    pub ty: &'a Type,
    pub is_flatten: bool,
}

pub struct ItemInfo<'a> {
    pub ident: &'a Ident,
    pub generics: &'a Generics,
    pub fields: Vec<Field<'a>>,
}

impl ItemEnum {
    pub fn get_info(&self) -> ItemInfo {
        let ident = &self.ident;
        let generics = &self.generics;
        let fields = self
            .variants
            .iter()
            .map(|v| Field {
                ident: &v.ident,
                ty: &v.ty,
                is_flatten: false,
            })
            .collect::<Vec<_>>();
        ItemInfo {
            ident,
            generics,
            fields,
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
