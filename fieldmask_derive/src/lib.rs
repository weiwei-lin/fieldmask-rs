use inflector::cases::snakecase::to_snake_case;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Index};
use utils::{Item, ItemInfo, ItemType};

mod utils;

#[proc_macro_derive(Maskable, attributes(fieldmask))]
pub fn derive_maskable(input: TokenStream) -> TokenStream {
    let input: Item = parse_macro_input!(input);
    let ItemInfo {
        item_type,
        ident,
        generics,
        fields,
    } = input.get_info();

    let (impl_generics, ty_generics, where_clauses) = generics.split_for_impl();
    let field_indices = fields.iter().enumerate().map(|(i, _field)| Index::from(i));
    let field_idents = fields.iter().map(|field| &field.ident).collect::<Vec<_>>();
    let field_types = fields.iter().map(|f| f.ty);
    let match_arms = fields.iter().enumerate().map(|(i, field)| {
        let index = Index::from(i);
        if field.is_flatten {
            quote! {
                _ if mask
                    .0
                    .#index
                    .try_bitor_assign(field_mask_segs)
                    .map(|_| true)
                    .or_else(|l| if l.depth == 0 { Ok(false) } else { Err(l) })? =>
                {
                    ()
                }
            }
        } else {
            let prefix = to_snake_case(&field.ident.to_string());
            quote! {
                [#prefix, tail @ ..] => mask.0.#index.try_bitor_assign(tail).map_err(|mut e| {
                    e.depth += 1;
                    e
                })?,
            }
        }
    });
    let match_arm_groups = fields.iter().map(|target_field| {
        let target_ident = target_field.ident;
        let arms = fields.iter().enumerate().map(|(i, src_field)| {
            let index = Index::from(i);
            let src_ident = src_field.ident;
            if src_ident == target_ident {
                quote! {
                    Self::#src_ident(s) if mask.0.#index != ::fieldmask::FieldMask::default() => {
                        mask.0.#index.apply(t, s);
                    }
                }
            } else {
                let src_ty = src_field.ty;
                quote! {
                    Self::#src_ident(s) if mask.0.#index != ::fieldmask::FieldMask::default() => {
                        let mut new = <#src_ty>::default();
                        mask.0.#index.apply(&mut new, s);
                        *self = Self::#src_ident(new);
                    }
                }
            }
        });
        quote! {
            Self::#target_ident(t) => match src {
                #(#arms)*
                _ => return false,
            }
        }
    });

    let additional_impl = match item_type {
        ItemType::Enum => quote! {
            impl#impl_generics ::fieldmask::OptionMaskable for #ident#ty_generics
            #where_clauses
            {
                fn apply_mask(&mut self, src: Self, mask: Self::Mask) -> bool {
                    match self {
                        #(#match_arm_groups)*
                    }
                    return true;
                }
            }
        },
        ItemType::Struct => quote! {
            impl#impl_generics ::fieldmask::SelfMaskable for #ident#ty_generics
            #where_clauses
            {
                fn apply_mask(&mut self, src: Self, mask: Self::Mask) {
                    #(mask.0.#field_indices.apply(&mut self.#field_idents, src.#field_idents);)*
                }
            }
        },
    };

    (quote! {
        impl#impl_generics ::fieldmask::Maskable for #ident#ty_generics
        #where_clauses
        {
            type Mask = ::fieldmask::BitwiseWrap<(#(::fieldmask::FieldMask<#field_types>,)*)>;

            fn try_bitor_assign_mask(
                mask: &mut Self::Mask,
                field_mask_segs: &[&::core::primitive::str],
            ) -> ::core::result::Result<(), ::fieldmask::DeserializeMaskError> {
                match field_mask_segs {
                    [] => *mask = !Self::Mask::default(),
                    #(#match_arms)*
                    _ => return ::core::result::Result::Err(::fieldmask::DeserializeMaskError{
                        type_str: stringify!(#ident),
                        field: field_mask_segs[0].into(),
                        depth: 0,
                    }),
                }
                Ok(())
            }
        }

        #additional_impl
    })
    .into()
}
