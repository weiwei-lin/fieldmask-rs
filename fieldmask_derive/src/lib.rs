use std::unimplemented;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Index};

#[proc_macro_derive(Maskable)]
pub fn derive_maskable(input: TokenStream) -> TokenStream {
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
    let field_types1 = fields.iter().map(|field| &field.ty);
    let field_types2 = field_types1.clone();
    let field_names1 = fields.iter().map(|field| &field.ident);
    let field_names2 = field_names1.clone();
    let field_names3 = field_names1.clone();
    let field_indices1 = fields.iter().enumerate().map(|(i, _field)| Index::from(i));
    let field_indices2 = field_indices1.clone();
    (quote! {
        impl#impl_generics ::fieldmask::Maskable for #name#ty_generics
        #where_clauses
        {
            type Mask = ::fieldmask::BitwiseWrap<(#(::fieldmask::FieldMask<#field_types1>,)*)>;

            fn deserialize_mask_impl<'a, T: Iterator<Item = &'a ::core::primitive::str>>(
                field_mask: T,
            ) -> ::core::result::Result<Self::Mask, &'a ::core::primitive::str> {
                let mut mask = Self::Mask::default();
                for entry in field_mask {
                    match entry {
                        #(stringify!(#field_names1) => mask.0.#field_indices1 |= !::fieldmask::FieldMask::<#field_types2>::default(),)*
                        _ => return Err(entry),
                    }
                }
                Ok(mask)
            }

            fn apply_mask_impl(&mut self, other: Self, mask: Self::Mask) {
                #(
                    ::fieldmask::Maskable::apply_mask(&mut self.#field_names2, other.#field_names3, mask.0.#field_indices2);
                    
                )*
            }
        }
    })
    .into()
}
