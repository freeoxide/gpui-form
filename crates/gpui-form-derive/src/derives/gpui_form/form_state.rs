//! Generates the per-form state facade (`<Name>FormState`) and patch type
//! (`<Name>FormPatch`) composing the core primitives (`FormState`,
//! `TouchedFields`, and optionally `ValidationCache`) via `<Name>FormPath`.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::DeriveInput;

use crate::derives::gpui_form::errors::participates_in_field_errors;
use crate::derives::gpui_form::structs::{FieldComposition, FieldOptionality};
use crate::derives::gpui_form::value_holder::form_field_type_tokens;

/// Generates `<Name>FormState` and `<Name>FormPatch` for one form, spliced into
/// the derive output. `enable_validation` gates the koruma validation cache and
/// its methods; callers pass the holder's `needs_koruma_derive` gate, non-generic only.
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

    // A zero-variant enum cannot reference `'a`; carry the lifetime only
    // when at least one field participates (mirrors errors.rs).
    let error_enum_has_variants = fields.iter().any(participates_in_field_errors);
    let error_enum_lifetime = if error_enum_has_variants {
        quote! { <'_> }
    } else {
        quote! {}
    };
    let validation_field = if enable_validation {
        quote! {
            /// Outcome cache for a live `is_valid` (fresh iff `current == validated snapshot`).
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
            /// Per-form state facade composing the standalone core primitives (`FormState`, `TouchedFields`, and the validation-outcome cache), keyed by typed `<Name>FormPath` values.
        }
    } else {
        quote! {
            /// Per-form state facade composing the standalone core primitives (`FormState`, `TouchedFields`), keyed by typed `<Name>FormPath` values.
        }
    };
    let new_method = if enable_validation {
        quote! {
            /// Create a facade whose baseline and current both start as `holder`, with an empty touched set and validation cache.
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
            /// Create a facade whose baseline and current both start as `holder` and whose touched set starts empty.
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
            /// Re-run koruma validation on `current`, cache the outcome, and return it.
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

            /// Cached validation outcome: `Some` only when fresh (`current` equals the validated snapshot).
            pub fn validation(&self) -> Option<&#errors_ident> {
                self.validation.get(self.state.current())
            }

            /// Always-accurate validity: the cached verdict when fresh, otherwise a fresh, uncached `validate_fields()`.
            pub fn is_valid(&self) -> bool {
                match self.validation.get(self.state.current()) {
                    Some(errors) => errors.is_empty(),
                    None => self.state.current().validate_fields().is_empty(),
                }
            }

            /// Drop the cached outcome so the next read recomputes (escape for callers bypassing the facade's mutators).
            pub fn invalidate_validation(&mut self) {
                self.validation.invalidate();
            }

            /// Path-keyed error lookup with caching: serves the cached verdict when fresh, otherwise validates and caches first.
            pub fn error_for(
                &mut self,
                field: &#path_ident,
            ) -> Option<#error_enum_ident #error_enum_lifetime> {
                // One trailing shared borrow: a hit returned ahead of
                // `validate` would pin the borrow across it (E0502).
                if self.validation.get(self.state.current()).is_none() {
                    Self::validate(self);
                }
                self.validation.stored()?.error_for(field)
            }
        }
    } else {
        quote! {}
    };
    let state_mut_method = if enable_validation {
        quote! {
            /// Mutably borrow the wrapped core state (escape hatch: no touched-set or validation-cache invalidation).
            pub fn state_mut(
                &mut self,
            ) -> &mut ::gpui_form::core::FormState<#holder_ident #ty_generics> {
                &mut self.state
            }
        }
    } else {
        quote! {
            /// Mutably borrow the wrapped core state (escape hatch: does not touch the touched set).
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
            let compare = quote! {
                self.state.current().#field_name != self.state.baseline().#field_name
            };
            match &f.composition {
                FieldComposition::Flat => quote! {
                    [::gpui_form::core::PathSegment::Name(#field_name_str)] => {
                        #compare
                    },
                },
                // Composed values are opaque to this form; any path rooted at
                // the field compares the value as a whole (deeper paths never assert).
                FieldComposition::Nested { .. } | FieldComposition::List { .. } => quote! {
                    [::gpui_form::core::PathSegment::Name(#field_name_str), ..] => {
                        #compare
                    },
                },
            }
        })
        .collect();
    // Empty forms have no valid paths, so an unknown path there is expected
    // rather than a caller bug — no debug assert.
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
            let ctor_ident = if f.composition.is_flat() {
                field_name.clone()
            } else {
                format_ident!("{}_root", field_name)
            };
            quote! {
                if self.#field_name.is_some() {
                    changed.push(#path_ident::#ctor_ident());
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
        /// Per-form patch: changed holder-typed field values between two holders (`Option` per field; `Some(None)` = changed to cleared; skipped fields absent).
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

            /// Apply the changed fields onto `holder`; unchanged (`None`) fields are left untouched.
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

            /// Mutably borrow the live holder value; touched tracking is event-driven — call [`Self::touch`] from the change handler.
            pub fn current_mut(&mut self) -> &mut #holder_ident #ty_generics {
                #invalidate_stmt
                self.state.current_mut()
            }

            /// Borrow the saved (`baseline`) holder value.
            pub fn baseline(&self) -> &#holder_ident #ty_generics {
                self.state.baseline()
            }

            /// Overwrite `current` with `holder`; the baseline and the touched set are untouched.
            pub fn replace_current(&mut self, holder: #holder_ident #ty_generics) {
                #invalidate_stmt
                self.state.replace_current(holder);
            }

            /// Consume the facade and return the live holder value.
            pub fn into_current(self) -> #holder_ident #ty_generics {
                self.state.into_current()
            }

            /// Reset `current` to the baseline, discard edits, and clear the touched set (a fresh canvas).
            pub fn reset_to_baseline(&mut self) {
                #invalidate_stmt
                self.state.reset_to_baseline();
                self.touched.clear_all();
            }

            /// Reset `current` to the baseline but keep the touched set (undo-style revert).
            pub fn reset_to_baseline_keeping_touched(&mut self) {
                #invalidate_stmt
                self.state.reset_to_baseline();
            }

            /// Snapshot `current` into `baseline` (call after a successful save); the touched set is untouched.
            pub fn sync_baseline(&mut self) {
                self.state.sync_baseline();
            }

            /// Whether the live holder differs from the baseline (whole-value comparison).
            pub fn is_dirty(&self) -> bool {
                self.state.is_dirty()
            }

            /// Whether one specific field differs between baseline and current; unknown paths return `false` (debug-asserted).
            pub fn is_field_dirty(&self, field: &#path_ident) -> bool {
                match field.path().segments() {
                    #(#is_field_dirty_arms)*
                    #is_field_dirty_unknown_arm
                }
            }

            /// Paths of the fields that differ between baseline and current, in declaration order.
            pub fn changed_fields(&self) -> Vec<#path_ident> {
                self.patch().changed_fields()
            }

            /// The field-level diff of baseline to current.
            pub fn patch(&self) -> #patch_ident #ty_generics {
                #patch_ident::between(self.state.baseline(), self.state.current())
            }

            #validation_methods

            /// Mark a field touched (from a blur or change event); unknown paths are a no-op.
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

            /// Clear the touched flag for one field; unknown paths are a no-op.
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
