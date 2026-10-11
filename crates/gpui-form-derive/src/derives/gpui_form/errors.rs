//! Generator for the per-form typed field-error surface (backlog #6).
//!
//! Koruma already generates a concrete, typed per-field error struct per
//! holder (`<Holder>KorumaValidationError` with one getter per field); what
//! is missing is a *bridge* from those typed structs to `<Name>FormPath`.
//! This module emits that bridge:
//!
//! - `<Name>FormFieldErrors`: a per-form map with one private slot per
//!   non-skipped holder field, re-keying koruma's getters by typed path.
//! - `<Name>FormFieldError<'a>`: a borrowing enum with one variant per
//!   field, for path-addressable/loop code (render all failing fields).
//! - `validate_fields()` on the holder: run koruma validation once and
//!   re-key the result. The raw koruma `validate()` stays untouched.
//!
//! # Gate
//!
//! Emitted only when the holder itself derives `Koruma`
//! (`needs_koruma_derive` in `value_holder.rs` — NOT
//! `koruma_options.is_some()`, which would drop field-attr-only koruma
//! forms), so every koruma error type referenced here is guaranteed to
//! exist. Generic source structs are excluded in FLAT v1: koruma's
//! generated error types carry a *subset* of the source generics chosen by
//! usage analysis, which cannot be reconstructed here; generic koruma forms
//! keep compiling exactly as before, minus this opt-in surface.
//!
//! # Scope (FLAT v1)
//!
//! - Element-validator errors and nested/newtype *inner* validator kinds are
//!   reachable through the typed accessors and the enum payloads; only
//!   field-level validator kinds get named `issue_keys()` entries (inner
//!   failures report the generic `validation.invalid` key).
//! - The map holds owned clones of koruma error structs (koruma errors
//!   derive `Clone, Debug, Default` only — no `PartialEq`, no serde).

use heck::{ToSnakeCase as _, ToUpperCamelCase as _};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Ident, Type};

use crate::derives::gpui_form::structs::FieldOptionality;
use crate::derives::gpui_form::value_holder::{
    form_base_type, should_wrap, validation_issue_key_for_kind,
};

/// How a field's koruma error surfaces on the generated map.
enum FieldErrorSlot {
    /// Regular field or newtype-with-validators: koruma's per-field error
    /// struct, reached through a `&Wrapper` getter.
    Wrapper,
    /// `#[koruma(nested)]` field: the nested type's own
    /// `ValidateExt::Error`, behind an `Option<&..>` getter.
    Nested { inner: Type },
    /// `#[koruma(newtype)]` with no mirrored validators: the inner type's
    /// `ValidateExt::Error`, reached either directly or behind an `Option`.
    BareNewtype { inner: Type, optional_getter: bool },
}

/// One validator kind mirrored onto the holder field, carrying the getter
/// name koruma generates (mirroring `koruma-derive`'s
/// `validator_field_ident`) and the i18n key for its failure.
struct MirroredValidator {
    getter: Ident,
    key: String,
}

fn snake_path(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string().to_snake_case())
        .collect::<Vec<_>>()
        .join("_")
}

fn mirrored_validators(field: &FieldOptionality) -> Vec<MirroredValidator> {
    // The holder's koruma attr list: synthetic required first, then the
    // source field validators, then element validators mirrored flat
    // (see `value_holder.rs` — element validators are re-emitted without
    // `each(..)`, so koruma parses them as field validators of the holder).
    let mut names: Vec<String> = Vec::new();
    let mut fallbacks: Vec<String> = Vec::new();
    if field.needs_required_validation() {
        names.push("RequiredValidation".to_string());
        fallbacks.push("koruma_collection_general_required_validation".to_string());
    }
    for validator in field
        .validation
        .field_validators
        .iter()
        .chain(field.validation.element_validators.iter())
    {
        names.push(validator.name().to_string());
        fallbacks.push(snake_path(&validator.validator));
    }

    // Collision fallback mirrors koruma: snake of the last path segment,
    // then the full path flattened to snake_case. (The final hash suffix
    // path only triggers for two validators with identical full paths,
    // which koruma itself cannot disambiguate.)
    let snake_names: Vec<String> = names.iter().map(|n| n.to_snake_case()).collect();
    let mut mirrored = Vec::with_capacity(names.len());
    for (index, name) in names.iter().enumerate() {
        let simple = &snake_names[index];
        let getter = if snake_names.iter().filter(|n| *n == simple).count() > 1 {
            &fallbacks[index]
        } else {
            simple
        };
        mirrored.push(MirroredValidator {
            getter: format_ident!("{getter}"),
            key: validation_issue_key_for_kind(name),
        });
    }
    mirrored
}

