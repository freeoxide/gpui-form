//! Generator for the per-form state facade (`<Name>FormState`) and the
//! per-form patch type (`<Name>FormPatch`).
//!
//! Milestone 1 of the form feature-parity plan (field-level state model):
//! the facade *composes* the standalone core primitives —
//! `::gpui_form::core::FormState<H>` for baseline/current values and
//! `::gpui_form::core::TouchedFields` for engagement tracking — behind typed
//! `<Name>FormPath` accessors, so `FormState<H>` itself stays a frozen
//! two-slot generic and every primitive remains usable standalone.
//!
//! # Rules
//!
//! - Holders stay plain data: the facade and patch are separate generated
//!   types, never fields or methods-with-state on the value holder.
//! - Both types carry the source struct's generics verbatim. `<Name>FormPath`
//!   can be generics-free because a path names no types; the facade names the
//!   holder (`FormState<<Name>FormValueHolder<..>>`), so dropping generics
//!   would break source structs that compile today.
//! - Skipped fields are absent from the patch and from every facade accessor.
//! - Per-field dirty comparison reuses the holder's unconditional `PartialEq`
//!   field types (comparison happens on holder types, not source types), so
//!   `type`/`from`/`into` conversion fields need no special handling here.
//! - The patch's serde derives are added only when the derive crate is built
//!   with its `serde` feature (same gate as the holder), with
//!   `skip_serializing_if = "Option::is_none"` per field so the wire format
//!   is the PATCH tri-state: absent = unchanged, `null` = cleared,
//!   value = set.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::DeriveInput;

use crate::derives::gpui_form::structs::FieldOptionality;
use crate::derives::gpui_form::value_holder::form_field_type_tokens;

