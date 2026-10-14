//! Bridges koruma's per-holder typed error structs to `<Name>FormPath`:
//! emits `<Name>FormFieldErrors`, `<Name>FormFieldError<'a>`, and the holder's
//! `validate_fields()`, gated on the holder deriving `Koruma` (non-generic only).

use heck::{ToSnakeCase as _, ToUpperCamelCase as _};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Ident, Type};

use crate::derives::gpui_form::structs::{FieldComposition, FieldOptionality};
use crate::derives::gpui_form::value_holder::{
    form_base_type, should_wrap, validation_issue_key_for_kind,
};

enum FieldErrorSlot {
    Wrapper,
    Nested {
        inner: Type,
    },
    BareNewtype {
        inner: Type,
        optional_getter: bool,
    },
    List {
        element_error: Ident,
        has_element_validators: bool,
    },
}

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
    if let Some(rule) = &field.required_if {
        let name = rule.validator.to_string();
        fallbacks.push(name.to_snake_case());
        names.push(name);
    }
    disambiguate_validators(names, fallbacks)
}

// List wrapper getters only: element validators surface via the element
// error struct, not wrapper getters.
fn mirrored_field_validators(field: &FieldOptionality) -> Vec<MirroredValidator> {
    let mut names: Vec<String> = Vec::new();
    let mut fallbacks: Vec<String> = Vec::new();
    if field.needs_required_validation() {
        names.push("RequiredValidation".to_string());
        fallbacks.push("koruma_collection_general_required_validation".to_string());
    }
    for validator in field.validation.field_validators.iter() {
        names.push(validator.name().to_string());
        fallbacks.push(snake_path(&validator.validator));
    }
    if let Some(rule) = &field.required_if {
        let name = rule.validator.to_string();
        fallbacks.push(name.to_snake_case());
        names.push(name);
    }
    disambiguate_validators(names, fallbacks)
}

