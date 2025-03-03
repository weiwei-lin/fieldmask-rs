use inflector::cases::snakecase::to_snake_case;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Index, parse_macro_input};
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
    let field_idents = fields.iter().map(|field| &field.ident).collect::<Vec<_>>();

    let mask_type_arms = fields.iter().map(|field| {
        let field_ty = field.ty;
        if field.is_flatten {
            quote! {
                ::fieldmask::Mask<#field_ty>,
            }
        } else {
            quote! {
                ::core::option::Option<::fieldmask::Mask<#field_ty>>,
            }
        }
    });

    // For each field in the struct, generate a match arm that processes a matching field path.
    let make_mask_include_field_match_arms = fields.iter().enumerate().map(|(i, field)| {
        let field_index = Index::from(i);
        // For flatten field, try to make the field parse the mask. If the field is not found, go to
        // the next match arm.
        if field.is_flatten {
            quote! {
                _ if mask
                    .#field_index
                    .include_field(field_path)
                    .map(|_| true)
                    .or_else(|e| {
                        if let ::fieldmask::DeserializeMaskError::FieldNotFound { .. } = e {
                            ::core::result::Result::Ok(false)
                        } else {
                            ::core::result::Result::Err(e)
                        }
                    })? =>
                {
                    ::core::result::Result::Ok(())
                }
            }
        } else {
            // Convert to snake case to match the field name in the mask.
            // This is useful for oneof fields where each oneof field is represented by a variant
            // in an enum, which is typically in PascalCase.
            let field_name = to_snake_case(&field.ident.to_string());
            quote! {
                [#field_name, tail @ ..] => {
                    mask.#field_index
                        .get_or_insert_with(::core::default::Default::default)
                        .include_field(tail)
                        .map_err(|err| {
                            ::fieldmask::DeserializeMaskError::InvalidField {
                                field: #field_name,
                                err: ::std::boxed::Box::new(err),
                            }
                        })?;
                    Ok(())
                }
            }
        }
    });

    let project_impl = match item_type {
        ItemType::Enum => {
            let match_arms = fields.iter().enumerate().map(|(i, field)| {
                let index = Index::from(i);
                let ident = field.ident;
                quote! {
                    Self::#ident(inner) => Self::#ident(
                        mask.#index
                            .as_ref()
                            .map(|mask| inner.project(mask))
                            .unwrap_or_default()
                    ),
                }
            });
            quote! {
                fn project(self, mask: &Self::Mask) -> Self {
                    match self {
                        #(#match_arms)*
                    }
                }
            }
        }
        ItemType::UnitEnum => {
            // Unit enums have no fields, the field mask is always empty.
            quote! {
                fn project(self, mask: &Self::Mask) -> Self {
                    self
                }
            }
        }
        ItemType::Struct => {
            // For each field in the struct, generate a field arm that performs projection on the field.
            let field_arms = fields.iter().enumerate().map(|(i, field)| {
                let index = Index::from(i);
                let ident = field.ident;

                if field.is_flatten {
                    quote! {
                        #ident: #ident.project(&mask.#index),
                    }
                } else {
                    quote! {
                        #ident: mask
                            .#index
                            .as_deref()
                            .map(|mask| #ident.project(mask))
                            .unwrap_or_default(),
                    }
                }
            });

            quote! {
                fn project(self, mask: &Self::Mask) -> Self {
                    if mask == &Self::Mask::default() {
                        return self;
                    }

                    let Self { #(#field_idents),* } = self;
                    Self {
                        #(#field_arms)*
                    }
                }
            }
        }
    };

    quote! {
        impl #impl_generics ::fieldmask::Maskable for #ident #ty_generics
        #where_clauses
        {
            type Mask = (#(#mask_type_arms)*);

            fn make_mask_include_field<'a>(
                mask: &mut Self::Mask,
                field_path: &[&'a ::core::primitive::str],
            ) -> ::core::result::Result<(), ::fieldmask::DeserializeMaskError<'a>> {
                match field_path {
                    [] => ::core::result::Result::Ok(()),
                    #(#make_mask_include_field_match_arms)*
                    [field, ..] => ::core::result::Result::Err(::fieldmask::DeserializeMaskError::FieldNotFound {
                        type_name: stringify!(#ident),
                        field,
                    }),
                }
            }

            #project_impl
        }
    }
    .into()
}