/// Generates `<Name>FormFieldErrors`, `<Name>FormFieldError<'a>`, and the
/// holder's `validate_fields()`. Returns an empty stream for generic source
/// structs (see module docs).
pub fn generate_field_errors(
    original_input: &DeriveInput,
    fields: &[FieldOptionality],
) -> TokenStream {
    if !original_input.generics.params.is_empty() {
        return quote! {};
    }

    let original_ident = &original_input.ident;
    let holder_ident = format_ident!("{}FormValueHolder", original_ident);
    let path_ident = format_ident!("{}FormPath", original_ident);
    let errors_ident = format_ident!("{}FormFieldErrors", original_ident);
    let error_enum_ident = format_ident!("{}FormFieldError", original_ident);
    let koruma_error_ident = format_ident!("{}KorumaValidationError", holder_ident);
    let (impl_generics, ty_generics, where_clause) = original_input.generics.split_for_impl();

    // Koruma generates a top-level error getter (and per-field error struct)
    // only for holder fields that actually carry koruma attrs. `value_holder`
    // mirrors attrs onto exactly these fields — mirror that predicate here so
    // the map never references a getter that does not exist. (Attr-less
    // fields are simply absent from the map; they can never fail.)
    let participating: Vec<&FieldOptionality> = fields
        .iter()
        .filter(|f| {
            !f.skip
                && (f.needs_required_validation()
                    || !f.validation.field_validators.is_empty()
                    || !f.validation.element_validators.is_empty()
                    || f.validation.is_newtype)
        })
        .collect();

    struct FieldSlots {
        name: Ident,
        variant: Ident,
        slot_inner: TokenStream,
        populate: TokenStream,
        issue_keys_body: TokenStream,
    }

    let slots: Vec<FieldSlots> = participating
        .iter()
        .map(|f| {
            let name = f.field_name.clone();
            let variant = format_ident!("{}", name.to_string().to_upper_camel_case());
            let newtype_with_validators = f.validation.is_newtype
                && (f.needs_required_validation()
                    || !f.validation.field_validators.is_empty()
                    || !f.validation.element_validators.is_empty());

            let slot = if f.validation.is_nested {
                FieldErrorSlot::Nested {
                    inner: form_base_type(f),
                }
            } else if f.validation.is_newtype && !newtype_with_validators {
                FieldErrorSlot::BareNewtype {
                    inner: form_base_type(f),
                    optional_getter: should_wrap(f),
                }
            } else {
                FieldErrorSlot::Wrapper
            };

            match slot {
                FieldErrorSlot::Wrapper => {
                    let wrapper_ident = format_ident!(
                        "{}{}KorumaValidationError",
                        holder_ident,
                        name.to_string().to_upper_camel_case()
                    );
                    let populate = quote! {
                        (!errors.#name().is_empty()).then(|| errors.#name().clone())
                    };
                    let validators = mirrored_validators(f);
                    let newtype_inner_check = if f.validation.is_newtype {
                        if should_wrap(f) {
                            quote! {
                                if field
                                    .inner()
                                    .is_some_and(|inner| !::koruma::ValidationError::is_empty(inner))
                                {
                                    keys.push("validation.invalid");
                                }
                            }
                        } else {
                            quote! {
                                if !::koruma::ValidationError::is_empty(field.inner()) {
                                    keys.push("validation.invalid");
                                }
                            }
                        }
                    } else {
                        quote! {}
                    };
                    let issue_keys_body =
                        if validators.is_empty() && newtype_inner_check.is_empty() {
                            quote! { Vec::new() }
                        } else {
                            let checks = validators.iter().map(|v| {
                                let getter = &v.getter;
                                let key = &v.key;
                                quote! {
                                    if field.#getter().is_some() {
                                        keys.push(#key);
                                    }
                                }
                            });
                            quote! {
                                let mut keys = Vec::new();
                                #(#checks)*
                                #newtype_inner_check
                                keys
                            }
                        };
                    FieldSlots {
                        slot_inner: quote! { #wrapper_ident },
                        populate,
                        issue_keys_body,
                        name,
                        variant,
                    }
                },
                FieldErrorSlot::Nested { inner } => FieldSlots {
                    slot_inner: quote! { <#inner as ::koruma::ValidateExt>::Error },
                    populate: quote! {
                        errors
                            .#name()
                            .filter(|inner| !::koruma::ValidationError::is_empty(*inner))
                            .cloned()
                    },
                    // Inner validator kinds live on the nested type and are
                    // invisible here; report the generic key.
                    issue_keys_body: quote! { vec!["validation.invalid"] },
                    name,
                    variant,
                },
                FieldErrorSlot::BareNewtype {
                    inner,
                    optional_getter,
                } => {
                    let populate = if optional_getter {
                        quote! {
                            errors
                                .#name()
                                .filter(|inner| !::koruma::ValidationError::is_empty(*inner))
                                .cloned()
                        }
                    } else {
                        quote! {
                            (!::koruma::ValidationError::is_empty(errors.#name()))
                                .then(|| errors.#name().clone())
                        }
                    };
                    FieldSlots {
                        slot_inner: quote! { <#inner as ::koruma::ValidateExt>::Error },
                        populate,
                        issue_keys_body: quote! { vec!["validation.invalid"] },
                        name,
                        variant,
                    }
                },
            }
        })
        .collect();

    let has_fields = !slots.is_empty();
    // The borrowing enum only needs its lifetime parameter when it has at
    // least one variant (an uninhabited zero-field enum cannot reference
    // `'a`, or it trips E0392).
    let enum_lifetime_params = if has_fields {
        quote! { <'a> }
    } else {
        quote! {}
    };
    let error_for_signature = if has_fields {
        quote! {
            pub fn error_for<'a>(&'a self, field: &#path_ident) -> Option<#error_enum_ident<'a>>
        }
    } else {
        quote! {
            pub fn error_for(&self, field: &#path_ident) -> Option<#error_enum_ident>
        }
    };
    let failing_fields_signature = if has_fields {
        quote! {
            pub fn failing_fields<'a>(&'a self)
                -> impl Iterator<Item = (#path_ident, #error_enum_ident<'a>)> + 'a
        }
    } else {
        quote! {
            pub fn failing_fields(&self) -> impl Iterator<Item = (#path_ident, #error_enum_ident)> + '_
        }
    };

    let map_fields = slots
        .iter()
        .map(|s| {
            let name = &s.name;
            let inner = &s.slot_inner;
            quote! { #name: Option<#inner> }
        })
        .collect::<Vec<_>>();

    let map_accessors = slots
        .iter()
        .map(|s| {
            let name = &s.name;
            let inner = &s.slot_inner;
            quote! {
                /// Typed accessor for this field's koruma error struct
                /// (`None` when the field passes all of its validators).
                pub fn #name(&self) -> Option<&#inner> {
                    self.#name.as_ref()
                }
            }
        })
        .collect::<Vec<_>>();

    let map_is_empty_checks = slots
        .iter()
        .map(|s| {
            let name = &s.name;
            quote! { self.#name.is_none() }
        })
        .collect::<Vec<_>>();
    let map_is_empty_body = if map_is_empty_checks.is_empty() {
        quote! { true }
    } else {
        quote! { #(#map_is_empty_checks)&&* }
    };

    let error_for_arms = slots
        .iter()
        .map(|s| {
            let name = &s.name;
            let name_str = name.to_string();
            let variant = &s.variant;
            quote! {
                [#name_str] => self.#name.as_ref().map(#error_enum_ident::#variant),
            }
        })
        .collect::<Vec<_>>();

    let failing_fields_pushes = slots
        .iter()
        .map(|s| {
            let name = &s.name;
            let variant = &s.variant;
            quote! {
                if let Some(field) = &self.#name {
                    failing.push((#path_ident::#name(), #error_enum_ident::#variant(field)));
                }
            }
        })
        .collect::<Vec<_>>();

    let enum_variants = slots
        .iter()
        .map(|s| {
            let variant = &s.variant;
            let inner = &s.slot_inner;
            quote! { #variant(&'a #inner) }
        })
        .collect::<Vec<_>>();

    let enum_path_arms = slots
        .iter()
        .map(|s| {
            let name = &s.name;
            let variant = &s.variant;
            quote! { Self::#variant(_) => #path_ident::#name(), }
        })
        .collect::<Vec<_>>();

    let enum_issue_keys_arms = slots
        .iter()
        .map(|s| {
            let variant = &s.variant;
            let body = &s.issue_keys_body;
            quote! { Self::#variant(field) => { #body }, }
        })
        .collect::<Vec<_>>();

    let enum_path_body = if has_fields {
        quote! { match self { #(#enum_path_arms)* } }
    } else {
        quote! { match *self {} }
    };
    let enum_issue_keys_body = if has_fields {
        quote! { match self { #(#enum_issue_keys_arms)* } }
    } else {
        quote! { match *self {} }
    };

    let from_koruma_fields = slots
        .iter()
        .map(|s| {
            let name = &s.name;
            let populate = &s.populate;
            quote! { #name: #populate }
        })
        .collect::<Vec<_>>();

    let holder_impl = quote! {
        impl #impl_generics #holder_ident #ty_generics #where_clause {
            /// Run koruma validation once and re-key the result by typed
            /// paths. The raw koruma `validate()` is untouched;
            /// `map.is_empty()` == `validate().is_ok()`.
            pub fn validate_fields(&self) -> #errors_ident {
                match self.validate() {
                    Ok(()) => #errors_ident::default(),
                    Err(errors) => #errors_ident::from_koruma_errors(&errors),
                }
            }
        }
    };

    quote! {
        /// Per-form field-error map: koruma's typed per-field error structs
        /// re-keyed by `<Name>FormPath`. One slot per non-skipped field;
        /// `None` means the field passes all of its validators.
        #[derive(Clone, Debug, Default)]
        pub struct #errors_ident {
            #(#map_fields),*
        }

        impl #errors_ident {
            /// Re-key a koruma validation error into the path-keyed map,
            /// cloning only the failing fields' error structs.
            pub fn from_koruma_errors(errors: &#koruma_error_ident) -> Self {
                Self {
                    #(#from_koruma_fields),*
                }
            }

            /// Whether no field has errors.
            pub fn is_empty(&self) -> bool {
                #map_is_empty_body
            }

            /// Whether any field has errors.
            pub fn has_errors(&self) -> bool {
                !#errors_ident::is_empty(self)
            }

            #(#map_accessors)*

            /// Path-addressable lookup: `Some` only when `field` names a
            /// failing field. Unknown paths return `None`.
            #error_for_signature {
                match field.path().segments() {
                    #(#error_for_arms)*
                    _ => None,
                }
            }

            /// (path, error) pairs in declaration order, failing fields only.
            #failing_fields_signature {
                let mut failing = Vec::new();
                #(#failing_fields_pushes)*
                failing.into_iter()
            }
        }

        impl ::gpui_form::core::FormFieldErrors for #errors_ident {
            fn is_empty(&self) -> bool {
                #errors_ident::is_empty(self)
            }

            fn failing_paths(&self) -> Vec<::gpui_form::core::FieldPath> {
                self.failing_fields()
                    .map(|(path, _)| path.into_path())
                    .collect()
            }
        }

        /// One variant per field, borrowing that field's koruma error.
        #[derive(Clone, Debug)]
        pub enum #error_enum_ident #enum_lifetime_params {
            #(#enum_variants),*
        }

        impl #enum_lifetime_params #error_enum_ident #enum_lifetime_params {
            /// The typed path of the failing field.
            pub fn path(&self) -> #path_ident {
                #enum_path_body
            }

            /// i18n keys for the failed field-level validators (e.g.
            /// `"validation.required"`), in koruma order. Inner newtype and
            /// nested failures report the generic `"validation.invalid"` key;
            /// resolve those through the typed accessors when the app knows
            /// the inner kinds.
            pub fn issue_keys(&self) -> Vec<&'static str> {
                #enum_issue_keys_body
            }
        }

        #holder_impl
    }
}