fn disambiguate_validators(names: Vec<String>, fallbacks: Vec<String>) -> Vec<MirroredValidator> {
    // Mirrors koruma's getter disambiguation: on a snake_case collision, use
    // the full path flattened to snake_case.
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

// Returns the nested map by value (the nested enum borrows its own map, not
// this one); flat `#[koruma(nested)]` fields resolve no convention, so no accessor.
fn nested_form_accessor(
    inner: &Type,
    field: &FieldOptionality,
    name: &Ident,
) -> Option<TokenStream> {
    if !matches!(field.composition, FieldComposition::Nested { .. }) {
        return None;
    }
    let Type::Path(type_path) = inner else {
        return None;
    };
    let holder_name = type_path.path.segments.last()?.ident.to_string();
    let nested_struct = holder_name.strip_suffix("FormValueHolder")?;
    let nested_errors_ident = format_ident!("{nested_struct}FormFieldErrors");
    let accessor_ident = format_ident!("{name}_errors");

    Some(quote! {
        /// Re-key this field's nested validation error into the nested form's
        /// own path-keyed map: `errors.#accessor_ident()?.error_for(&..FormPath::city())`.
        pub fn #accessor_ident(&self) -> ::core::option::Option<#nested_errors_ident> {
            self.#name
                .as_ref()
                .map(#nested_errors_ident::from_koruma_errors)
        }
    })
}

/// Whether a field gets an error-map slot: koruma emits a per-field error
/// getter only for holder fields carrying koruma attrs, and `value_holder`
/// mirrors attrs onto exactly those fields — both surfaces must agree.
pub(crate) fn participates_in_field_errors(field: &FieldOptionality) -> bool {
    !field.skip
        && (field.needs_required_validation()
            || !field.validation.field_validators.is_empty()
            || !field.validation.element_validators.is_empty()
            || field.validation.is_newtype
            || (field.composition.is_nested() && field.validation.is_nested)
            || field.required_if.is_some())
}

/// Generates `<Name>FormFieldErrors`, `<Name>FormFieldError<'a>`, and the
/// holder's `validate_fields()`. Returns an empty stream for generic source
/// structs.
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

    let participating: Vec<&FieldOptionality> = fields
        .iter()
        .filter(|f| participates_in_field_errors(f))
        .collect();

    struct FieldSlots {
        name: Ident,
        variant: Ident,
        slot_inner: TokenStream,
        populate: TokenStream,
        issue_keys_body: TokenStream,
        composition: SlotRouting,
        nested_accessor: Option<TokenStream>,
        list_element_accessor: Option<TokenStream>,
        failing_push: Option<TokenStream>,
    }

    #[derive(Clone, Copy)]
    enum SlotRouting {
        Exact,
        Prefix,
        List { indexed: bool },
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

            let slot = match &f.composition {
                FieldComposition::List { .. } => FieldErrorSlot::List {
                    element_error: format_ident!(
                        "{}{}ElementKorumaValidationError",
                        holder_ident,
                        name.to_string().to_upper_camel_case()
                    ),
                    has_element_validators: !f.validation.element_validators.is_empty(),
                },
                FieldComposition::Flat if f.validation.is_nested => {
                    FieldErrorSlot::Nested {
                        inner: form_base_type(f),
                    }
                },
                FieldComposition::Nested { .. } if f.validation.is_nested => {
                    // Same slot as flat `#[koruma(nested)]`, but `inner` is the
                    // nested form's HOLDER, so `ValidateExt::Error` is the nested form's own.
                    FieldErrorSlot::Nested {
                        inner: form_base_type(f),
                    }
                },
                FieldComposition::Nested { .. } => {
                    // No `#[koruma(nested)]`: no koruma getter exists; other
                    // validators route this field to the wrapper slot.
                    FieldErrorSlot::Wrapper
                },
                FieldComposition::Flat if f.validation.is_newtype && !newtype_with_validators => {
                    FieldErrorSlot::BareNewtype {
                        inner: form_base_type(f),
                        optional_getter: should_wrap(f),
                    }
                },
                _ => FieldErrorSlot::Wrapper,
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
                        composition: SlotRouting::Exact,
                        nested_accessor: None,
                        list_element_accessor: None,
                        failing_push: None,
                        name,
                        variant,
                    }
                },
                FieldErrorSlot::Nested { inner } => {
                    let nested_accessor = nested_form_accessor(&inner, f, &name);
                    FieldSlots {
                        slot_inner: quote! { <#inner as ::koruma::ValidateExt>::Error },
                        populate: quote! {
                            errors
                                .#name()
                                .filter(|inner| !::koruma::ValidationError::is_empty(*inner))
                                .cloned()
                        },
                        // Inner validator kinds live on the nested type; report
                        // the generic key here.
                        issue_keys_body: quote! { vec!["validation.invalid"] },
                        composition: SlotRouting::Prefix,
                        nested_accessor,
                        list_element_accessor: None,
                        failing_push: None,
                        name,
                        variant,
                    }
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
                        composition: SlotRouting::Exact,
                        nested_accessor: None,
                        list_element_accessor: None,
                        failing_push: None,
                        name,
                        variant,
                    }
                },
                FieldErrorSlot::List {
                    element_error,
                    has_element_validators,
                } => {
                    let wrapper_ident = format_ident!(
                        "{}{}KorumaValidationError",
                        holder_ident,
                        name.to_string().to_upper_camel_case()
                    );
                    let populate = quote! {
                        (!errors.#name().is_empty()).then(|| errors.#name().clone())
                    };

                    let field_validators = mirrored_field_validators(f);
                    let element_keys: Vec<String> = f
                        .validation
                        .element_validators
                        .iter()
                        .map(|validator| validation_issue_key_for_kind(&validator.name().to_string()))
                        .collect();
                    let issue_keys_body = if field_validators.is_empty() && element_keys.is_empty()
                    {
                        quote! { Vec::new() }
                    } else {
                        let field_checks = field_validators.iter().map(|v| {
                            let getter = &v.getter;
                            let key = &v.key;
                            quote! {
                                if field.#getter().is_some() {
                                    keys.push(#key);
                                }
                            }
                        });
                        let element_checks = if element_keys.is_empty() {
                            quote! {}
                        } else {
                            quote! {
                                if !field.element_errors().is_empty() {
                                    #( keys.push(#element_keys); )*
                                }
                            }
                        };
                        quote! {
                            let mut keys = Vec::new();
                            #(#field_checks)*
                            #element_checks
                            keys
                        }
                    };

                    let element_error_accessor =
                        format_ident!("{}_element_error", name);
                    let list_element_accessor = has_element_validators.then(|| {
                        quote! {
                            /// The element error for item `index`, when that
                            /// item failed its `each(...)` validators.
                            pub fn #element_error_accessor(&self, index: usize)
                                -> Option<&#element_error> {
                                self.#name.as_ref()?.element_errors().iter()
                                    .find(|(failed_at, _)| *failed_at == index)
                                    .map(|(_, error)| error)
                            }
                        }
                    });

                    // Report per-item paths plus the root: the wrapper slot is
                    // populated whenever anything failed, so the root resolves too.
                    let name_root_ctor = format_ident!("{}_root", name);
                    let failing_push = has_element_validators.then(|| {
                        quote! {
                            if let Some(field) = &self.#name {
                                for (failed_at, _) in field.element_errors() {
                                    failing.push((
                                        #path_ident::#name(*failed_at),
                                        #error_enum_ident::#variant(field),
                                    ));
                                }
                                failing.push((
                                    #path_ident::#name_root_ctor(),
                                    #error_enum_ident::#variant(field),
                                ));
                            }
                        }
                    });

                    FieldSlots {
                        slot_inner: quote! { #wrapper_ident },
                        populate,
                        issue_keys_body,
                        composition: SlotRouting::List {
                            indexed: has_element_validators,
                        },
                        nested_accessor: None,
                        list_element_accessor,
                        failing_push,
                        name,
                        variant,
                    }
                },
            }
        })
        .collect();

    let has_fields = !slots.is_empty();
    // A zero-variant enum cannot reference `'a` (E0392); carry the lifetime
    // only when variants exist.
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
            match s.composition {
                SlotRouting::Exact => quote! {
                    [::gpui_form::core::PathSegment::Name(#name_str)] =>
                        self.#name.as_ref().map(#error_enum_ident::#variant),
                },
                SlotRouting::Prefix => quote! {
                    // Any `<field>...` path routes to the nested slot; deeper
                    // segments resolve via `<field>_errors()`.
                    [::gpui_form::core::PathSegment::Name(#name_str), ..] =>
                        self.#name.as_ref().map(#error_enum_ident::#variant),
                },
                SlotRouting::List { indexed } => {
                    let root_arm = quote! {
                        [::gpui_form::core::PathSegment::Name(#name_str)] =>
                            self.#name.as_ref().map(#error_enum_ident::#variant),
                    };
                    if indexed {
                        quote! {
                            #root_arm
                            // `field.<i>`: `Some` only when item `i` failed its
                            // `each(..)` validators; see `<field>_element_error`.
                            [
                                ::gpui_form::core::PathSegment::Name(#name_str),
                                ::gpui_form::core::PathSegment::Index(index),
                                ..
                            ] => match self.#name.as_ref() {
                                ::core::option::Option::Some(wrapper) => wrapper
                                    .element_errors()
                                    .iter()
                                    // `index` binds by reference through the
                                    // slice pattern; compare the pointee.
                                    .any(|failed_at| failed_at.0 == *index)
                                    .then(|| #error_enum_ident::#variant(wrapper)),
                                ::core::option::Option::None => ::core::option::Option::None,
                            },
                        }
                    } else {
                        root_arm
                    }
                },
            }
        })
        .collect::<Vec<_>>();

    let failing_fields_pushes = slots
        .iter()
        .map(|s| {
            let name = &s.name;
            let variant = &s.variant;
            if let Some(custom) = &s.failing_push {
                return custom.clone();
            }
            let root_ctor = match s.composition {
                SlotRouting::Exact => name.clone(),
                SlotRouting::Prefix | SlotRouting::List { .. } => {
                    format_ident!("{}_root", name)
                },
            };
            quote! {
                if let Some(field) = &self.#name {
                    failing.push((#path_ident::#root_ctor(), #error_enum_ident::#variant(field)));
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
            let root_ident = match s.composition {
                SlotRouting::Exact => name.clone(),
                SlotRouting::Prefix | SlotRouting::List { .. } => {
                    format_ident!("{}_root", name)
                },
            };
            quote! { Self::#variant(_) => #path_ident::#root_ident(), }
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

    let nested_and_list_accessors: Vec<TokenStream> = slots
        .iter()
        .filter_map(|s| {
            let nested = s.nested_accessor.as_ref();
            let list = s.list_element_accessor.as_ref();
            match (nested, list) {
                (Some(tokens), _) | (None, Some(tokens)) => Some(tokens.clone()),
                (None, None) => None,
            }
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
            #(#nested_and_list_accessors)*

            /// Path-addressable lookup: `Some` only when `field` names a failing field.
            /// Nested fields route any `<field>...` path to their slot; list fields
            /// route `<field>` plus typed-index `<field>.<i>` paths. Unknown: `None`.
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

            /// i18n keys for the failed field-level validators (e.g. `"validation.required"`),
            /// in koruma order. Inner newtype/nested failures report the
            /// generic `"validation.invalid"` key; use the typed accessors for those.
            pub fn issue_keys(&self) -> Vec<&'static str> {
                #enum_issue_keys_body
            }
        }

        #holder_impl
    }
}
