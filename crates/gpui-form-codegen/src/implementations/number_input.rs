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

        let field_structure_definition = quote! {
            pub #field_name_ident: ::gpui::Entity<::gpui_kit::component::input::InputState>,
        };

        let validation_type = options
            .r#as
            .as_ref()
            .map(|ty| quote! { #ty })
            .unwrap_or_else(|| quote! { #r#type });
        // Same shared classification the recorded metadata payload carries,
        // so an `as = ...` field cannot record one family and validate as another.
        let is_unsigned = number_input_kind(options, r#type).is_unsigned();

        let validation_logic = if is_unsigned {
            quote! {
                .validate(|value, _| {
                    ::gpui_form::numeric::validate_unsigned_numeric::<#validation_type>(value, true)
                })
            }
        } else {
            quote! {
                .validate(|value, _| {
                    ::gpui_form::numeric::validate_signed_numeric::<#validation_type>(value, true)
                })
            }
        };

        let field_base_declaration = quote! {
            pub fn #field_name_ident(
                window: &mut ::gpui::Window,
                cx: &mut ::gpui::Context<'_, ::gpui_kit::component::input::InputState>
            ) -> ::gpui_kit::component::input::InputState {
                ::gpui_kit::component::input::InputState::new(window, cx)#validation_logic
            }
        };

        field_structure_tokens.extend(field_structure_definition);
        field_base_declarations_tokens.extend(field_base_declaration);
    }
}
