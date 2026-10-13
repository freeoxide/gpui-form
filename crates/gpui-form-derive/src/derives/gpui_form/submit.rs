//! Generator for the opt-in submit-lifecycle alias (backlog #7, [D7]).
//!
//! `#[gpui_form(submit)]` emits exactly one item:
//!
//! ```ignore
//! pub type <Name>FormSubmission = ::gpui_form::core::submit::SubmitTracker<E>;
//! ```
//!
//! `E` defaults to the holder's koruma error type
//! (`<Name>FormValueHolder as ::koruma::ValidateExt>::Error`) when the holder
//! derives `Koruma` — the natural failure type for a
//! validate-then-convert submit — and falls back to the core
//! `::gpui_form::core::submit::BoxError` alias otherwise.
//!
//! # Why an alias and not generated state
//!
//! The backlog is explicit that submit lifecycle is "optional runtime glue,
//! not required generated state. The holder should remain plain data."
//! `SubmitTracker<E>` lives in `gpui-form-core`; the alias is the only
//! per-form surface, so forms that do not opt in see zero emitted diff.
//! GPUI button wiring stays app- or prototyping-authored ([D5]).
//!
//! Note the trait used for the koruma error type: `ValidateExt` owns the
//! associated `Error` type (`Validate<T>` is the validator trait and has no
//! associated types).

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::DeriveInput;

/// Emits the `<Name>FormSubmission` alias when `emit` is set.
///
/// `koruma_enabled` must be the same gate that puts `#[derive(Koruma)]` on
/// the holder (`needs_koruma_derive`), so the referenced `ValidateExt::Error`
/// is guaranteed to exist. Generic source structs fall back to `BoxError`:
/// koruma's generated error types carry a usage-analysis *subset* of the
/// source generics, which cannot be reconstructed here (the same exclusion
/// the typed error surface applies).
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

    // Generated-item docs stay one-line summaries (the derive output is
    // ~45% #[doc] payload by bytes otherwise); the removed essay lives in
    // this generator comment: the alias is `::gpui_form::core::submit::
    // SubmitTracker`; the error parameter is this form's koruma validation
    // error type, so a failed validate-then-submit carries the typed field
    // errors. Driving the submit (executor, button wiring) is application
    // code — the holder stays plain data.
    quote! {
        /// Headless submit-lifecycle tracker for this form, opt-in via `#[gpui_form(submit)]` (backlog #7).
        pub type #alias_ident = ::gpui_form::core::submit::SubmitTracker<#error_type>;
    }
}
