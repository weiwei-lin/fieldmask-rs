use std::unimplemented;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident, Index, Meta};

#[proc_macro_derive(Maskable, attributes(flatten))]
pub fn derive_maskable(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let (impl_generics, ty_generics, where_clauses) = &ast.generics.split_for_impl();
    let name = &ast.ident;
    let fields = match &ast.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(fields) => fields
                .named
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    (
                        i,
                        f.ident.clone().expect("should be named"),
                        &f.ty,
                        &f.attrs,
                    )
                })
                .collect::<Vec<_>>(),
            _ => unimplemented!(),
        },
        Data::Enum(e) => {
            let a = e.variants.iter().enumerate().map(|(i, v)| {
                (
                    i,
                    Ident::new(&v.ident.to_string().to_lowercase(), v.ident.span()),
                    match &v.fields {
                        Fields::Unnamed(fields) => {
                            &fields
                                .unnamed
                                .first()
                                .expect("must have exactly one field")
                                .ty
                        }
                        _ => unimplemented!(),
                    },
                    &v.attrs,
                )
            });
            a.collect::<Vec<_>>()
        }
        _ => unimplemented!(),
    };
    let field_types1 = fields.iter().map(|(_i, _ident, ty, _attrs)| ty);
    let match_arms = fields.iter().map(|(i, ident, _ty, attrs)| {
        let index = Index::from(*i);
        let is_flatten = attrs
            .iter()
            .map(|attr| attr.parse_meta())
            .collect::<Result<Vec<_>, _>>()
            .expect("failed to parse meta")
            .iter()
            .any(|m| match m {
                Meta::Path(p) => {
                    p.get_ident().expect("unable to get ident")
                        == &Ident::new("flatten", Span::call_site())
                }
                _ => panic!("wrong meta"),
            });
        if is_flatten {
            quote! {
                _ if mask.0.#index.try_bitand_assign(field_mask_segs).map(|_| true).or_else(|e| if e== 0 { Ok(false)} else {Err(e)})? => (),
            }
        } else {
            quote! {
                [stringify!(#ident), tail @ ..] => mask.0.#index.try_bitand_assign(tail).map_err(|i| i + 1)?,
            }
        }
    });
    (quote! {
        impl#impl_generics ::fieldmask::Maskable for #name#ty_generics
        #where_clauses
        {
            type Mask = ::fieldmask::BitwiseWrap<(#(::fieldmask::FieldMask<#field_types1>,)*)>;

            fn deserialize_mask(
                mask: &mut Self::Mask,
                field_mask_segs: &[&::core::primitive::str],
            ) -> ::core::result::Result<(), u8> {
                match field_mask_segs {
                    [] => *mask = !Self::Mask::default(),
                    #(#match_arms)*
                    _ => return ::core::result::Result::Err(0),
                }
                Ok(())
            }
        }
    })
    .into()
}

#[proc_macro_derive(AbsoluteMaskable)]
pub fn derive_absolute_maskable(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let (impl_generics, ty_generics, where_clauses) = &ast.generics.split_for_impl();
    let name = &ast.ident;
    let fields = match ast.data {
        Data::Struct(s) => match s.fields {
            Fields::Named(fields) => fields.named,
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };
    let field_names1 = fields.iter().map(|field| &field.ident);
    let field_names2 = field_names1.clone();
    let field_indices1 = fields.iter().enumerate().map(|(i, _field)| Index::from(i));
    (quote! {
        impl#impl_generics ::fieldmask::AbsoluteMaskable for #name#ty_generics
        #where_clauses
        {
            fn apply_mask(&mut self, src: Self, mask: Self::Mask) {
                #(mask.0.#field_indices1.apply(&mut self.#field_names1, src.#field_names2);)*
            }
        }
    })
    .into()
}

#[proc_macro_derive(OptionalMaskable)]
pub fn derive_optional_maskable(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let (impl_generics, ty_generics, where_clauses) = &ast.generics.split_for_impl();
    let name = &ast.ident;
    let fields = match ast.data {
        Data::Enum(s) => s.variants,
        _ => unimplemented!(),
    };
    let field_names1 = fields.iter().map(|field| &field.ident);
    let match_clauses = fields.iter().map(|target_var| {
        let clauses = fields.iter().enumerate().map(|(i, src_var)| {
            let index = Index::from(i);
            let src_ident = src_var.ident.clone();
            if src_ident == target_var.ident {
                quote! {
                    Self::#src_ident(s) if mask.0.#index != ::fieldmask::FieldMask::default() => {
                        mask.0.#index.apply(t, s);
                    }
                }
            } else {
                let src_ty = src_var.fields.iter().next().unwrap().ty.clone();
                quote! {
                    Self::#src_ident(s) if mask.0.#index != ::fieldmask::FieldMask::default() => {
                        let mut new = #src_ty::default();
                        mask.0.#index.apply(&mut new, s);
                        *self = Self::#src_ident(new);
                    }
                }
            }
        });
        quote! {
            #(#clauses)*
        }
    });
    (quote! {
        impl#impl_generics ::fieldmask::OptionalMaskable for #name#ty_generics
        #where_clauses
        {
            fn apply_mask(&mut self, src: Self, mask: Self::Mask) -> bool {
                match self {
                    #(
                        Self::#field_names1(t) => match src {
                            #match_clauses
                            _ => return false,
                        }
                    )*
                }
                return true;
            }
        }
    })
    .into()
}
