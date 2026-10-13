use heck::{ToShoutySnakeCase as _, ToSnakeCase as _};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::{BTreeSet, HashMap};
use syn::{DeriveInput, Type};

use crate::derives::gpui_form::koruma::validator_attr_to_tokens;
use crate::derives::gpui_form::structs::{ComponentField, FieldComposition, FieldOptionality};
use crate::derives::gpui_form::utils::extract_option_inner_type;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FieldStorage {
    OriginallyOptional,
    WrappedOption,
    Plain,
    Nested,
    List,
}

fn field_storage(field: &FieldOptionality) -> FieldStorage {
    match &field.composition {
        FieldComposition::Flat => {
            if field.was_optional {
                FieldStorage::OriginallyOptional
            } else if field.wrap_in_option {
                FieldStorage::WrappedOption
            } else {
                FieldStorage::Plain
            }
        },
        FieldComposition::Nested { .. } => FieldStorage::Nested,
        FieldComposition::List { .. } => FieldStorage::List,
    }
}

pub(crate) fn should_wrap(field: &FieldOptionality) -> bool {
    if field.skip {
        return false;
    }
    match field_storage(field) {
        FieldStorage::Plain => false,
        FieldStorage::OriginallyOptional | FieldStorage::WrappedOption => true,
        // Components never wrap composed fields, so `koruma(nested)` /
        // `each(..)` apply directly; wrap only when the source was `Option<..>`.
        FieldStorage::Nested | FieldStorage::List => field.was_optional,
    }
}

pub(crate) fn form_base_type(field: &FieldOptionality) -> Type {
    match &field.composition {
        FieldComposition::Nested { holder } => {
            if let Some(override_type) = &field.override_type {
                extract_option_inner_type(override_type).1
            } else {
                holder.clone()
            }
        },
        // `inner_type` is already Option-stripped; the holder stores the full `Vec`.
        FieldComposition::List { .. } => field.inner_type.clone(),
        FieldComposition::Flat => {
            if let Some(override_type) = &field.override_type {
                extract_option_inner_type(override_type).1
            } else {
                field.inner_type.clone()
            }
        },
    }
}

