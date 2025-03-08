use inflector::cases::snakecase::to_snake_case;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Index, parse_macro_input};
use utils::{Message, MessageInfo, MessageType};

mod utils;

/// Derive `Maskable` for the type.
///
/// The type must be one of the following types:
/// - A unit-like enum.
///   - `OptionMaskable` is also derived on this type.
/// - An enum where each variant has exactly one unnamed associated field. The associated field must
///   implement `SelfMaskable` and `Default`.
///   - `OptionMaskable` is also derived on this type.
/// - A struct with named fields, where the type of each field must implement `SelfMaskable` and
///   `Default`.
///   - `SelfMaskable` is also derived on this type.
// We cannot split the implementation of the `(Self/Option)Maskable` traits into multiple functions
// because `Self/OptionMaskable`'s implementation depends on `Maskable`'s implementation.
#[proc_macro_derive(Maskable, attributes(fieldmask))]
pub fn derive_maskable(input: TokenStream) -> TokenStream {
    let input: Message = parse_macro_input!(input);
    let MessageInfo {
        message_type,
        ident,
        generics,
        fields,
    } = input.get_message_info();

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
                    ::core::result::Result::Ok(())
                }
            }
        }
    });

    let additional_impl = match message_type {
        MessageType::UnitEnum => {
            // Unit enums have no fields, the field mask is always empty.
            quote! {
                impl ::fieldmask::OptionMaskable for #ident {
                    fn option_project(
                        this: ::core::option::Option<Self>,
                        _mask: &Self::Mask,
                    ) -> ::core::option::Option<Self> {
                        this
                    }

                    fn option_update(
                        this: &mut ::core::option::Option<Self>,
                        source: ::core::option::Option<Self>,
                        mask: &Self::Mask,
                        options: &::fieldmask::UpdateOptions,
                    ) {
                        Self::option_update_as_field(this, source, mask, options);
                    }

                    fn option_update_as_field(
                        this: &mut ::core::option::Option<Self>,
                        source: ::core::option::Option<Self>,
                        _mask: &Self::Mask,
                        _options: &::fieldmask::UpdateOptions,
                    ) {
                        *this = source;
                    }

                    fn option_merge(
                        this: &mut ::core::option::Option<Self>,
                        source: ::core::option::Option<Self>,
                        _options: &::fieldmask::UpdateOptions,
                    ) {
                        if source.is_some() {
                            *this = source;
                        }
                    }
                }
            }
        }
        MessageType::TupleEnum => {
            let project_match_arms = fields.iter().enumerate().map(|(i, field)| {
                let index = Index::from(i);
                let ident = field.ident;
                // If the variant is not selected by the mask, return None.
                quote! {
                    Self::#ident(inner) => mask.#index
                        .as_ref()
                        .map(|mask| Self::#ident(inner.project(mask))),
                }
            });

            let update_source_arms = fields.iter().enumerate().map(|(i, field)| {
                let index = Index::from(i);
                let ident = field.ident;
                quote! {
                    Self::#ident(source_inner) => {
                        if let ::core::option::Option::Some(mask) = &mask.#index {
                            if let ::core::option::Option::Some(Self::#ident(this_inner)) = this {
                                this_inner.update_as_field(source_inner, mask, options);
                            } else {
                                *this = ::core::option::Option::Some(Self::#ident(source_inner.project(mask)));
                            }
                            return;
                        }
                    }
                }
            });

            let update_this_arms = fields.iter().enumerate().map(|(i, field)| {
                let index = Index::from(i);
                let ident = field.ident;
                quote! {
                    Self::#ident(this_inner) => {
                        if let ::core::option::Option::Some(mask) = &mask.#index {
                            this_inner.update_as_field(::core::default::Default::default(), mask, options);
                        }
                    }
                }
            });

            let merge_arms = fields.iter().map(|field| {
                let ident = field.ident;

                quote! {
                    Self::#ident(source_inner) => {
                        if let ::core::option::Option::Some(Self::#ident(this_inner)) = this {
                            this_inner.merge(source_inner, options);
                        } else {
                            *this = ::core::option::Option::Some(Self::#ident(source_inner));
                        }
                    }
                }
            });

            quote! {
                impl #impl_generics ::fieldmask::OptionMaskable for #ident #ty_generics
                #where_clauses
                {
                    fn option_project(
                        this: ::core::option::Option<Self>,
                        mask: &Self::Mask,
                    ) -> Option<Self> {
                        match this {
                            ::core::option::Option::Some(inner) => {
                                match inner {
                                    #(#project_match_arms)*
                                }
                            }
                            ::core::option::Option::None => None,
                        }
                    }

                    fn option_update(
                        this: &mut ::core::option::Option<Self>,
                        source: ::core::option::Option<Self>,
                        mask: &Self::Mask,
                        options: &::fieldmask::UpdateOptions,
                    ) {
                        if mask == &Self::Mask::default() {
                            Self::option_update_as_field(this, source, &Self::full_mask(), options);
                            return;
                        }

                        Self::option_update_as_field(this, source, mask, options);
                    }

                    fn option_update_as_field(
                        this: &mut ::core::option::Option<Self>,
                        source: ::core::option::Option<Self>,
                        mask: &Self::Mask,
                        options: &::fieldmask::UpdateOptions,
                    ) {
                        if mask == &Self::Mask::default() {
                            Self::option_merge(this, source, options);
                            return;
                        }

                        if let ::core::option::Option::Some(source_inner) = source {
                            match source_inner {
                                #(#update_source_arms)*
                            }
                        }

                        if let ::core::option::Option::Some(this_inner) = this {
                            match this_inner {
                                #(#update_this_arms)*
                            }
                        }
                    }

                    fn option_merge(
                        this: &mut ::core::option::Option<Self>,
                        source: ::core::option::Option<Self>,
                        options: &::fieldmask::UpdateOptions,
                    ) {
                        if options.replace_message {
                            *this = source;
                            return;
                        }

                        if let ::core::option::Option::Some(source_inner) = source {
                            match source_inner {
                                #(#merge_arms)*
                            }
                        }
                    }
                }
            }
        }
        MessageType::Struct => {
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
                        self.#ident.update_as_field(source.#ident, &mask.#index, options);
                    }
                } else {
                    quote! {
                        if let Some(mask) = &mask.#index {
                            self.#ident.update_as_field(source.#ident, mask, options);
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
                        if mask == &Self::Mask::default() {
                            self.merge(source, options);
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
