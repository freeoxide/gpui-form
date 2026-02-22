use darling::FromDeriveInput as _;
use itertools::Itertools as _;
use koruma_derive_core::{FieldInfo as KorumaFieldInfo, ParseFieldResult};
use proc_macro2::TokenStream;
use quote::{ToTokens as _, format_ident, quote};
use std::collections::HashMap;
use syn::DeriveInput;

use crate::derives::gpui_form::cfg_attr::flatten_cfg_attr_in_derive_input;
use crate::derives::gpui_form::components::{
    generate_component_field, get_components_behaviour_tokens,
};
use crate::derives::gpui_form::structs::{ComponentStruct, FieldOptionality, GpuiFormOptions};
use crate::derives::gpui_form::utils::extract_option_inner_type;
use crate::derives::gpui_form::value_holder::{generate_value_holder, parse_field_default};
use gpui_form_core::components::Components;

pub fn expand_gpui_form(
    derive_input: DeriveInput,
    options: GpuiFormOptions,
) -> proc_macro2::TokenStream {
    let original_input = derive_input.clone();
    let derive_input = flatten_cfg_attr_in_derive_input(derive_input);

    let parsed = match ComponentStruct::from_derive_input(&derive_input) {
        Ok(parsed) => parsed,
        Err(e) => return e.write_errors(),
    };

    let struct_name = &parsed.ident;
    let components_holder_name = format_ident!("{}FormFields", struct_name);
    let components_base_declarations_name = format_ident!("{}FormComponents", struct_name);
    let items_errors_struct_name = format_ident!("{}FormItemsErrors", struct_name);

    let koruma_options = parsed.koruma.as_ref().map(|k| k.0.clone());

    if parsed.empty {
        let enable_koruma = koruma_options.is_some();
        let enable_koruma_fluent = koruma_options.as_ref().map(|k| k.fluent).unwrap_or(false);
        let empty_fields: Vec<FieldOptionality> = Vec::new();
        let (value_holder_tokens, _) = generate_value_holder(
            &original_input,
            &empty_fields,
            enable_koruma,
            enable_koruma_fluent,
        );
        let shape_impl = if options.generate_shape {
            quote! {
                ::gpui_form::core::registry::inventory::submit! {
                    ::gpui_form::core::registry::GpuiFormShape::new(
                        stringify!(#struct_name),
                        &[],
                        file!(),
                        #enable_koruma
                    )
                }
            }
        } else {
            quote! {}
        };

        return quote! {
            #value_holder_tokens
            pub struct #components_holder_name;

            #shape_impl

            pub struct #components_base_declarations_name;
        };
    }

    let fields_iter = match &parsed.data {
        darling::ast::Data::Struct(s) => &s.fields,
        _ => unreachable!("GpuiForm derive only supports named structs"),
    };

    let has_skipped_fields = fields_iter.iter().any(|field| field.skip());
    if has_skipped_fields
        && let Some(from_field) = fields_iter
            .iter()
            .find(|field| !field.skip() && field.from.is_some())
    {
        let from_name = from_field
            .ident
            .as_ref()
            .map(|ident| ident.to_string())
            .unwrap_or_else(|| "<unknown>".to_string());
        let skip_name = fields_iter
            .iter()
            .find(|field| field.skip())
            .and_then(|field| field.ident.as_ref())
            .map(|ident| ident.to_string())
            .unwrap_or_else(|| "<unknown>".to_string());
        return syn::Error::new_spanned(
            from_field.from.as_ref().expect("checked from above"),
            format!(
                "field `{}` uses `from = ...`, but `#[gpui_form(skip)]` on `{}` disables \
                     generating `From<Original>` for the form value holder, so `from` conversions \
                     are ignored. Remove `skip` items or remove this `from`",
                from_name, skip_name
            ),
        )
        .to_compile_error();
    }

    let parsed_koruma_fields: HashMap<String, KorumaFieldInfo> = match &derive_input.data {
        syn::Data::Struct(data_struct) => data_struct
            .fields
            .iter()
            .filter_map(|field| {
                let ident = field.ident.as_ref()?.to_string();
                match koruma_derive_core::parse_field(field, 0) {
                    ParseFieldResult::Valid(info) => Some((ident, *info)),
                    ParseFieldResult::Skip | ParseFieldResult::Error(_) => None,
                }
            })
            .collect(),
        _ => HashMap::new(),
    };

    let enable_koruma = koruma_options.is_some() || !parsed_koruma_fields.is_empty();
    let enable_koruma_fluent = koruma_options.as_ref().map(|k| k.fluent).unwrap_or(false);

    let koruma_validations: HashMap<String, Vec<String>> = parsed_koruma_fields
        .iter()
        .map(|(name, info)| {
            let mut validator_names: Vec<String> = info
                .validation
                .field_validators
                .iter()
                .map(|v| v.name().to_string())
                .collect();

            if info.is_newtype() {
                validator_names.push("NewtypeValidation".to_string());
            }

            if info.is_nested() {
                validator_names.push("NestedValidation".to_string());
            }

            (name.clone(), validator_names)
        })
        .collect();

    if fields_iter.is_empty() {
        return syn::Error::new_spanned(
            &derive_input,
            format!(
                "Struct `{}` has no fields. Add `#[gpui_form(empty)]` attribute to explicitly mark it as an empty form.",
                struct_name
            ),
        )
        .to_compile_error();
    }

    let component_field_pairs: Vec<crate::derives::gpui_form::structs::ComponentFieldContent> =
        fields_iter
            .iter()
            .filter(|field| !field.skip())
            .map(generate_component_field)
            .collect();

    let (field_structure_tokens, field_base_declarations_tokens, wrap_in_option_map): (
        Vec<TokenStream>,
        Vec<TokenStream>,
        HashMap<String, bool>,
    ) = component_field_pairs
        .into_iter()
        .map(|content| {
            (
                content.field_structure_tokens,
                content.field_base_declarations_tokens,
                content.wrap_in_option,
            )
        })
        .multiunzip();

    let mut field_optionality = Vec::new();
    for field in fields_iter {
        let field_name = field.ident.clone().unwrap();
        let field_name_str = field_name.to_string();
        let (was_optional, inner_type) = extract_option_inner_type(&field.ty);
        let wrap_in_option = !field.skip()
            && field.component.is_some()
            && wrap_in_option_map
                .get(&field_name_str)
                .copied()
                .unwrap_or(false);
        let koruma_info = parsed_koruma_fields.get(&field_name_str);
        let validation = koruma_info
            .map(|info| info.validation.clone())
            .unwrap_or_default();
        let default_expr = parse_field_default(field);
        field_optionality.push(FieldOptionality {
            field_name,
            original_type: field.ty.clone(),
            inner_type,
            was_optional,
            wrap_in_option,
            validation,
            default_expr,
            override_type: field.r#type.as_ref().map(|ty| ty.0.clone()),
            into_expr: field.into.clone(),
            from_expr: field.from.clone(),
            skip: field.skip(),
        });
    }

    let has_fields_needing_required = field_optionality.iter().any(|f| {
        !f.skip
            && f.wrap_in_option
            && !f.was_optional
            && !f.validation.is_newtype
            && !f.validation.is_nested
    });

    let has_any_koruma_validations = field_optionality.iter().any(|f| {
        !f.skip
            && (!f.validation.field_validators.is_empty()
                || !f.validation.element_validators.is_empty()
                || f.validation.is_nested)
    });

    let effective_enable_koruma =
        enable_koruma || (has_fields_needing_required && has_any_koruma_validations);
    let (value_holder_tokens, fields_requiring_required) = generate_value_holder(
        &original_input,
        &field_optionality,
        effective_enable_koruma,
        enable_koruma_fluent,
    );

    let items_error_struct_fields: Vec<TokenStream> = fields_iter
        .iter()
        .filter(|field| !field.skip() && field.component.is_some())
        .map(|field| {
            let field_name = field.ident.as_ref().unwrap();
            quote! {
                pub #field_name: String,
            }
        })
        .collect();

    let items_error_struct_defaults: Vec<TokenStream> = fields_iter
        .iter()
        .filter(|field| !field.skip() && field.component.is_some())
        .map(|field| {
            let field_name = field.ident.as_ref().unwrap();
            quote! {
                #field_name: String::new(),
            }
        })
        .collect();

    let items_error_has_error_checks: Vec<TokenStream> = fields_iter
        .iter()
        .filter(|field| !field.skip() && field.component.is_some())
        .map(|field| {
            let field_name = field.ident.as_ref().unwrap();
            quote! {
                !self.#field_name.is_empty()
            }
        })
        .collect();

    let field_variant_construction_code: Vec<TokenStream> = fields_iter
        .iter()
        .filter_map(|field| {
            if field.skip() {
                None
            } else if let Some(component_def) = field.component.as_ref() {
                let field_name_str = field
                    .ident
                    .as_ref()
                    .expect("Field should have an ident if not skipped and has component")
                    .to_string();
                let (was_optional, original_inner_type) = extract_option_inner_type(&field.ty);
                let base_type = field
                    .r#type
                    .as_ref()
                    .map(|ty| extract_option_inner_type(&ty.0).1)
                    .unwrap_or(original_inner_type);

                let is_optional = was_optional;
                let field_type_str = base_type.to_token_stream().to_string();
                let behaviour_tokens = get_components_behaviour_tokens(component_def);
                let mut validation_rules = koruma_validations
                    .get(&field_name_str)
                    .cloned()
                    .unwrap_or_default();

                if fields_requiring_required.contains(&field_name_str)
                    && !validation_rules.contains(&"RequiredValidation".to_string())
                {
                    validation_rules.insert(0, "RequiredValidation".to_string());
                }

                let validation_literals: Vec<_> = validation_rules
                    .iter()
                    .map(|v| syn::LitStr::new(v, proc_macro2::Span::call_site()))
                    .collect();

                let default_expr_tokens = field.default.as_ref().map(|expr| {
                    let expr_str = expr.0.to_token_stream().to_string();
                    quote! { .with_default(#expr_str) }
                });

                let custom_component_tokens = if let Components::Custom(opts) = component_def {
                    opts.component.as_ref().map(|comp| {
                        let comp_str = comp.to_token_stream().to_string();
                        quote! { .with_custom_component(#comp_str) }
                    })
                } else {
                    None
                };

                Some(quote! {
                    ::gpui_form::core::registry::FieldVariant::new(
                        #field_name_str,
                        #field_type_str,
                        #is_optional,
                        #behaviour_tokens
                    ).with_validations(&[
                        #( #validation_literals ),*
                    ])
                    #default_expr_tokens
                    #custom_component_tokens
                })
            } else {
                None
            }
        })
        .collect();

    let shape_impl = if options.generate_shape {
        quote! {
            ::gpui_form::core::registry::inventory::submit! {
                ::gpui_form::core::registry::GpuiFormShape::new(
                    stringify!(#struct_name),
                    &[
                        #(#field_variant_construction_code),*
                    ],
                    file!(),
                    #effective_enable_koruma
                )
            }
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
        #value_holder_tokens
        pub struct #components_holder_name {
            #(#field_structure_tokens)*
        }

        #shape_impl

        pub struct #components_base_declarations_name;

        impl #components_base_declarations_name {
          #(#field_base_declarations_tokens)*
        }

        #[derive(Clone, Debug)]
        pub struct #items_errors_struct_name {
            #(#items_error_struct_fields)*
        }

        impl Default for #items_errors_struct_name {
            fn default() -> Self {
                Self {
                    #(#items_error_struct_defaults)*
                }
            }
        }

        impl #items_errors_struct_name {
            pub fn has_errors(&self) -> bool {
                #(#items_error_has_error_checks)||*
            }

            pub fn clear(&mut self) {
                *self = Self::default();
            }
        }
    };

    expanded
}
