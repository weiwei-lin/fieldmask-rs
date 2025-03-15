use inflector::cases::snakecase::to_snake_case;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Index, parse_macro_input};

use super::ast::{Input, InputType, MessageInfo};

/// The implementation for `derive_maskable`.
pub fn derive_maskable_impl(input: TokenStream) -> TokenStream {
    let input: Input = parse_macro_input!(input);
    let MessageInfo {
        ident,
        generics,
        fields,
        ..
    } = input.get_message_info();

    let (impl_generics, ty_generics, where_clauses) = generics.split_for_impl();

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
                ::core::option::Option::Some(::fieldmask::Mask::full()),
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

    quote! {
        impl #impl_generics ::fieldmask::Maskable for #ident #ty_generics
        #where_clauses
        {
            type Mask = (#(#mask_type_arms)*);

            #[allow(clippy::unused_unit)]
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
                    [field, ..] => ::core::result::Result::Err(
                        ::fieldmask::DeserializeMaskError::FieldNotFound {
                            type_name: ::core::stringify!(#ident),
                            field,
                        }
                    ),
                }
            }
        }
    }
    .into()
}

/// The implementation for `derive_option_maskable`.
pub fn derive_option_maskable_impl(input: TokenStream) -> TokenStream {
    let input: Input = parse_macro_input!(input);
    let MessageInfo {
        message_type,
        ident,
        generics,
        fields,
    } = input.get_message_info();
    let (impl_generics, ty_generics, where_clauses) = generics.split_for_impl();

    match message_type {
        InputType::UnitEnum => {
            // Unit enums have no fields, the field mask is always empty.
            quote! {
                impl ::fieldmask::OptionMaskable for #ident {
                    fn option_project(
                        this: ::core::option::Option<Self>,
                        _mask: &<Self as ::fieldmask::Maskable>::Mask,
                    ) -> ::core::option::Option<Self> {
                        this
                    }

                    fn option_update_as_field(
                        this: &mut ::core::option::Option<Self>,
                        source: ::core::option::Option<Self>,
                        _mask: &<Self as ::fieldmask::Maskable>::Mask,
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
        InputType::TupleEnum => {
            let project_match_arms = fields.iter().enumerate().map(|(i, field)| {
                let index = Index::from(i);
                let ident = field.ident;
                // If the variant is not selected by the mask, return None.
                quote! {
                    Self::#ident(this) => mask.#index
                        .as_ref()
                        .map(|mask| Self::#ident(::fieldmask::SelfMaskable::project(this, mask))),
                }
            });

            let update_source_arms = fields.iter().enumerate().map(|(i, field)| {
                let index = Index::from(i);
                let ident = field.ident;
                quote! {
                    Self::#ident(source) => {
                        if let ::core::option::Option::Some(mask) = &mask.#index {
                            if let ::core::option::Option::Some(Self::#ident(this)) = this {
                                ::fieldmask::SelfMaskable::update_as_field(this, source, mask, options);
                            } else {
                                *this = ::core::option::Option::Some(
                                    Self::#ident(
                                        ::fieldmask::SelfMaskable::project(source, mask)
                                    )
                                );
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
                    Self::#ident(this) => {
                        if let ::core::option::Option::Some(mask) = &mask.#index {
                            ::fieldmask::SelfMaskable::update_as_field(
                                this,
                                ::core::default::Default::default(),
                                mask,
                                options,
                            );
                        }
                    }
                }
            });

            let merge_arms = fields.iter().map(|field| {
                let ident = field.ident;

                quote! {
                    Self::#ident(source) => {
                        if let ::core::option::Option::Some(Self::#ident(this)) = this {
                            ::fieldmask::SelfMaskable::merge(this, source, options);
                        } else {
                            *this = ::core::option::Option::Some(Self::#ident(source));
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
                        mask: &<Self as ::fieldmask::Maskable>::Mask,
                    ) -> ::core::option::Option<Self> {
                        match this {
                            ::core::option::Option::Some(this) => {
                                match this {
                                    #(#project_match_arms)*
                                }
                            }
                            ::core::option::Option::None => ::core::option::Option::None,
                        }
                    }

                    fn option_update_as_field(
                        this: &mut ::core::option::Option<Self>,
                        source: ::core::option::Option<Self>,
                        mask: &<Self as ::fieldmask::Maskable>::Mask,
                        options: &::fieldmask::UpdateOptions,
                    ) {
                        if mask == &<Self as ::fieldmask::Maskable>::Mask::default() {
                            <Self as ::fieldmask::OptionMaskable>::option_merge(this, source, options);
                            return;
                        }

                        if let ::core::option::Option::Some(source) = source {
                            match source {
                                #(#update_source_arms)*
                            }
                        }

                        if let ::core::option::Option::Some(this) = this {
                            match this {
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

                        if let ::core::option::Option::Some(source) = source {
                            match source {
                                #(#merge_arms)*
                            }
                        }
                    }
                }
            }
        }
        InputType::Struct => {
            quote!{
                impl #impl_generics ::fieldmask::OptionMaskable for #ident #ty_generics
                #where_clauses
                {
                    fn option_project(
                        this: ::core::option::Option<Self>,
                        mask: &<Self as ::fieldmask::Maskable>::Mask,
                    ) -> ::core::option::Option<Self> {
                        this.map(|this| ::fieldmask::SelfMaskable::project(this, mask))
                    }

                    fn option_update_as_field(
                        this: &mut ::core::option::Option<Self>,
                        source: ::core::option::Option<Self>,
                        mask: &<Self as ::fieldmask::Maskable>::Mask,
                        options: &::fieldmask::UpdateOptions,
                    ) {
                        match (this.as_mut(), source) {
                            (::core::option::Option::Some(this), ::core::option::Option::Some(source)) => {
                                ::fieldmask::SelfMaskable::update_as_field(this, source, mask, options);
                            }
                            (::core::option::Option::Some(this), ::core::option::Option::None) => {
                                ::fieldmask::SelfMaskable::update_as_field(
                                    this,
                                    ::core::default::Default::default(),
                                    mask,
                                    options,
                                );
                            }
                            (::core::option::Option::None, source) => {
                                *this = source.map(|s| ::fieldmask::SelfMaskable::project(s, mask));
                            }
                        }
                    }

                    fn option_merge(
                        this: &mut ::core::option::Option<Self>,
                        source: ::core::option::Option<Self>,
                        options: &::fieldmask::UpdateOptions,
                    ) {
                        match (this.as_mut(), source) {
                            (::core::option::Option::Some(this), ::core::option::Option::Some(source)) => {
                                ::fieldmask::SelfMaskable::merge(this, source, options);
                            }
                            (_, ::core::option::Option::None) => {}
                            (::core::option::Option::None, source) => {
                                *this = source;
                            }
                        }
                    }
                }
            }
        }
    }.into()
}

/// The implementation for `derive_self_maskable`.
pub fn derive_self_maskable_impl(input: TokenStream) -> TokenStream {
    let input: Input = parse_macro_input!(input);
    let MessageInfo {
        message_type,
        ident,
        generics,
        fields,
    } = input.get_message_info();

    let (impl_generics, ty_generics, where_clauses) = generics.split_for_impl();

    match message_type {
        InputType::UnitEnum => {
            // Unit enums have no fields, the field mask is always empty.
            quote! {
                impl ::fieldmask::SelfMaskable for #ident {
                    fn project(self, as_derefmask: &<Self as ::fieldmask::Maskable>::Mask) -> Self {
                        self
                    }

                    fn update_as_field(
                        &mut self,
                        source: Self,
                        _mask: &<Self as ::fieldmask::Maskable>::Mask,
                        _options: &::fieldmask::UpdateOptions,
                    ) {
                        *self = source;
                    }

                    fn merge(&mut self, source: Self, _options: &::fieldmask::UpdateOptions) {
                        if source != ::core::default::Default::default() {
                            *self = source;
                        }
                    }
                }
            }
        }
        InputType::TupleEnum => {
            panic!(
                "Cannot derive `SelfMaskable` for a tuple enum. You can derive `OptionMaskable` instead."
            );
        }
        InputType::Struct => {
            let field_idents = fields.iter().map(|field| &field.ident).collect::<Vec<_>>();

            // For each field in the struct, generate a field arm that performs projection on the field.
            let project_arms = fields.iter().enumerate().map(|(i, field)| {
                let index = Index::from(i);
                let ident = field.ident;

                if field.is_flatten {
                    quote! {
                        #ident: ::fieldmask::SelfMaskable::project(#ident, &mask.#index),
                    }
                } else {
                    quote! {
                        #ident: mask
                            .#index
                            .as_deref()
                            .map(|mask| ::fieldmask::SelfMaskable::project(#ident, mask))
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
                        ::fieldmask::SelfMaskable::update_as_field(
                            &mut self.#ident,
                            source.#ident,
                            &mask.#index,
                            options,
                        );
                    }
                } else {
                    quote! {
                        if let Some(mask) = &mask.#index {
                            ::fieldmask::SelfMaskable::update_as_field(&mut self.#ident, source.#ident, mask, options);
                        }
                    }
                }
            });

            let merge_arms = fields.iter().map(|field| {
                let ident = field.ident;

                quote! {
                    ::fieldmask::SelfMaskable::merge(&mut self.#ident, source.#ident, options);
                }
            });

            quote! {
                impl #impl_generics ::fieldmask::SelfMaskable for #ident #ty_generics
                #where_clauses
                {
                    fn project(self, mask: &<Self as ::fieldmask::Maskable>::Mask) -> Self {
                        if mask == &<Self as ::fieldmask::Maskable>::Mask::default() {
                            return self;
                        }

                        let Self { #(#field_idents),* } = self;
                        Self {
                            #(#project_arms)*
                        }
                    }

                    fn update_as_field(
                        &mut self,
                        source: Self,
                        mask: &<Self as ::fieldmask::Maskable>::Mask,
                        options: &::fieldmask::UpdateOptions,
                    ) {
                        if mask == &<Self as ::fieldmask::Maskable>::Mask::default() {
                            ::fieldmask::SelfMaskable::merge(self, source, options);
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
    }.into()
}
