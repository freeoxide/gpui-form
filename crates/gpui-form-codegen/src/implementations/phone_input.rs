use crate::components::*;
use proc_macro2::TokenStream;
use quote::quote;

impl super::ComponentLayout for PhoneInputComponent {
    fn field_tokens(
        &self,
        field_structure_tokens: &mut TokenStream,
        field_base_declarations_tokens: &mut TokenStream,
    ) {
        // The emitted validator ignores `country`: cross-field selected-country
        // enforcement needs the sibling's value, unavailable in this control.
        let FieldInformation::<PhoneInputOptions> {
            options: _,
            name,
            r#type: _,
        } = &self.0;

        let field_name_ident = crate::component_field_name!(name);

        let field_structure_definition = quote! {
            pub #field_name_ident: ::gpui::Entity<::gpui_kit::component::input::InputState>,
        };

        // Empty input passes so partially typed or cleared fields are not
        // flagged mid-entry; required-ness is the validation layer's concern.
        let field_base_declaration = quote! {
            pub fn #field_name_ident(
                window: &mut ::gpui::Window,
                cx: &mut ::gpui::Context<'_, ::gpui_kit::component::input::InputState>
            ) -> ::gpui_kit::component::input::InputState {
                ::gpui_kit::component::input::InputState::new(window, cx)
                    .validate(|value, _| {
                        ::gpui_form::phone::validate_optional_phone_number(value, None)
                            .is_valid_or_empty()
                    })
            }
        };

        field_structure_tokens.extend(field_structure_definition);
        field_base_declarations_tokens.extend(field_base_declaration);
    }
}
