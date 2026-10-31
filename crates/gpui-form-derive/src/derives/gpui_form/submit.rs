use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::DeriveInput;

/// Emits the `<Name>FormSubmission` alias when `emit` is set. `koruma_enabled`
/// must be the holder's `#[derive(Koruma)]` gate so `ValidateExt::Error` exists;
/// generic structs fall back to `BoxError` (koruma's error types cannot be reconstructed).
pub fn generate_submit_alias(
    original_input: &DeriveInput,
    emit: bool,
    koruma_enabled: bool,
) -> TokenStream {
    if !emit {
        return quote! {};
    }

    let alias_ident = format_ident!("{}FormSubmission", original_input.ident);
    let error_type = if koruma_enabled && original_input.generics.params.is_empty() {
        let holder_ident = format_ident!("{}FormValueHolder", original_input.ident);
        quote! { <#holder_ident as ::koruma::ValidateExt>::Error }
    } else {
        quote! { ::gpui_form::core::submit::BoxError }
    };

    quote! {
        /// Headless submit-lifecycle tracker for this form, opt-in via `#[gpui_form(submit)]`.
        pub type #alias_ident = ::gpui_form::core::submit::SubmitTracker<#error_type>;
    }
}
