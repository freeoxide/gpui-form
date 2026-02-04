use super::__crate_paths;
use crate::components::*;
use proc_macro2::TokenStream;
use quote::quote;

impl super::ComponentLayout for NumberInputComponent {
    fn field_tokens(
        &self,
        field_structure_tokens: &mut TokenStream,
        field_base_declarations_tokens: &mut TokenStream,
    ) {
        let FieldInformation::<NumberInputOptions> {
            options,
            name,
            r#type,
        } = &self.0;

        let field_name_ident = crate::component_field_name!(name);

        use __crate_paths::gpui::{Context, Entity, Window};
        use __crate_paths::gpui_component::input::InputState;

        let field_structure_definition = quote! {
            pub #field_name_ident: #Entity<#InputState>,
        };

        // Determine if we have an `as` attribute for custom types
        let has_as_type = options.r#as.is_some();

        // Use the `as` option if provided for validation type detection, otherwise use the field type
        let validation_type_ident = options.r#as.as_ref().unwrap_or(r#type);
        let type_str = validation_type_ident.to_string();
        // Treat custom types as signed by default; only explicit `u*` types are unsigned.
        let is_unsigned = type_str.starts_with('u');

        let validation_logic = if is_unsigned {
            let require_parse = !has_as_type;
            quote! {
                .validate(|value, _| {
                    ::gpui_form::numeric::validate_unsigned_numeric::<#r#type>(value, #require_parse)
                })
            }
        } else {
            let require_parse = !has_as_type;
            quote! {
                .validate(|value, _| {
                    ::gpui_form::numeric::validate_signed_numeric::<#r#type>(value, #require_parse)
                })
            }
        };

        let field_base_declaration = quote! {
            pub fn #field_name_ident(
                window: &mut #Window,
                cx: &mut #Context<'_, #InputState>
            ) -> #InputState {
                #InputState::new(window, cx)#validation_logic
            }
        };

        field_structure_tokens.extend(field_structure_definition);
        field_base_declarations_tokens.extend(field_base_declaration);
    }
}
