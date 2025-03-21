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
/// - A struct that implements `Default`, `PartialEq` and `SelfMaskable`.
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

/// Treat the type as an atomic value and Implement `Maskable`, `OptionMaskable`, `SelfMaskable`
/// for the type.
///
/// You can override the default implementation of `update_as_field` and `merge` if needed.
///
/// ### Example:
/// ```ignore
/// maskable_atomic!(impl bool {});
///
/// maskable_atomic!(
///     impl<T> Vec<T> {
///         // Omit this if you don't want to override the default implementation.
///         fn update_as_field(&mut self, source: Self, _mask: &Self::Mask, options: &UpdateOptions) {
///             self.merge(source, options);
///         }
///
///         // Omit this if you don't want to override the default implementation.
///         fn merge(&mut self, source: Self, options: &UpdateOptions) {
///             if options.replace_repeated {
///                 *self = source;
///                 return;
///             }
///
///             self.extend(source);
///         }
///     }
/// );
/// ```
#[proc_macro]
pub fn maskable_atomic(input: TokenStream) -> TokenStream {
    maskable_atomic_impl(input)
}
