use super::__crate_paths;
use crate::components::*;
use proc_macro2::TokenStream;
use quote::quote;

impl super::ComponentLayout for InputComponent {
    fn field_tokens(
        &self,
        field_structure_tokens: &mut TokenStream,
        field_base_declarations_tokens: &mut TokenStream,
    ) {
        let FieldInformation::<InputOptions> {
            options: _,
            name,
            r#type,
            item_type,
        } = &self.0;

        let field_name_ident = crate::component_field_name!(name);

        use __crate_paths::gpui::{Context, Entity, Window};
        use __crate_paths::gpui_component::input::InputState;

        let field_structure_definition = quote! {
            pub #field_name_ident: #Entity<#InputState>,
        };

        // Generate validation logic based on whether this has an item_type for two-phase validation
        // If item_type is set: validate against item_type during typing (allows intermediate values)
        // If no item_type: validate against field type during typing (original behavior)
        // Skip validation for String types since they always parse successfully
        let type_str = r#type.to_string();
        let validation_logic = if let Some(parse_type) = item_type {
            // Two-phase validation: validate against item_type during typing
            let parse_type_str = parse_type.to_string();
            if parse_type_str == "String" {
                // String always parses, no validation needed
                quote! {}
            } else {
                quote! {
                    .validate(|value, _| value.parse::<#parse_type>().is_ok())
                }
            }
        } else if type_str == "String" {
            // String type: no validation needed (always parses)
            quote! {}
        } else {
            // Single-phase: validate against field type during typing
            quote! {
                .validate(|value, _| value.parse::<#r#type>().is_ok())
            }
        };

        let field_base_declaration = quote! {
            pub fn #field_name_ident(window: &mut #Window, cx: &mut #Context<'_, #InputState>) -> #InputState {
                #InputState::new(window, cx)#validation_logic
            }
        };

        field_structure_tokens.extend(field_structure_definition);
        field_base_declarations_tokens.extend(field_base_declaration);
    }
}
