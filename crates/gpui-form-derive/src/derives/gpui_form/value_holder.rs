use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashMap;
use syn::DeriveInput;
use unwrapped_core::{WrappedOpts, WrappedProcUsageOpts};

use crate::derives::gpui_form::koruma::validator_attr_to_tokens;
use crate::derives::gpui_form::structs::{ComponentField, FieldOptionality};

/// Generates a custom Default implementation for the FormValueHolder that uses
/// the specified default expressions for fields that have them.
fn generate_default_impl(
    _original_input: &DeriveInput,
    fields: &[FieldOptionality],
    struct_name: &syn::Ident,
) -> TokenStream {
    let default_fields: Vec<TokenStream> = fields
        .iter()
        .map(|f| {
            let field_name = &f.field_name;
            if let Some(default_expr) = &f.default_expr {
                // Field has a custom default - wrap it in Some()
                quote! {
                    #field_name: Some(#default_expr)
                }
            } else if f.wrap_in_option {
                // Field is wrapped in Option but has no custom default
                quote! {
                    #field_name: None
                }
            } else {
                // Field is not wrapped, use standard Default
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

    // Check if any field has a custom default expression
    let has_custom_defaults = fields.iter().any(|f| f.default_expr.is_some());

    let mut wrapped_options = WrappedOpts::builder()
        .suffix(format_ident!("FormValueHolder"))
        .build();

    // Only derive Default automatically if there are no custom defaults
    // If there are custom defaults, we'll generate a custom Default impl
    if !has_custom_defaults {
        wrapped_options = wrapped_options.with_derive(quote! { Clone, Debug, Default });
    } else {
        wrapped_options = wrapped_options.with_derive(quote! { Clone, Debug });
    }
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

    let mut macro_options =
        WrappedProcUsageOpts::new(fields_to_wrap, Some(format_ident!("gpui_form")));

    // Add default expressions to field options
    for field in fields {
        if let Some(default_expr) = &field.default_expr {
            macro_options = macro_options.with_field_opts(
                &field.field_name.to_string(),
                unwrapped_core::wrapped::FieldProcOpts::new(field.wrap_in_option)
                    .with_default(default_expr.clone()),
            );
        }
    }

    let mut tokens = unwrapped_core::wrapped(original_input, Some(wrapped_options), macro_options);

    // If there are custom defaults, generate a custom Default implementation
    if has_custom_defaults {
        // Get the original struct name and create the wrapped struct name
        let original_ident = &original_input.ident;
        let wrapped_ident = format_ident!("{}FormValueHolder", original_ident);

        let default_impl = generate_default_impl(original_input, fields, &wrapped_ident);
        tokens = quote! {
            #tokens
            #default_impl
        };
    }

    (tokens, fields_requiring_required)
}
