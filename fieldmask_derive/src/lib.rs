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

            fn deserialize_mask<'a, I: ::core::iter::Iterator<Item = &'a str>>(
                mask: &mut Self::Mask,
                mut field_mask_segs: I,
            ) -> ::core::result::Result<(), ()> {
                let seg = ::core::iter::Iterator::next(&mut field_mask_segs);
                match seg {
                    None => *mask = !Self::Mask::default(),
                    #(Some(stringify!(#field_names1)) => mask.0.#field_indices1.try_bitand_assign(field_mask_segs)?,)*
                    Some(_) => return Err(()),
                }
                Ok(())
            }

            fn apply_mask(&mut self, src: Self, mask: Self::Mask) {
                #(mask.0.#field_indices2.apply(&mut self.#field_names2, src.#field_names3);)*
            }
        }
    })
    .into()
}
