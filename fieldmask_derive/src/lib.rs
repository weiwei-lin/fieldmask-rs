mod derive;
mod func;

use derive::{derive_maskable_impl, derive_option_maskable_impl, derive_self_maskable_impl};
use func::maskable_atomic_impl;
use proc_macro::TokenStream;

/// Derive `Maskable` for the type.
///
/// The type must be one of the following types:
/// - A unit-like enum.
/// - An enum where each variant has exactly one unnamed associated field. The associated field must
///   implement `Maskable`.
/// - A struct with named fields, where the type of each field must implement `Maskable`.
#[proc_macro_derive(Maskable, attributes(fieldmask))]
pub fn derive_maskable(input: TokenStream) -> TokenStream {
    derive_maskable_impl(input)
}

/// Derive `OptionMaskable` for the type.
///
/// The type must be one of the following types:
/// - A unit-like enum.
/// - An enum where each variant has exactly one unnamed associated field. The associated field must
///   implement `SelfMaskable` and `Default`.
#[proc_macro_derive(OptionMaskable, attributes(fieldmask))]
pub fn derive_option_maskable(input: TokenStream) -> TokenStream {
    derive_option_maskable_impl(input)
}

/// Derive `SelfMaskable` for the type.
///
/// The type must be one of the following types:
/// - A unit-like enum that implements `Default` and `PartialEqual`.
/// - A struct with named fields, where the type of each field must implement `Default`,
///   `PartialEqual`, and `SelfMaskable`.
#[proc_macro_derive(SelfMaskable, attributes(fieldmask))]
pub fn derive_self_maskable(input: TokenStream) -> TokenStream {
    derive_self_maskable_impl(input)
}

#[proc_macro]
pub fn maskable_atomic(input: TokenStream) -> TokenStream {
    maskable_atomic_impl(input)
}
