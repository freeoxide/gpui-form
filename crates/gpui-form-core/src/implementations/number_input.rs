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
        let is_signed = type_str.starts_with('i') || type_str.starts_with('f');

        let validation_logic = if is_signed {
            if has_as_type {
                // For custom types with `as`, only validate character pattern (no parse check)
                quote! {
                    .validate(|value, _| {
                        if value.is_empty() {
                            return true;
                        }

                        let chars: Vec<char> = value.chars().collect();

                        // Allow just "-" for intermediate input
                        if chars.len() == 1 && chars[0] == '-' {
                            return true;
                        }

                        // First character: must be 0-9 or '-'
                        if !chars[0].is_ascii_digit() && chars[0] != '-' {
                            return false;
                        }

                        // If there are more characters, validate from second character onwards
                        if chars.len() > 1 {
                            let start_idx = if chars[0] == '-' { 1 } else { 0 };

                            // Check for invalid leading zeros: "0X" where X is a digit (not decimal point)
                            // Allow: "0", "0.", "0.5", "-0", "-0.", "-0.5"
                            // Reject: "00", "01", "-00", "-01"
                            if chars[start_idx] == '0' && chars.len() > start_idx + 1 && chars[start_idx + 1].is_ascii_digit() {
                                return false;
                            }

                            // Second character onwards: can't be '-'
                            if !chars[start_idx..].iter().all(|&c| c.is_ascii_digit() || c == '.') {
                                return false;
                            }
                        }

                        true
                    })
                }
            } else {
                // For standard types, validate character pattern AND parse
                quote! {
                    .validate(|value, _| {
                        if value.is_empty() {
                            return true;
                        }

                        let chars: Vec<char> = value.chars().collect();

                        // Allow just "-" for intermediate input
                        if chars.len() == 1 && chars[0] == '-' {
                            return true;
                        }

                        // First character: must be 0-9 or '-'
                        if !chars[0].is_ascii_digit() && chars[0] != '-' {
                            return false;
                        }

                        // If there are more characters, validate from second character onwards
                        if chars.len() > 1 {
                            let start_idx = if chars[0] == '-' { 1 } else { 0 };

                            // Check for invalid leading zeros: "0X" where X is a digit (not decimal point)
                            // Allow: "0", "0.", "0.5", "-0", "-0.", "-0.5"
                            // Reject: "00", "01", "-00", "-01"
                            if chars[start_idx] == '0' && chars.len() > start_idx + 1 && chars[start_idx + 1].is_ascii_digit() {
                                return false;
                            }

                            // Second character onwards: can't be '-'
                            if !chars[start_idx..].iter().all(|&c| c.is_ascii_digit() || c == '.') {
                                return false;
                            }
                        }

                        // Finally, check if it can parse
                        value.parse::<#r#type>().is_ok()
                    })
                }
            }
        } else {
            if has_as_type {
                // For custom types with `as`, only validate character pattern (no parse check)
                quote! {
                    .validate(|value, _| {
                        if value.is_empty() {
                            return true;
                        }

                        let chars: Vec<char> = value.chars().collect();

                        // For unsigned types, first character must be 0-9
                        if !chars[0].is_ascii_digit() {
                            return false;
                        }

                        // Check for invalid leading zeros: "0X" where X is a digit
                        // Allow: "0"
                        // Reject: "00", "01", "001"
                        if chars[0] == '0' && chars.len() > 1 && chars[1].is_ascii_digit() {
                            return false;
                        }

                        // All characters must be digits
                        if !chars.iter().all(|&c| c.is_ascii_digit()) {
                            return false;
                        }

                        true
                    })
                }
            } else {
                // For standard types, validate character pattern AND parse
                quote! {
                    .validate(|value, _| {
                        if value.is_empty() {
                            return true;
                        }

                        let chars: Vec<char> = value.chars().collect();

                        // For unsigned types, first character must be 0-9
                        if !chars[0].is_ascii_digit() {
                            return false;
                        }

                        // Check for invalid leading zeros: "0X" where X is a digit
                        // Allow: "0"
                        // Reject: "00", "01", "001"
                        if chars[0] == '0' && chars.len() > 1 && chars[1].is_ascii_digit() {
                            return false;
                        }

                        // All characters must be digits
                        if !chars.iter().all(|&c| c.is_ascii_digit()) {
                            return false;
                        }

                        // Finally, check if it can parse
                        value.parse::<#r#type>().is_ok()
                    })
                }
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
