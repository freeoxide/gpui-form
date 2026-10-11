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
//! Milestone 2 adds the validation-state cache for a live `is_valid`
//! ([D6]): koruma-gated facades gain a `validation` slot of
//! `::gpui_form::core::ValidationCache<Holder, <Name>FormFieldErrors>` plus
//! `validate`/`validation`/`is_valid`/`invalidate_validation`/`error_for`.
//! Freshness is snapshot-compare, so the cache stays sound even under the
//! `state_mut()` escape hatch; reset-style mutators still invalidate
//! explicitly so a reverted-to-same-value form does not keep displaying the
//! old verdict.
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
//! - The validation cache emits only when the holder derives koruma AND the
//!   source struct is non-generic (the same combined gate as the
//!   `<Name>FormFieldErrors` map it stores — koruma's generated error types
//!   for generic structs carry a usage-analysis subset of the generics that
//!   cannot be reconstructed here). Empty forms never emit it (nothing to
//!   validate).

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::DeriveInput;

use crate::derives::gpui_form::errors::participates_in_field_errors;
use crate::derives::gpui_form::structs::FieldOptionality;
use crate::derives::gpui_form::value_holder::form_field_type_tokens;

/// Generates `<Name>FormState` and `<Name>FormPatch` for a single form.
///
/// Spliced into BOTH expansion branches (empty-form early return and main
/// expansion) next to the value-holder and path tokens. Empty forms emit a
/// facade over an empty holder and a field-less patch — and never a
/// validation cache (nothing to validate), so `enable_validation` is
/// meaningless for them.
///
/// `enable_validation` gates the koruma validation cache and its methods;
/// the caller passes the same `needs_koruma_derive` gate that puts
/// `#[derive(Koruma)]` on the holder, restricted to non-generic structs.
pub fn generate_form_state(
    original_input: &DeriveInput,
    fields: &[FieldOptionality],
    enable_validation: bool,
) -> TokenStream {
    let original_ident = &original_input.ident;
    let holder_ident = format_ident!("{}FormValueHolder", original_ident);
    let path_ident = format_ident!("{}FormPath", original_ident);
    let state_ident = format_ident!("{}FormState", original_ident);
    let patch_ident = format_ident!("{}FormPatch", original_ident);
    let errors_ident = format_ident!("{}FormFieldErrors", original_ident);
    let error_enum_ident = format_ident!("{}FormFieldError", original_ident);

    let generics = &original_input.generics;
    let (impl_generics, ty_generics, where_clause) = original_input.generics.split_for_impl();

    let non_skipped: Vec<&FieldOptionality> = fields.iter().filter(|f| !f.skip).collect();

    // ---- validation cache (M2, koruma-gated) --------------------------------

    // Mirror errors.rs: the borrowing enum only carries its lifetime
    // parameter when it has at least one variant (a zero-variant enum
    // cannot reference `'a`).
    let error_enum_has_variants = fields.iter().any(participates_in_field_errors);
    let error_enum_lifetime = if error_enum_has_variants {
        quote! { <'_> }
    } else {
        quote! {}
    };
    let validation_field = if enable_validation {
        quote! {
            /// Outcome cache for a live `is_valid` (fresh iff
            /// `current == validated snapshot`).
            validation: ::gpui_form::core::ValidationCache<#holder_ident #ty_generics, #errors_ident>,
        }
    } else {
        quote! {}
    };
    let validation_init = if enable_validation {
        quote! { validation: ::gpui_form::core::ValidationCache::new(), }
    } else {
        quote! {}
    };
    let invalidate_stmt = if enable_validation {
        quote! { self.validation.invalidate(); }
    } else {
        quote! {}
    };
    let state_struct_doc = if enable_validation {
        quote! {
            /// Per-form state facade composing the standalone core primitives:
            /// baseline/current values via `::gpui_form::core::FormState`,
            /// touched-field tracking via `::gpui_form::core::TouchedFields`,
            /// and a validation-outcome cache via
            /// `::gpui_form::core::ValidationCache` for a live `is_valid` —
            /// all keyed by typed `<Name>FormPath` values.
        }
    } else {
        quote! {
            /// Per-form state facade composing the standalone core primitives:
            /// baseline/current values via `::gpui_form::core::FormState` and
            /// touched-field tracking via `::gpui_form::core::TouchedFields`,
            /// both keyed by typed `<Name>FormPath` values.
        }
    };
    let new_method = if enable_validation {
        quote! {
            /// Create a facade whose baseline and current both start as
            /// `holder`, whose touched set starts empty, and whose
            /// validation cache starts empty.
            pub fn new(holder: #holder_ident #ty_generics) -> Self {
                Self {
                    state: ::gpui_form::core::FormState::new(holder),
                    touched: ::gpui_form::core::TouchedFields::new(#path_ident::FIELD_COUNT),
                    #validation_init
                }
            }
        }
    } else {
        quote! {
            /// Create a facade whose baseline and current both start as
            /// `holder` and whose touched set starts empty.
            pub fn new(holder: #holder_ident #ty_generics) -> Self {
                Self {
                    state: ::gpui_form::core::FormState::new(holder),
                    touched: ::gpui_form::core::TouchedFields::new(#path_ident::FIELD_COUNT),
                }
            }
        }
    };
    let validation_methods = if enable_validation {
        quote! {
            /// Re-run koruma validation on `current`, cache the outcome,
            /// and return it. The raw holder `validate()` stays untouched.
            ///
            /// Call this whenever validation should refresh (a change
            /// handler, a Validate button); the cached outcome then serves
            /// every read — including render-path `is_valid()` — for one
            /// `PartialEq` until the holder changes again.
            pub fn validate(&mut self) -> &#errors_ident {
                let outcome = self.state.current().validate_fields();
                let snapshot = self.state.current().clone();
                self.validation.store(snapshot, outcome);
                match self.validation.stored() {
                    Some(errors) => errors,
                    None => ::core::unreachable!(
                        "ValidationCache::store always leaves a stored outcome"
                    ),
                }
            }

            /// Cached validation outcome: `Some` only when fresh
            /// (`current` equals the validated snapshot); `None` when the
            /// holder changed since the last [`Self::validate`].
            ///
            /// Invalidation contract: the facade's own mutators
            /// (`current_mut`, `replace_current`, both resets) invalidate
            /// up front; `sync_baseline` deliberately does not; the raw
            /// `state_mut()` escape hatch does not either, but the
            /// snapshot comparison still detects the value change (and
            /// [`Self::invalidate_validation`] is the manual recourse).
            pub fn validation(&self) -> Option<&#errors_ident> {
                self.validation.get(self.state.current())
            }

            /// Always-accurate validity: the cached verdict when fresh,
            /// otherwise a fresh `validate_fields()` computed with `&self`
            /// and *not* cached. Never lies between cache generations, so it
            /// is safe in render paths even when the cache is stale.
            pub fn is_valid(&self) -> bool {
                match self.validation.get(self.state.current()) {
                    Some(errors) => errors.is_empty(),
                    None => self.state.current().validate_fields().is_empty(),
                }
            }

            /// Manual escape for callers that bypass the facade's mutators
            /// (e.g. `state_mut().current_mut()`): drop the cached outcome
            /// so the next read recomputes. Not normally needed — the
            /// snapshot comparison already detects value changes — but it
            /// forces a recompute even when a mutated value compares equal
            /// to the validated snapshot.
            pub fn invalidate_validation(&mut self) {
                self.validation.invalidate();
            }

            /// Path-keyed error lookup with caching: returns the cached
            /// verdict when fresh, otherwise validates and caches first
            /// (hence `&mut self`). `None` for valid fields and unknown
            /// paths.
            pub fn error_for(
                &mut self,
                field: &#path_ident,
            ) -> Option<#error_enum_ident #error_enum_lifetime> {
                Self::validate(self).error_for(field)
            }
        }
    } else {
        quote! {}
    };
    let state_mut_method = if enable_validation {
        quote! {
            /// Mutably borrow the wrapped core state. Mutating through this
            /// escape hatch does not touch the touched set.
            ///
            /// It also bypasses validation-cache invalidation: the snapshot
            /// comparison keeps `validation()`/`is_valid()` sound (a changed
            /// holder stops comparing equal), and
            /// [`Self::invalidate_validation`] forces a recompute when
            /// equality alone is not enough.
            pub fn state_mut(
                &mut self,
            ) -> &mut ::gpui_form::core::FormState<#holder_ident #ty_generics> {
                &mut self.state
            }
        }
    } else {
        quote! {
            /// Mutably borrow the wrapped core state. Mutating through this
            /// escape hatch does not touch the touched set.
            pub fn state_mut(
                &mut self,
            ) -> &mut ::gpui_form::core::FormState<#holder_ident #ty_generics> {
                &mut self.state
            }
        }
    };

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

        #state_struct_doc
        pub struct #state_ident #generics {
            state: ::gpui_form::core::FormState<#holder_ident #ty_generics>,
            touched: ::gpui_form::core::TouchedFields,
            #validation_field
        }

        impl #impl_generics #state_ident #ty_generics #where_clause {
            #new_method

            /// Borrow the wrapped core state.
            pub fn state(&self) -> &::gpui_form::core::FormState<#holder_ident #ty_generics> {
                &self.state
            }

            #state_mut_method

            /// Borrow the live (`current`) holder value.
            pub fn current(&self) -> &#holder_ident #ty_generics {
                self.state.current()
            }

            /// Mutably borrow the live holder value. Touched tracking is
            /// event-driven and separate: mutate here, then call
            /// [`Self::touch`] from the component event handler. (On
            /// koruma-gated facades this also invalidates the validation
            /// cache — see [`Self::validation`].)
            pub fn current_mut(&mut self) -> &mut #holder_ident #ty_generics {
                #invalidate_stmt
                self.state.current_mut()
            }

            /// Borrow the saved (`baseline`) holder value.
            pub fn baseline(&self) -> &#holder_ident #ty_generics {
                self.state.baseline()
            }

            /// Overwrite `current` with `holder`; the baseline and the
            /// touched set are untouched. (On koruma-gated facades the
            /// validation cache is invalidated — see [`Self::validation`].)
            pub fn replace_current(&mut self, holder: #holder_ident #ty_generics) {
                #invalidate_stmt
                self.state.replace_current(holder);
            }

            /// Consume the facade and return the live holder value.
            pub fn into_current(self) -> #holder_ident #ty_generics {
                self.state.into_current()
            }

            /// Reset `current` back to the baseline, discard edits, and clear
            /// the touched set — a fresh canvas. (On koruma-gated facades the
            /// validation cache is invalidated too, so error display is
            /// suppressed after a full reset.)
            pub fn reset_to_baseline(&mut self) {
                #invalidate_stmt
                self.state.reset_to_baseline();
                self.touched.clear_all();
            }

            /// Reset `current` back to the baseline but keep the touched set
            /// (undo-style revert: fields stay visited).
            pub fn reset_to_baseline_keeping_touched(&mut self) {
                #invalidate_stmt
                self.state.reset_to_baseline();
            }

            /// Snapshot `current` into `baseline` (call after a successful
            /// save). The touched set is untouched: a save does not un-visit
            /// fields. (On koruma-gated facades the validation cache stays
            /// fresh — validity depends only on `current`, which a sync
            /// never changes.)
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

            #validation_methods

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
