use super::*;
use crate::derives::gpui_form::structs::FieldOptionality;
use crate::derives::gpui_form::utils::extract_option_inner_type;
use quote::ToTokens as _;

pub(super) struct FieldDescriptorTokens {
    pub(super) constants: Vec<TokenStream>,
    pub(super) descriptor: TokenStream,
}

enum FieldStoragePlan {
    OriginallyOptional,
    WrappedOption,
    Direct,
}

fn field_storage(field: &FieldOptionality) -> FieldStoragePlan {
    if field.was_optional {
        FieldStoragePlan::OriginallyOptional
    } else if field.wrap_in_option {
        FieldStoragePlan::WrappedOption
    } else {
        FieldStoragePlan::Direct
    }
}

fn form_base_type(field: &FieldOptionality) -> syn::Type {
    if let Some(override_type) = &field.override_type {
        extract_option_inner_type(override_type).1
    } else {
        field.inner_type.clone()
    }
}

fn field_value_presence_tokens(facade_crate: &Path, field: &FieldOptionality) -> TokenStream {
    match field_storage(field) {
        FieldStoragePlan::OriginallyOptional => {
            quote! {
                #facade_crate::mcp::FieldValuePresence::Optional
            }
        },
        FieldStoragePlan::WrappedOption | FieldStoragePlan::Direct => {
            quote! {
                #facade_crate::mcp::FieldValuePresence::DirectStorage
            }
        },
    }
}

pub(super) fn field_decode_tokens(_facade_crate: &Path, field: &FieldOptionality) -> TokenStream {
    let field_name = &field.field_name;
    let field_name_str = field.field_name.to_string();
    let base_type = form_base_type(field);
    let required = !matches!(field_storage(field), FieldStoragePlan::OriginallyOptional)
        && field.default_expr.is_none();

    match field_storage(field) {
        FieldStoragePlan::OriginallyOptional => {
            quote! {
                if let Some(__gpui_form_decoded) =
                    __gpui_form_arguments
                        .take_present_tool_value::<Option<#base_type>>(#field_name_str)?
                {
                    __gpui_form_holder.#field_name = __gpui_form_decoded;
                }
            }
        },
        FieldStoragePlan::WrappedOption => {
            if required {
                quote! {
                    __gpui_form_holder.#field_name =
                        ::std::option::Option::Some(
                            __gpui_form_arguments
                                .take_required_tool_value::<#base_type>(#field_name_str)?
                        );
                }
            } else {
                quote! {
                    if let Some(__gpui_form_decoded) =
                        __gpui_form_arguments
                            .take_present_tool_value::<#base_type>(#field_name_str)?
                    {
                        __gpui_form_holder.#field_name = ::std::option::Option::Some(
                            __gpui_form_decoded,
                        );
                    }
                }
            }
        },
        FieldStoragePlan::Direct => {
            if required {
                quote! {
                    __gpui_form_holder.#field_name =
                        __gpui_form_arguments
                            .take_required_tool_value::<#base_type>(#field_name_str)?;
                }
            } else {
                quote! {
                    if let Some(__gpui_form_decoded) =
                        __gpui_form_arguments
                            .take_present_tool_value::<#base_type>(#field_name_str)?
                    {
                        __gpui_form_holder.#field_name = __gpui_form_decoded;
                    }
                }
            }
        },
    }
}

pub(super) fn field_decode_arm_tokens(facade_crate: &Path, field: &FieldOptionality) -> TokenStream {
    let field_name_str = field.field_name.to_string();
    let field_decode = field_decode_tokens(facade_crate, field);

    quote! {
        #field_name_str => {
            let mut __gpui_form_values = #facade_crate::mcp::serde_json::Map::new();
            __gpui_form_values.insert(field.to_string(), value.into_value());
            let mut __gpui_form_arguments =
                #facade_crate::mcp::McpArguments::new(__gpui_form_values);
            let __gpui_form_holder = holder;
            #field_decode
            __gpui_form_arguments.finish()?;
            Ok(())
        }
    }
}

pub(super) fn field_descriptor_tokens(
    facade_crate: &Path,
    form_ident: &syn::Ident,
    field_index: usize,
    field: &FieldOptionality,
    enable_koruma: bool,
) -> Option<FieldDescriptorTokens> {
    let field_name_str = field.field_name.to_string();
    let base_type = form_base_type(field);
    let value_type_tokens = rust_type_tokens(facade_crate, &base_type);
    let value_presence_tokens = field_value_presence_tokens(facade_crate, field);
    let has_default = field.default_expr.is_some();
    let field_descriptor_constructor = quote! {
        #facade_crate::mcp::McpField::typed::<#base_type>(
            #field_name_str,
            #value_type_tokens,
            #value_presence_tokens
        )
    };
    let label_tokens = field.label.as_ref().map(|label| {
        let label = LitStr::new(label, field.field_name.span());
        quote! { .with_label(#label) }
    });
    let description_tokens = field.description.as_ref().map(|description| {
        let description = LitStr::new(description, field.field_name.span());
        quote! { .with_description(#description) }
    });
    let (validation_rule_consts, validation_rules_tokens) =
        field_validation_rules_tokens(facade_crate, form_ident, field_index, field, enable_koruma);

    let descriptor = quote! {
        #field_descriptor_constructor
        .with_default(#has_default)
        #label_tokens
        #description_tokens
        #validation_rules_tokens
    };

    Some(FieldDescriptorTokens {
        constants: validation_rule_consts,
        descriptor,
    })
}

fn rust_type_tokens(facade_crate: &Path, ty: &syn::Type) -> TokenStream {
    let rendered = ty.to_token_stream().to_string();
    quote! { #facade_crate::mcp::RustType::from_macro_tokens_unchecked(#rendered) }
}
