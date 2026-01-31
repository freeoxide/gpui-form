use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashMap;
use syn::DeriveInput;
use unwrapped_core::{WrappedOpts, WrappedProcUsageOpts};

use crate::derives::gpui_form::koruma::validator_attr_to_tokens;
use crate::derives::gpui_form::structs::{ComponentField, FieldOptionality};

pub fn parse_field_default(field: &ComponentField) -> Option<TokenStream> {
    field.default.as_ref().map(|expr| quote! { #expr })
}

/// Generates the FormValueHolder struct and its implementations using unwrapped.
pub fn generate_value_holder(
    original_input: &DeriveInput,
    fields: &[FieldOptionality],
    enable_koruma: bool,
    enable_koruma_fluent: bool,
) -> (TokenStream, Vec<String>) {
    let fields_requiring_required: Vec<String> = fields
        .iter()
        .filter(|f| {
            f.wrap_in_option
                && !f.was_optional
                && !f.validation.is_newtype
                && !f.validation.is_nested
        })
        .map(|f| f.field_name.to_string())
        .collect();

    let has_any_koruma = fields.iter().any(|f| {
        !f.validation.field_validators.is_empty()
            || !f.validation.element_validators.is_empty()
            || f.validation.is_nested
            || f.validation.is_newtype
    });

    let has_any_required = fields.iter().any(|f| {
        f.wrap_in_option && !f.was_optional && !f.validation.is_newtype && !f.validation.is_nested
    });

    let mut fields_to_wrap: HashMap<String, bool> = HashMap::new();
    let mut field_attrs: HashMap<String, Vec<TokenStream>> = HashMap::new();

    for f in fields {
        let field_name = f.field_name.to_string();
        fields_to_wrap.insert(field_name.clone(), f.wrap_in_option);

        if enable_koruma {
            let needs_required = f.wrap_in_option
                && !f.was_optional
                && !f.validation.is_newtype
                && !f.validation.is_nested;

            let has_existing_validations = !f.validation.field_validators.is_empty()
                || !f.validation.element_validators.is_empty();
            let has_newtype = f.validation.is_newtype;

            if needs_required || has_existing_validations || has_newtype {
                let mut attrs: Vec<TokenStream> = Vec::new();

                if f.validation.is_newtype {
                    attrs.push(quote! { #[koruma(newtype)] });
                }
                if f.validation.is_nested {
                    attrs.push(quote! { #[koruma(nested)] });
                }

                let mut koruma_items: Vec<TokenStream> = Vec::new();

                if needs_required {
                    koruma_items.push(
                        quote! { koruma_collection::general::RequiredValidation::<Option<_>> },
                    );
                }

                let existing_validations: Vec<TokenStream> = f
                    .validation
                    .field_validators
                    .iter()
                    .chain(f.validation.element_validators.iter())
                    .map(validator_attr_to_tokens)
                    .collect();
                koruma_items.extend(existing_validations);

                if !koruma_items.is_empty() {
                    attrs.push(quote! { #[koruma(#(#koruma_items),*)] });
                }

                if !attrs.is_empty() {
                    field_attrs.insert(field_name, attrs);
                }
            }
        }
    }

    let needs_koruma_for_required = has_any_required && enable_koruma;
    let needs_koruma_derive = (enable_koruma && has_any_koruma) || needs_koruma_for_required;

    let mut wrapped_options = WrappedOpts::builder()
        .suffix(format_ident!("FormValueHolder"))
        .build();

    wrapped_options = wrapped_options.with_derive(quote! { Clone, Debug, Default });
    if needs_koruma_derive {
        if enable_koruma_fluent {
            wrapped_options =
                wrapped_options.with_derive(quote! { ::koruma::Koruma, ::koruma::KorumaAllFluent });
        } else {
            wrapped_options = wrapped_options.with_derive(quote! { ::koruma::Koruma });
        }
    }

    for (field_name, attrs) in field_attrs {
        for attr in attrs {
            wrapped_options = wrapped_options.with_field_attr(&field_name, attr);
        }
    }

    let macro_options = WrappedProcUsageOpts::new(fields_to_wrap, Some(format_ident!("gpui_form")));

    let tokens = unwrapped_core::wrapped(original_input, Some(wrapped_options), macro_options);

    (tokens, fields_requiring_required)
}
