use gpui_form_core::components::*;
use gpui_form_core::implementations::ComponentLayout as _;
use proc_macro2::TokenStream;
use quote::quote;

use crate::derives::gpui_form::structs::ComponentField;
use crate::derives::gpui_form::structs::ComponentFieldContent;
use crate::derives::gpui_form::utils::extract_type_ident;

pub fn get_components_behaviour_tokens(component: &Components) -> TokenStream {
    match component {
        Components::Input => {
            quote! { ::gpui_form::core::components::ComponentsBehaviour::Input }
        },
        Components::NumberInput(_) => {
            quote! { ::gpui_form::core::components::ComponentsBehaviour::NumberInput }
        },
        Components::Checkbox => {
            quote! { ::gpui_form::core::components::ComponentsBehaviour::Checkbox }
        },
        Components::Switch => {
            quote! { ::gpui_form::core::components::ComponentsBehaviour::Switch }
        },
        Components::Select(options) => {
            let searchable = options.behaviour.searchable;
            let partial = options.behaviour.partial;
            quote! {
                ::gpui_form::core::components::ComponentsBehaviour::Select(
                    ::gpui_form::core::components::BehaviourSelectOptions {
                        searchable: #searchable,
                        partial: #partial,
                    }
                )
            }
        },
        Components::InfiniteSelect(options) => {
            let searchable = options.behaviour.searchable;
            let max_depth = match options.behaviour.max_depth {
                Some(d) => quote! { Some(#d) },
                None => quote! { None },
            };
            quote! {
                ::gpui_form::core::components::ComponentsBehaviour::InfiniteSelect(
                    ::gpui_form::core::components::BehaviourInfiniteSelectOptions {
                        searchable: #searchable,
                        max_depth: #max_depth,
                    }
                )
            }
        },
        Components::Custom(_) => {
            quote! { ::gpui_form::core::components::ComponentsBehaviour::Custom }
        },
        Components::DatePicker => {
            quote! { ::gpui_form::core::components::ComponentsBehaviour::DatePicker }
        },
    }
}

/// Extracts a path from a field default expression if it's a path expression
fn extract_default_path(field: &ComponentField) -> Option<syn::Path> {
    field.default.as_ref().and_then(|expr| {
        if let syn::Expr::Path(expr_path) = &expr.0 {
            Some(expr_path.path.clone())
        } else {
            None
        }
    })
}

pub fn generate_component_field(field: &ComponentField) -> ComponentFieldContent {
    let field_name = field.ident.as_ref().unwrap().to_string();
    let field_type = field.r#type.as_ref().map(|ty| &ty.0).unwrap_or(&field.ty);

    let mut field_structure_tokens = proc_macro2::TokenStream::new();
    let mut field_base_declarations_tokens = proc_macro2::TokenStream::new();

    let component_def = match &field.component {
        Some(c) => c,
        None => {
            return ComponentFieldContent {
                field_structure_tokens,
                field_base_declarations_tokens,
                wrap_in_option: (field_name, false),
            };
        },
    };

    let wrap_in_option = component_def.wraps_in_option();

    match component_def {
        Components::Input => {
            let component = InputComponent(FieldInformation::new(
                InputOptions,
                field_name.clone(),
                extract_type_ident(field_type),
            ));
            component.field_tokens(
                &mut field_structure_tokens,
                &mut field_base_declarations_tokens,
            );
        },
        Components::NumberInput(options) => {
            let component = NumberInputComponent(FieldInformation::new(
                options.clone(),
                field_name.clone(),
                extract_type_ident(field_type),
            ));
            component.field_tokens(
                &mut field_structure_tokens,
                &mut field_base_declarations_tokens,
            );
        },
        Components::Checkbox => {
            let component = CheckboxComponent(FieldInformation::new(
                CheckboxOptions,
                field_name.clone(),
                extract_type_ident(field_type),
            ));
            component.field_tokens(
                &mut field_structure_tokens,
                &mut field_base_declarations_tokens,
            );
        },
        Components::Switch => {
            let component = SwitchComponent(FieldInformation::new(
                SwitchOptions,
                field_name.clone(),
                extract_type_ident(field_type),
            ));
            component.field_tokens(
                &mut field_structure_tokens,
                &mut field_base_declarations_tokens,
            );
        },
        Components::Select(options) => {
            let field_default = extract_default_path(field);
            let options_with_default = options.clone().with_field_default(field_default);
            let component = SelectComponent(FieldInformation::new(
                options_with_default,
                field_name.clone(),
                extract_type_ident(field_type),
            ));
            component.field_tokens(
                &mut field_structure_tokens,
                &mut field_base_declarations_tokens,
            );
        },
        Components::InfiniteSelect(options) => {
            let field_default = extract_default_path(field);
            let options_with_default = options.clone().with_field_default(field_default);
            let component = InfiniteSelectComponent(FieldInformation::new(
                options_with_default,
                field_name.clone(),
                extract_type_ident(field_type),
            ));
            component.field_tokens(
                &mut field_structure_tokens,
                &mut field_base_declarations_tokens,
            );
        },
        Components::Custom(options) => {
            let component = CustomComponent(FieldInformation::new(
                options.clone(),
                field_name.clone(),
                extract_type_ident(field_type),
            ));
            component.field_tokens(
                &mut field_structure_tokens,
                &mut field_base_declarations_tokens,
            );
        },
        Components::DatePicker => {
            let component = DatePickerComponent(FieldInformation::new(
                DatePickerOptions,
                field_name.clone(),
                extract_type_ident(field_type),
            ));
            component.field_tokens(
                &mut field_structure_tokens,
                &mut field_base_declarations_tokens,
            );
        },
    }

    ComponentFieldContent {
        field_structure_tokens,
        field_base_declarations_tokens,
        wrap_in_option: (field_name, wrap_in_option),
    }
}
