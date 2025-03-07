use inflector::cases::snakecase::to_snake_case;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Index, parse_macro_input};
use utils::{Item, ItemInfo, ItemType};

mod utils;

// We cannot split the implementation of the `(Self/Option)Maskable` traits into multiple functions
// because `Self/OptionMaskable`'s implementation depends on `Maskable`'s implementation.
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

    let full_mask_arms = fields.iter().map(|field| {
        if field.is_flatten {
            quote! {
                ::fieldmask::Mask::full(),
            }
        } else {
            quote! {
                Some(::fieldmask::Mask::full()),
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
                        .get_or_insert_default()
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

    let additional_impl = match item_type {
        ItemType::UnitEnum => {
            // Unit enums have no fields, the field mask is always empty.
            quote! {
                impl ::fieldmask::OptionMaskable for #ident {
                    fn option_project(this: Option<Self>, _mask: &Self::Mask) -> Option<Self> {
                        this
                    }

                    fn option_update(this: &mut Option<Self>, source: Option<Self>, mask: &Self::Mask, options: &::fieldmask::UpdateOptions) {
                        Self::option_update_as_field(this, source, mask, options);
                    }

                    fn option_update_as_field(this: &mut Option<Self>, source: Option<Self>, _mask: &Self::Mask, _options: &::fieldmask::UpdateOptions) {
                        *this = source;
                    }

                    fn option_merge(this: &mut Option<Self>, source: Option<Self>, _options: &::fieldmask::UpdateOptions) {
                        if source.is_some() {
                            *this = source;
                        }
                    }
                }
            }
        }
        ItemType::Enum => {
            let project_match_arms = fields.iter().enumerate().map(|(i, field)| {
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

            let update_arms = fields.iter().enumerate().map(|(i, field)| {
                let index = Index::from(i);
                let ident = field.ident;
                quote! {
                    Self::#ident(source_inner) => {
                        if let Some(mask) = &mask.#index {
                            if let Self::#ident(self_inner) = self {
                                self_inner.update_as_field(source_inner, mask, options);
                            } else {
                                *self = Self::#ident(source_inner);
                            }
                        }
                    }
                }
            });

            let merge_arms = fields.iter().map(|field| {
                let ident = field.ident;

                quote! {
                    Self::#ident(source_inner) => {
                        if let Self::#ident(self_inner) = self {
                            self_inner.merge(source_inner, options);
                        } else {
                            *self = Self::#ident(source_inner);
                        }
                    }
                }
            });

            quote! {
                impl #impl_generics ::fieldmask::SelfMaskable for #ident #ty_generics
                #where_clauses
                {
                    fn project(self, mask: &Self::Mask) -> Self {
                        match self {
                            #(#project_match_arms)*
                        }
                    }

                    fn update(&mut self, source: Self, mask: &Self::Mask, options: &::fieldmask::UpdateOptions) {
                        if mask == &Self::Mask::default() {
                            self.update_as_field(source, &Self::full_mask(), options);
                            return;
                        }

                        self.update_as_field(source, mask, options);
                    }

                    fn update_as_field(&mut self, source: Self, mask: &Self::Mask, options: &::fieldmask::UpdateOptions) {
                        if mask == &Self::Mask::default() && options.replace_message {
                            *self = source;
                            return;
                        }

                        match source {
                            #(#update_arms)*
                        }
                    }

                    fn merge(&mut self, source: Self, options: &::fieldmask::UpdateOptions) {
                        if options.replace_message {
                            *self = source;
                            return;
                        }

                        match source {
                            #(#merge_arms)*
                        }
                    }
                }
            }
        }
        ItemType::Struct => {
            // For each field in the struct, generate a field arm that performs projection on the field.
            let project_arms = fields.iter().enumerate().map(|(i, field)| {
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

            // For each field in the struct, generate a field arm that performs update on the field.
            let update_arms = fields.iter().enumerate().map(|(i, field)| {
                let index = Index::from(i);
                let ident = field.ident;

                if field.is_flatten {
                    quote! {
                        self.#ident.update(source.#ident, &mask.#index, options);
                    }
                } else {
                    quote! {
                        if let Some(mask) = &mask.#index {
                            self.#ident.update(source.#ident, mask, options);
                        }
                    }
                }
            });

            let merge_arms = fields.iter().map(|field| {
                let ident = field.ident;

                quote! {
                    self.#ident.merge(source.#ident, options);
                }
            });

            quote! {
                impl #impl_generics ::fieldmask::SelfMaskable for #ident #ty_generics
                #where_clauses
                {
                    fn project(self, mask: &Self::Mask) -> Self {
                        if mask == &Self::Mask::default() {
                            return self;
                        }

                        let Self { #(#field_idents),* } = self;
                        Self {
                            #(#project_arms)*
                        }
                    }

                    fn update(&mut self, source: Self, mask: &Self::Mask, options: &::fieldmask::UpdateOptions) {
                        if mask == &Self::Mask::default() {
                            self.update_as_field(source, &Self::full_mask(), options);
                            return;
                        }

                        self.update_as_field(source, mask, options);
                    }

                    fn update_as_field(&mut self, source: Self, mask: &Self::Mask, options: &::fieldmask::UpdateOptions) {
                        if mask == &Self::Mask::default() && options.replace_message {
                            *self = source;
                            return;
                        }

                        #(#update_arms)*
                    }

                    fn merge(&mut self, source: Self, options: &::fieldmask::UpdateOptions) {
                        if options.replace_message {
                            *self = source;
                            return;
                        }

                        #(#merge_arms)*
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

            fn full_mask() -> Self::Mask {
                (#(#full_mask_arms)*)
            }

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
        }

        #additional_impl
    }
    .into()
}
