#![allow(dead_code)]

use syn::{
    Generics, ImplItemFn, Token, Type, braced,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    token::Brace,
};

pub struct Input {
    pub impl_token: Token![impl],
    pub generics: Generics,
    pub ty: Type,
    pub brace_token: Brace,
    pub update_as_field_fn: Option<ImplItemFn>,
    pub merge_fn: Option<ImplItemFn>,
    pub option_project_fn: Option<ImplItemFn>,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let impl_token = input.parse()?;
        let mut generics: Generics = input.parse()?;
        let ty = input.parse()?;
        generics.where_clause = input.parse()?;
        let content;
        let brace_token = braced!(content in input);

        let mut ret = Self {
            impl_token,
            generics,
            ty,
            brace_token,
            update_as_field_fn: None,
            merge_fn: None,
            option_project_fn: None,
        };

        while content.peek(Token![fn]) {
            let item_fn: ImplItemFn = content.parse()?;
            match item_fn.sig.ident.to_string().as_str() {
                "update_as_field" => {
                    if ret.update_as_field_fn.is_some() {
                        return Err(syn::Error::new(
                            item_fn.span(),
                            "duplicated declaration of method `update_as_field`",
                        ));
                    }
                    ret.update_as_field_fn = Some(item_fn);
                }
                "merge" => {
                    if ret.merge_fn.is_some() {
                        return Err(syn::Error::new(
                            item_fn.span(),
                            "duplicated declaration of method `merge`",
                        ));
                    }
                    ret.merge_fn = Some(item_fn);
                }
                "option_project" => {
                    if ret.option_project_fn.is_some() {
                        return Err(syn::Error::new(
                            item_fn.span(),
                            "duplicated declaration of method `option_project`",
                        ));
                    }
                    ret.option_project_fn = Some(item_fn);
                }
                name => {
                    return Err(syn::Error::new(
                        item_fn.span(),
                        format!("unexpected method `{}`", name),
                    ));
                }
            }
        }

        Ok(ret)
    }
}
