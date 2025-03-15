use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::parse_macro_input;

use super::ast::Input;

pub fn maskable_atomic_impl(input: TokenStream) -> TokenStream {
    let Input {
        generics,
        ty,
        update_as_field_fn,
        merge_fn,
        ..
    } = parse_macro_input!(input);

    let (impl_generics, _ty_generics, where_clauses) = generics.split_for_impl();
    let update_as_field_fn = update_as_field_fn
        .map(|item| item.to_token_stream())
        .unwrap_or_else(|| {
            quote! {
                fn update_as_field(
                    &mut self,
                    source: Self,
                    _mask: &Self::Mask,
                    _options: &::fieldmask::UpdateOptions,
                ) {
                    *self = source;
                }
            }
        });
    let merge_fn = merge_fn
        .map(|item| item.to_token_stream())
        .unwrap_or_else(|| {
            quote! {
                fn merge(
                    &mut self,
                    source: Self,
                    _options: &::fieldmask::UpdateOptions,
                ) {
                    *self = source;
                }
            }
        });

    quote! {
        impl #impl_generics ::fieldmask::Maskable for #ty
        #where_clauses
        {
            type Mask = ();

            fn full_mask() -> Self::Mask {}

            fn make_mask_include_field<'a>(
                _mask: &mut Self::Mask,
                field_path: &[&'a ::core::primitive::str],
            ) -> ::core::result::Result<(), ::fieldmask::DeserializeMaskError<'a>> {
                if field_path.is_empty() {
                    return ::core::result::Result::Ok(());
                }
                ::core::result::Result::Err(::fieldmask::DeserializeMaskError::FieldNotFound {
                    type_name: ::core::stringify!(#ty),
                    field: field_path[0],
                })
            }
        }

        impl #impl_generics ::fieldmask::SelfMaskable for #ty
        #where_clauses
        {
            fn project(self, _mask: &Self::Mask) -> Self {
                return self;
            }

            #update_as_field_fn
            #merge_fn
        }
    }
    .into()
}