pub(crate) fn form_field_type_tokens(field: &FieldOptionality) -> TokenStream {
    let base_type = form_base_type(field);
    if should_wrap(field) {
        quote! { Option<#base_type> }
    } else {
        quote! { #base_type }
    }
}

fn apply_from_conversion(field: &FieldOptionality, value: TokenStream) -> TokenStream {
    if let Some(expr) = &field.from_expr {
        quote! { (#expr)(#value) }
    } else if field.override_type.is_some() || field.composition.is_nested() {
        quote! { ::core::convert::From::from(#value) }
    } else {
        value
    }
}

fn apply_into_conversion(field: &FieldOptionality, value: TokenStream) -> TokenStream {
    if let Some(expr) = &field.into_expr {
        quote! { (#expr)(#value) }
    } else if field.override_type.is_some() || field.composition.is_nested() {
        quote! { ::core::convert::From::from(#value) }
    } else {
        value
    }
}

fn needs_from_conversion(field: &FieldOptionality) -> bool {
    field.from_expr.is_some() || field.override_type.is_some() || field.composition.is_nested()
}

fn needs_into_conversion(field: &FieldOptionality) -> bool {
    field.into_expr.is_some() || field.override_type.is_some() || field.composition.is_nested()
}

fn try_from_field_tokens(
    field: &FieldOptionality,
    source: TokenStream,
    error_type: &syn::Ident,
) -> TokenStream {
    let field_name = &field.field_name;
    let access = quote! { #source.#field_name };
    match field_storage(field) {
        FieldStorage::OriginallyOptional => optional_value_try_from(field, &access, error_type),
        FieldStorage::Nested if field.was_optional => {
            optional_value_try_from(field, &access, error_type)
        },
        FieldStorage::WrappedOption => wrapped_option_try_from(field, &access, error_type),
        FieldStorage::Plain | FieldStorage::Nested | FieldStorage::List => {
            let converted = apply_into_conversion(field, access);
            quote! {
                #field_name: #converted
            }
        },
    }
}

fn optional_value_try_from(
    field: &FieldOptionality,
    access: &TokenStream,
    _error_type: &syn::Ident,
) -> TokenStream {
    let field_name = &field.field_name;
    if needs_into_conversion(field) {
        let converted = apply_into_conversion(field, quote! { value });
        quote! {
            #field_name: #access.map(|value| #converted)
        }
    } else {
        quote! {
            #field_name: #access
        }
    }
}

fn wrapped_option_try_from(
    field: &FieldOptionality,
    access: &TokenStream,
    error_type: &syn::Ident,
) -> TokenStream {
    let field_name = &field.field_name;
    let field_name_str = field_name.to_string();
    if needs_into_conversion(field) {
        let converted = apply_into_conversion(field, quote! { value });
        quote! {
            #field_name: {
                let value = #access.ok_or(#error_type{
                    field_name: #field_name_str,
                    index: ::core::option::Option::None
                })?;
                #converted
            }
        }
    } else {
        quote! {
            #field_name: #access.ok_or(#error_type{
                field_name: #field_name_str,
                index: ::core::option::Option::None
            })?
        }
    }
}

fn generate_conversion_error_type(error_name: &syn::Ident) -> TokenStream {
    quote! {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct #error_name {
            pub field_name: &'static str,
            /// Failing list-item index for list-field conversion failures (`None` for whole-field failures).
            pub index: ::core::option::Option<usize>,
        }

        impl ::core::fmt::Display for #error_name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                match self.index {
                    ::core::option::Option::Some(index) => write!(
                        f,
                        "Failed to convert item {} of field '{}'",
                        index, self.field_name
                    ),
                    ::core::option::Option::None => write!(
                        f,
                        "Failed to unwrap an Option for field '{}', found None",
                        self.field_name
                    ),
                }
            }
        }

        impl ::core::error::Error for #error_name {}
    }
}

fn unwrap_expr(expr: &syn::Expr) -> &syn::Expr {
    match expr {
        syn::Expr::Group(group) => unwrap_expr(&group.expr),
        syn::Expr::Paren(paren) => unwrap_expr(&paren.expr),
        _ => expr,
    }
}

fn should_wrap_default_into(expr: &syn::Expr) -> bool {
    matches!(unwrap_expr(expr), syn::Expr::Lit(_))
}

fn default_expr_for_original(expr: &syn::Expr) -> TokenStream {
    let expr_tokens = quote! { #expr };
    if should_wrap_default_into(expr) {
        quote! { ::core::convert::Into::into(#expr_tokens) }
    } else {
        expr_tokens
    }
}

fn generate_default_impl(fields: &[FieldOptionality], struct_name: &syn::Ident) -> TokenStream {
    let default_fields: Vec<TokenStream> = fields
        .iter()
        .filter(|f| !f.skip)
        .map(|f| {
            let field_name = &f.field_name;
            if let Some(default_expr) = &f.default_expr {
                let default_original = default_expr_for_original(default_expr);
                let default_value = apply_from_conversion(f, default_original);
                if should_wrap(f) {
                    quote! {
                        #field_name: Some(#default_value)
                    }
                } else {
                    quote! {
                        #field_name: #default_value
                    }
                }
            } else if should_wrap(f) {
                quote! {
                    #field_name: None
                }
            } else {
                quote! {
                    #field_name: ::core::default::Default::default()
                }
            }
        })
        .collect();

    quote! {
        impl ::core::default::Default for #struct_name {
            fn default() -> Self {
                Self {
                    #(#default_fields),*
                }
            }
        }
    }
}

fn optional_value_to_wrapped(field: &FieldOptionality) -> TokenStream {
    let field_name = &field.field_name;
    if needs_from_conversion(field) {
        let converted = apply_from_conversion(field, quote! { value });
        quote! {
            #field_name: from.#field_name.map(|value| #converted)
        }
    } else {
        quote! {
            #field_name: from.#field_name
        }
    }
}

fn optional_value_from_wrapped(field: &FieldOptionality) -> TokenStream {
    let field_name = &field.field_name;
    if needs_into_conversion(field) {
        let converted = apply_into_conversion(field, quote! { value });
        quote! {
            #field_name: from.#field_name.map(|value| #converted)
        }
    } else {
        quote! {
            #field_name: from.#field_name
        }
    }
}

fn optional_value_present_json(field: &FieldOptionality) -> TokenStream {
    let field_name = &field.field_name;
    let field_name_str = field_name.to_string();
    // Nested entries skip into-conversion: `From::from` is inference-ambiguous
    // in the `format!("{:?}")` dump context (both directions exist).
    if needs_into_conversion(field) && field.composition.is_flat() {
        let converted = apply_into_conversion(field, quote! { value });
        quote! {
            let converted = self.#field_name.clone().map(|value| #converted);
            if converted.is_some() {
                entries.push(format!(
                    "\"{}\":\"{}\"",
                    #field_name_str,
                    Self::escape_json_string(&format!("{:?}", converted))
                ));
            }
        }
    } else {
        quote! {
            if self.#field_name.is_some() {
                entries.push(format!(
                    "\"{}\":\"{}\"",
                    #field_name_str,
                    Self::escape_json_string(&format!("{:?}", &self.#field_name))
                ));
            }
        }
    }
}

fn generate_to_wrapped_field(field: &FieldOptionality) -> TokenStream {
    let field_name = &field.field_name;

    match field_storage(field) {
        FieldStorage::OriginallyOptional => optional_value_to_wrapped(field),
        FieldStorage::Nested if field.was_optional => optional_value_to_wrapped(field),
        FieldStorage::WrappedOption => {
            if let Some(default_expr) = &field.default_expr {
                let default_original = default_expr_for_original(default_expr);
                let original_type = &field.original_type;
                if needs_from_conversion(field) {
                    let converted = apply_from_conversion(field, quote! { value });
                    quote! {
                        #field_name: {
                            let value = from.#field_name;
                            let default_original: #original_type = #default_original;
                            if value == default_original {
                                None
                            } else {
                                Some(#converted)
                            }
                        }
                    }
                } else {
                    quote! {
                        #field_name: {
                            let value = from.#field_name;
                            let default_original: #original_type = #default_original;
                            if value == default_original {
                                None
                            } else {
                                Some(value)
                            }
                        }
                    }
                }
            } else if needs_from_conversion(field) {
                let converted = apply_from_conversion(field, quote! { value });
                quote! {
                    #field_name: {
                        let value = from.#field_name;
                        Some(#converted)
                    }
                }
            } else {
                quote! {
                    #field_name: Some(from.#field_name)
                }
            }
        },
        FieldStorage::Plain | FieldStorage::Nested | FieldStorage::List => {
            let converted = apply_from_conversion(field, quote! { from.#field_name });
            quote! {
                #field_name: #converted
            }
        },
    }
}

fn generate_from_wrapped_field(field: &FieldOptionality) -> TokenStream {
    let field_name = &field.field_name;

    match field_storage(field) {
        FieldStorage::OriginallyOptional => optional_value_from_wrapped(field),
        FieldStorage::Nested if field.was_optional => optional_value_from_wrapped(field),
        FieldStorage::WrappedOption => {
            if let Some(default_expr) = &field.default_expr {
                let default_original = default_expr_for_original(default_expr);
                if needs_into_conversion(field) {
                    let converted = apply_into_conversion(field, quote! { value });
                    quote! {
                        #field_name: from.#field_name
                            .map(|value| #converted)
                            .unwrap_or(#default_original)
                    }
                } else {
                    quote! {
                        #field_name: from.#field_name.unwrap_or(#default_original)
                    }
                }
            } else if needs_into_conversion(field) {
                let converted = apply_into_conversion(field, quote! { value });
                quote! {
                    #field_name: from.#field_name
                        .map(|value| #converted)
                        .unwrap_or_default()
                }
            } else {
                quote! {
                    #field_name: from.#field_name.unwrap_or_default()
                }
            }
        },
        FieldStorage::Plain | FieldStorage::Nested | FieldStorage::List => {
            let converted = apply_into_conversion(field, quote! { from.#field_name });
            quote! {
                #field_name: #converted
            }
        },
    }
}

fn generate_present_fields_json_entry(field: &FieldOptionality) -> TokenStream {
    let field_name = &field.field_name;
    let field_name_str = field_name.to_string();

    match field_storage(field) {
        FieldStorage::OriginallyOptional => optional_value_present_json(field),
        FieldStorage::Nested if field.was_optional => optional_value_present_json(field),
        FieldStorage::WrappedOption => {
            if needs_into_conversion(field) {
                let converted = apply_into_conversion(field, quote! { value });
                quote! {
                    if let Some(value) = self.#field_name.clone() {
                        let converted = #converted;
                        entries.push(format!(
                            "\"{}\":\"{}\"",
                            #field_name_str,
                            Self::escape_json_string(&format!("{:?}", converted))
                        ));
                    }
                }
            } else {
                quote! {
                    if let Some(value) = self.#field_name.as_ref() {
                        entries.push(format!(
                            "\"{}\":\"{}\"",
                            #field_name_str,
                            Self::escape_json_string(&format!("{:?}", value))
                        ));
                    }
                }
            }
        },
        FieldStorage::Plain | FieldStorage::Nested | FieldStorage::List => {
            if needs_into_conversion(field) && field.composition.is_flat() {
                let converted = apply_into_conversion(field, quote! { value });
                quote! {
                    let value = self.#field_name.clone();
                    let converted = #converted;
                    entries.push(format!(
                        "\"{}\":\"{}\"",
                        #field_name_str,
                        Self::escape_json_string(&format!("{:?}", converted))
                    ));
                }
            } else {
                quote! {
                    entries.push(format!(
                        "\"{}\":\"{}\"",
                        #field_name_str,
                        Self::escape_json_string(&format!("{:?}", &self.#field_name))
                    ));
                }
            }
        },
    }
}

pub fn parse_field_default(field: &ComponentField) -> Option<syn::Expr> {
    field.default.as_ref().map(|expr| expr.0.clone())
}

/// Generates the `<Name>FormValueHolder` struct and impls. Returns the emitted
/// tokens, the fields given a synthetic `RequiredValidation`, and
/// `needs_koruma_derive` — the gate downstream emitters must key on.
pub fn generate_value_holder(
    original_input: &DeriveInput,
    fields: &[FieldOptionality],
    enable_koruma: bool,
    enable_koruma_fluent: bool,
) -> (TokenStream, Vec<String>, bool) {
    let has_skipped_fields = fields.iter().any(|f| f.skip);
    let fields_requiring_required: Vec<String> = fields
        .iter()
        .filter(|f| f.needs_required_validation())
        .map(|f| f.field_name.to_string())
        .collect();

    let mut field_attrs: HashMap<String, Vec<TokenStream>> = HashMap::new();

    for f in fields {
        if f.skip {
            continue;
        }
        let field_name = f.field_name.to_string();

        if enable_koruma {
            let needs_required = f.needs_required_validation();

            let has_existing_validations = !f.validation.field_validators.is_empty()
                || !f.validation.element_validators.is_empty();
            let has_newtype = f.validation.is_newtype;
            // Keyed on composition, not `validation.is_nested`: koruma's per-field
            // error getter exists only when the holder field carries `#[koruma(nested)]`.
            let has_nested = f.composition.is_nested();
            let has_required_if = f.required_if.is_some();

            if needs_required
                || has_existing_validations
                || has_newtype
                || has_nested
                || has_required_if
            {
                let mut koruma_items: Vec<TokenStream> = Vec::new();

                if needs_required {
                    koruma_items.push(quote! {
                        koruma_collection::general::RequiredValidation::<Option<_>>::builder()
                    });
                }

                if let Some(rule) = &f.required_if {
                    let validator = &rule.validator;
                    let when = rule.condition.when_argument_tokens(&rule.sibling);
                    koruma_items.push(quote! {
                        #validator::<Option<_>>::builder().when(#when)
                    });
                }

                let existing_validations: Vec<TokenStream> = f
                    .validation
                    .field_validators
                    .iter()
                    .map(validator_attr_to_tokens)
                    .collect();
                koruma_items.extend(existing_validations);

                // On list fields `each(..)` keeps element scoping so koruma
                // reports failures per item index; flat fields mirror whole-value.
                let element_validations: Vec<TokenStream> = f
                    .validation
                    .element_validators
                    .iter()
                    .map(|validator| {
                        let tokens = validator_attr_to_tokens(validator);
                        if f.composition.is_list() {
                            quote! { each(#tokens) }
                        } else {
                            tokens
                        }
                    })
                    .collect();
                koruma_items.extend(element_validations);

                let mut attrs: Vec<TokenStream> = Vec::new();

                if !koruma_items.is_empty() {
                    attrs.push(quote! { #[koruma(#(#koruma_items),*)] });
                }

                // newtype/nested must be separate attrs when combined with
                // other validators, to avoid koruma type-resolution issues.
                if f.validation.is_newtype {
                    attrs.insert(0, quote! { #[koruma(newtype)] });
                }
                if f.validation.is_nested {
                    attrs.insert(0, quote! { #[koruma(nested)] });
                }

                if !attrs.is_empty() {
                    field_attrs.insert(field_name, attrs);
                }
            }
        }
    }

    let needs_koruma_derive = enable_koruma;

    let required_if_validator_ident = fields
        .iter()
        .filter(|f| !f.skip)
        .find_map(|f| f.required_if.as_ref().map(|rule| rule.validator.clone()));
    let required_if_validator_tokens = match (&required_if_validator_ident, needs_koruma_derive) {
        (Some(validator), true) => generate_required_if_validator(validator),
        _ => quote! {},
    };

    let has_custom_defaults = fields.iter().any(|f| !f.skip && f.default_expr.is_some());

    let mut derives: Vec<TokenStream> = vec![quote! { Clone }, quote! { Debug }];
    if !has_custom_defaults {
        derives.push(quote! { Default });
    }
    if has_skipped_fields {
        derives.push(quote! { ::gpui_form::bon::Builder });
    }
    if needs_koruma_derive {
        derives.push(quote! { ::koruma::Koruma });
        if enable_koruma_fluent {
            derives.push(quote! { ::koruma::KorumaAllDisplay });
        }
    }
    // Unconditional `PartialEq` (never `Eq`): `f64` fields are non-`Eq`, and
    // core's `is_dirty`/`diff_against` require `H: PartialEq`.
    derives.push(quote! { ::core::cmp::PartialEq });
    #[cfg(feature = "serde")]
    {
        derives.push(quote! { ::serde::Serialize });
        derives.push(quote! { ::serde::Deserialize });
    }

    let derive_output = quote! {
        #[allow(clippy::derive_partial_eq_without_eq)]
        #[derive(#(#derives),*)]
    };
    let builder_attr = if has_skipped_fields {
        quote! { #[builder(crate = ::gpui_form::bon)] }
    } else {
        quote! {}
    };

    let original_ident = &original_input.ident;
    let wrapped_ident = format_ident!("{}FormValueHolder", original_ident);
    let conversion_error_ident = format_ident!("{}ConversionError", wrapped_ident);
    let conversion_error_type = generate_conversion_error_type(&conversion_error_ident);
    let (impl_generics, ty_generics, where_clause) = original_input.generics.split_for_impl();

    let field_definitions: Vec<TokenStream> = fields
        .iter()
        .filter(|f| !f.skip)
        .map(|f| {
            let field_name = &f.field_name;
            let field_type = form_field_type_tokens(f);
            let attrs = field_attrs
                .get(&field_name.to_string())
                .cloned()
                .unwrap_or_default();
            quote! {
                #(#attrs)* pub #field_name: #field_type
            }
        })
        .collect();

    let list_mutation_methods: Vec<TokenStream> = fields
        .iter()
        .filter(|f| !f.skip)
        .filter_map(|f| match &f.composition {
            FieldComposition::List { item } => {
                let field_name = &f.field_name;
                let add_name = format_ident!("add_{}", field_name);
                let remove_name = format_ident!("remove_{}", field_name);
                let items_name = format_ident!("{}_items", field_name);
                if f.was_optional {
                    Some(quote! {
                        /// Append one item, creating the list if the field was `None`.
                        pub fn #add_name(&mut self, value: #item) {
                            self.#field_name.get_or_insert_with(::std::vec::Vec::new).push(value);
                        }

                        /// Remove the item at `index` (same semantics as `Vec::remove`; a no-op when the field is `None`).
                        pub fn #remove_name(&mut self, index: usize) {
                            if let ::core::option::Option::Some(items) = self.#field_name.as_mut() {
                                items.remove(index);
                            }
                        }

                        /// Borrow the current items; empty when the field is `None`.
                        pub fn #items_name(&self) -> &[#item] {
                            self.#field_name.as_deref().unwrap_or(&[])
                        }
                    })
                } else {
                    Some(quote! {
                        /// Append one item.
                        pub fn #add_name(&mut self, value: #item) {
                            self.#field_name.push(value);
                        }

                        /// Remove the item at `index` (same semantics as
                        /// `Vec::remove`).
                        pub fn #remove_name(&mut self, index: usize) {
                            self.#field_name.remove(index);
                        }

                        /// Borrow the current items.
                        pub fn #items_name(&self) -> &[#item] {
                            &self.#field_name
                        }
                    })
                }
            },
            _ => None,
        })
        .collect();
    let list_mutation_impl = if list_mutation_methods.is_empty() {
        quote! {}
    } else {
        quote! {
            impl #impl_generics #wrapped_ident #ty_generics #where_clause {
                #(#list_mutation_methods)*
            }
        }
    };

    let to_wrapped_fields: Vec<TokenStream> = fields
        .iter()
        .filter(|f| !f.skip)
        .map(generate_to_wrapped_field)
        .collect();

    let from_wrapped_fields: Vec<TokenStream> = fields
        .iter()
        .filter(|f| !f.skip)
        .map(generate_from_wrapped_field)
        .collect();

    let try_from_fields: Vec<TokenStream> = fields
        .iter()
        .filter(|f| !f.skip)
        .map(|f| try_from_field_tokens(f, quote! { from }, &conversion_error_ident))
        .collect();

    let present_fields_json_entries: Vec<TokenStream> = fields
        .iter()
        .filter(|f| !f.skip)
        .map(generate_present_fields_json_entry)
        .collect();

    let mut from_where_clause = where_clause.cloned();
    let mut new_predicates: Vec<syn::WherePredicate> = Vec::new();
    for f in fields {
        if !f.skip && !f.was_optional && f.wrap_in_option {
            let original_type = &f.original_type;
            new_predicates.push(syn::parse_quote!(#original_type: ::core::default::Default));
        }
    }
    if !new_predicates.is_empty() {
        let wc = from_where_clause.get_or_insert_with(|| syn::parse_quote!(where));
        wc.predicates.extend(new_predicates);
    }

    let skipped_params: Vec<TokenStream> = fields
        .iter()
        .filter(|f| f.skip)
        .map(|f| {
            let field_name = &f.field_name;
            let ty = &f.original_type;
            quote! { #field_name: #ty }
        })
        .collect();

    let into_original_fields: Vec<TokenStream> = fields
        .iter()
        .map(|f| {
            if f.skip {
                let field_name = &f.field_name;
                quote! { #field_name }
            } else {
                try_from_field_tokens(f, quote! { self }, &conversion_error_ident)
            }
        })
        .collect();

    let skipped_fields_impl = if has_skipped_fields {
        quote! {
            impl #impl_generics #wrapped_ident #ty_generics #where_clause {
                pub fn present_fields_json(&self) -> String {
                    let mut entries: Vec<String> = Vec::new();
                    #(#present_fields_json_entries)*
                    format!("{{{}}}", entries.join(","))
                }

                fn escape_json_string(input: &str) -> String {
                    let mut escaped = String::new();
                    for ch in input.chars() {
                        match ch {
                            '"' => escaped.push_str("\\\""),
                            '\\' => escaped.push_str("\\\\"),
                            '\n' => escaped.push_str("\\n"),
                            '\r' => escaped.push_str("\\r"),
                            '\t' => escaped.push_str("\\t"),
                            '\u{08}' => escaped.push_str("\\b"),
                            '\u{0C}' => escaped.push_str("\\f"),
                            c if c.is_control() => {
                                use ::core::fmt::Write as _;
                                let _ = write!(&mut escaped, "\\u{:04x}", c as u32);
                            },
                            c => escaped.push(c),
                        }
                    }
                    escaped
                }

                pub fn into_original(
                    self,
                    #(#skipped_params),*
                ) -> Result<#original_ident #ty_generics, #conversion_error_ident> {
                    Ok(#original_ident {
                        #(#into_original_fields),*
                    })
                }
            }
        }
    } else {
        quote! {
            impl #impl_generics ::core::convert::From<#wrapped_ident #ty_generics>
                for #original_ident #ty_generics #from_where_clause
            {
                fn from(from: #wrapped_ident #ty_generics) -> Self {
                    Self {
                        #(#from_wrapped_fields),*
                    }
                }
            }

            impl #impl_generics #wrapped_ident #ty_generics #where_clause {
                pub fn try_from(
                    from: #wrapped_ident #ty_generics
                ) -> Result<#original_ident #ty_generics, #conversion_error_ident> {
                    Ok(#original_ident {
                        #(#try_from_fields),*
                    })
                }
            }
        }
    };

    let i18n_impl = if enable_koruma_fluent {
        generate_koruma_i18n_impl(original_input, fields)
    } else {
        quote! {}
    };

    // Same gate as the holder's `#[derive(Koruma)]`, so every koruma error
    // type the emitter references is guaranteed to exist.
    let field_errors_impl = if needs_koruma_derive {
        crate::derives::gpui_form::errors::generate_field_errors(original_input, fields)
    } else {
        quote! {}
    };

    let mut tokens = quote! {
        #conversion_error_type
        #required_if_validator_tokens
        #derive_output
        #builder_attr
        pub struct #wrapped_ident #ty_generics #where_clause {
            #(#field_definitions),*
        }

        impl #impl_generics ::core::convert::From<#original_ident #ty_generics>
            for #wrapped_ident #ty_generics #where_clause
        {
            fn from(from: #original_ident #ty_generics) -> Self {
                Self {
                    #(#to_wrapped_fields),*
                }
            }
        }

        #skipped_fields_impl

        #list_mutation_impl

        #i18n_impl

        #field_errors_impl
    };

    if has_custom_defaults {
        let default_impl = generate_default_impl(fields, &wrapped_ident);
        tokens = quote! {
            #tokens
            #default_impl
        };
    }

    (tokens, fields_requiring_required, needs_koruma_derive)
}

pub fn validation_issue_key_for_kind(kind: &str) -> String {
    // The per-form `<Name>FormRequiredIfValidation` failure IS a required
    // failure; reuse the already-localized `validation.required` key.
    if kind.ends_with("FormRequiredIfValidation") {
        return "validation.required".to_string();
    }
    let stem = kind.strip_suffix("Validation").unwrap_or(kind);
    format!("validation.{}", stem.to_snake_case())
}

fn generate_required_if_validator(validator: &syn::Ident) -> TokenStream {
    quote! {
        /// Conditional-required validator desugared from `#[gpui_form(required_if = <field>)]`.
        #[doc(hidden)]
        #[derive(Clone, Debug)]
        #[::koruma::validator]
        pub struct #validator<T> {
            /// Whether the controlling sibling field makes the value required (captured per validation run).
            when: bool,
            /// Presence-only slot; never captured — the validated value arrives as `validate`'s argument.
            #[koruma(value, skip_capture)]
            actual: ::core::option::Option<T>,
        }

        impl<T> ::koruma::Validate<::core::option::Option<T>>
            for #validator<::core::option::Option<T>>
        {
            fn validate(&self, value: &::core::option::Option<T>) -> bool {
                ::gpui_form::core::required_when(self.when, value.is_some())
            }
        }

        impl<T> ::core::fmt::Display for #validator<T> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.write_str("This field is required.")
            }
        }
    }
}

fn generate_koruma_i18n_impl(
    original_input: &DeriveInput,
    fields: &[FieldOptionality],
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = original_input.generics.split_for_impl();
    let wrapped_ident = format_ident!("{}FormValueHolder", original_input.ident);
    let form_key = original_input.ident.to_string().to_snake_case();
    let prefix_const = format_ident!(
        "{}_I18N_KEY_PREFIX",
        original_input.ident.to_string().to_shouty_snake_case()
    );

    let label_consts = fields.iter().filter(|f| !f.skip).map(|f| {
        let field = f.field_name.to_string();
        let const_ident = format_ident!("{}_LABEL_KEY", field.to_shouty_snake_case());
        let key = format!("{form_key}.{field}_label");
        quote! { pub const #const_ident: &'static str = #key; }
    });

    let mut kinds: BTreeSet<String> = fields
        .iter()
        .filter(|f| !f.skip)
        .flat_map(|f| {
            f.validation
                .field_validators
                .iter()
                .chain(f.validation.element_validators.iter())
                .map(|v| v.name().to_string())
        })
        .collect();
    if fields.iter().any(|f| f.needs_required_validation()) {
        kinds.insert("RequiredValidation".to_string());
    }
    for f in fields.iter().filter(|f| !f.skip) {
        if let Some(rule) = &f.required_if {
            kinds.insert(rule.validator.to_string());
        }
    }

    let kind_arms = kinds.iter().map(|kind| {
        let key = validation_issue_key_for_kind(kind);
        quote! { #kind => #key, }
    });

    quote! {
        impl #impl_generics #wrapped_ident #ty_generics #where_clause {
            pub const #prefix_const: &'static str = #form_key;
            #(#label_consts)*

            pub fn validation_issue_key(kind: &str) -> &'static str {
                match kind {
                    #(#kind_arms)*
                    _ => "validation.invalid",
                }
            }

            pub fn localized_validation_issue(kind: &str) -> ::std::borrow::Cow<'static, str> {
                let key = Self::validation_issue_key(kind);
                ::rust_i18n::t!(key)
            }
        }
    }
}