/// Generates `<Name>FormState` and `<Name>FormPatch` for a single form.
///
/// Spliced into BOTH expansion branches (empty-form early return and main
/// expansion) next to the value-holder and path tokens. Empty forms emit a
/// facade over an empty holder and a field-less patch.
pub fn generate_form_state(
    original_input: &DeriveInput,
    fields: &[FieldOptionality],
) -> TokenStream {
    let original_ident = &original_input.ident;
    let holder_ident = format_ident!("{}FormValueHolder", original_ident);
    let path_ident = format_ident!("{}FormPath", original_ident);
    let state_ident = format_ident!("{}FormState", original_ident);
    let patch_ident = format_ident!("{}FormPatch", original_ident);

    let generics = &original_input.generics;
    let (impl_generics, ty_generics, where_clause) = original_input.generics.split_for_impl();

    let non_skipped: Vec<&FieldOptionality> = fields.iter().filter(|f| !f.skip).collect();

    let is_field_dirty_arms: Vec<TokenStream> = non_skipped
        .iter()
        .map(|f| {
            let field_name = &f.field_name;
            let field_name_str = field_name.to_string();
            quote! {
                [#field_name_str] => {
                    self.state.current().#field_name != self.state.baseline().#field_name
                },
            }
        })
        .collect();
    // Empty forms have no valid paths at all, so an unknown path there is
    // expected rather than a caller bug — no debug assert.
    let is_field_dirty_unknown_arm = if is_field_dirty_arms.is_empty() {
        quote! { _ => false, }
    } else {
        quote! {
            _ => {
                ::core::debug_assert!(
                    false,
                    "is_field_dirty: path is not a field of this form"
                );
                false
            },
        }
    };

    let mut patch_derives = vec![
        quote! { Clone },
        quote! { Debug },
        quote! { Default },
        quote! { ::core::cmp::PartialEq },
    ];
    let patch_field_serde_attr: TokenStream = if cfg!(feature = "serde") {
        patch_derives.push(quote! { ::serde::Serialize });
        patch_derives.push(quote! { ::serde::Deserialize });
        quote! { #[serde(skip_serializing_if = "Option::is_none", default)] }
    } else {
        quote! {}
    };

    let patch_between_fields: Vec<TokenStream> = non_skipped
        .iter()
        .map(|f| {
            let field_name = &f.field_name;
            quote! {
                #field_name: (from.#field_name != to.#field_name)
                    .then(|| to.#field_name.clone())
            }
        })
        .collect();

    let patch_is_empty_checks: Vec<TokenStream> = non_skipped
        .iter()
        .map(|f| {
            let field_name = &f.field_name;
            quote! { self.#field_name.is_none() }
        })
        .collect();
    let patch_is_empty_body = if patch_is_empty_checks.is_empty() {
        quote! { true }
    } else {
        quote! { #(#patch_is_empty_checks)&&* }
    };

    let patch_changed_fields_pushes: Vec<TokenStream> = non_skipped
        .iter()
        .map(|f| {
            let field_name = &f.field_name;
            quote! {
                if self.#field_name.is_some() {
                    changed.push(#path_ident::#field_name());
                }
            }
        })
        .collect();

    let patch_apply_stmts: Vec<TokenStream> = non_skipped
        .iter()
        .map(|f| {
            let field_name = &f.field_name;
            quote! {
                if let Some(value) = self.#field_name {
                    holder.#field_name = value;
                }
            }
        })
        .collect();

    let patch_struct_fields: Vec<TokenStream> = non_skipped
        .iter()
        .map(|f| {
            let field_name = &f.field_name;
            let field_type = form_field_type_tokens(f);
            let serde_attr = &patch_field_serde_attr;
            quote! {
                #serde_attr pub #field_name: Option<#field_type>
            }
        })
        .collect();

    quote! {
        /// Per-form patch: changed holder-typed field values between two
        /// holders, in declaration order.
        ///
        /// Outer `Option` per field = changed; the inner value is the
        /// holder-typed value, so wrapped or originally-optional fields read
        /// as `Option<Option<T>>` (`Some(None)` = changed to cleared).
        /// Skipped fields are absent.
        #[allow(clippy::derive_partial_eq_without_eq)]
        #[derive(#(#patch_derives),*)]
        pub struct #patch_ident #generics {
            #(#patch_struct_fields),*
        }

        impl #impl_generics #patch_ident #ty_generics #where_clause {
            /// Diff two holders field-by-field.
            pub fn between(from: &#holder_ident #ty_generics, to: &#holder_ident #ty_generics) -> Self {
                Self {
                    #(#patch_between_fields),*
                }
            }

            /// Whether no field changed.
            pub fn is_empty(&self) -> bool {
                #patch_is_empty_body
            }

            /// Paths of the changed fields in declaration order.
            pub fn changed_fields(&self) -> Vec<#path_ident> {
                let mut changed = Vec::new();
                #(#patch_changed_fields_pushes)*
                changed
            }

            /// Apply the changed fields onto `holder`; unchanged (`None`)
            /// fields are left untouched.
            pub fn apply_to(self, holder: &mut #holder_ident #ty_generics) {
                #(#patch_apply_stmts)*
            }
        }

        /// Per-form state facade composing the standalone core primitives:
        /// baseline/current values via `::gpui_form::core::FormState` and
        /// touched-field tracking via `::gpui_form::core::TouchedFields`,
        /// both keyed by typed `<Name>FormPath` values.
        pub struct #state_ident #generics {
            state: ::gpui_form::core::FormState<#holder_ident #ty_generics>,
            touched: ::gpui_form::core::TouchedFields,
        }

        impl #impl_generics #state_ident #ty_generics #where_clause {
            /// Create a facade whose baseline and current both start as
            /// `holder` and whose touched set starts empty.
            pub fn new(holder: #holder_ident #ty_generics) -> Self {
                Self {
                    state: ::gpui_form::core::FormState::new(holder),
                    touched: ::gpui_form::core::TouchedFields::new(#path_ident::FIELD_COUNT),
                }
            }

            /// Borrow the wrapped core state.
            pub fn state(&self) -> &::gpui_form::core::FormState<#holder_ident #ty_generics> {
                &self.state
            }

            /// Mutably borrow the wrapped core state. Mutating through this
            /// escape hatch does not touch the touched set.
            pub fn state_mut(
                &mut self,
            ) -> &mut ::gpui_form::core::FormState<#holder_ident #ty_generics> {
                &mut self.state
            }

            /// Borrow the live (`current`) holder value.
            pub fn current(&self) -> &#holder_ident #ty_generics {
                self.state.current()
            }

            /// Mutably borrow the live holder value. Touched tracking is
            /// event-driven and separate: mutate here, then call
            /// [`Self::touch`] from the component event handler.
            pub fn current_mut(&mut self) -> &mut #holder_ident #ty_generics {
                self.state.current_mut()
            }

            /// Borrow the saved (`baseline`) holder value.
            pub fn baseline(&self) -> &#holder_ident #ty_generics {
                self.state.baseline()
            }

            /// Overwrite `current` with `holder`; the baseline and the
            /// touched set are untouched.
            pub fn replace_current(&mut self, holder: #holder_ident #ty_generics) {
                self.state.replace_current(holder);
            }

            /// Consume the facade and return the live holder value.
            pub fn into_current(self) -> #holder_ident #ty_generics {
                self.state.into_current()
            }

            /// Reset `current` back to the baseline, discard edits, and clear
            /// the touched set — a fresh canvas.
            pub fn reset_to_baseline(&mut self) {
                self.state.reset_to_baseline();
                self.touched.clear_all();
            }

            /// Reset `current` back to the baseline but keep the touched set
            /// (undo-style revert: fields stay visited).
            pub fn reset_to_baseline_keeping_touched(&mut self) {
                self.state.reset_to_baseline();
            }

            /// Snapshot `current` into `baseline` (call after a successful
            /// save). The touched set is untouched: a save does not un-visit
            /// fields.
            pub fn sync_baseline(&mut self) {
                self.state.sync_baseline();
            }

            /// Whether the live holder differs from the baseline
            /// (whole-value comparison; delegates to the core state).
            pub fn is_dirty(&self) -> bool {
                self.state.is_dirty()
            }

            /// Whether one specific field differs between baseline and
            /// current. Unknown paths return `false` (and assert in debug
            /// builds).
            pub fn is_field_dirty(&self, field: &#path_ident) -> bool {
                match field.path().segments() {
                    #(#is_field_dirty_arms)*
                    #is_field_dirty_unknown_arm
                }
            }

            /// Paths of the fields that differ between baseline and current,
            /// in declaration order.
            pub fn changed_fields(&self) -> Vec<#path_ident> {
                self.patch().changed_fields()
            }

            /// The field-level diff of baseline to current.
            pub fn patch(&self) -> #patch_ident #ty_generics {
                #patch_ident::between(self.state.baseline(), self.state.current())
            }

            /// Mark a field touched (from a blur or change event). Unknown
            /// paths are a no-op.
            pub fn touch(&mut self, field: &#path_ident) {
                if let Some(index) = field.field_index() {
                    self.touched.touch_index(index);
                }
            }

            /// Whether a field is touched. Unknown paths are `false`.
            pub fn is_touched(&self, field: &#path_ident) -> bool {
                field
                    .field_index()
                    .is_some_and(|index| self.touched.is_touched_index(index))
            }

            /// Clear the touched flag for one field. Unknown paths are a
            /// no-op.
            pub fn clear_touched(&mut self, field: &#path_ident) {
                if let Some(index) = field.field_index() {
                    self.touched.clear_index(index);
                }
            }

            /// Clear the whole touched set.
            pub fn clear_all_touched(&mut self) {
                self.touched.clear_all();
            }

            /// Touched field paths in declaration order.
            pub fn touched_fields(&self) -> Vec<#path_ident> {
                self.touched
                    .touched_indices()
                    .filter_map(#path_ident::path_for_index)
                    .collect()
            }

            /// Whether any field is touched.
            pub fn is_any_touched(&self) -> bool {
                self.touched.is_any_touched()
            }
        }
    }
}
