use gpui_form_core::registry::{FieldVariant, GpuiFormShape};
use proc_macro2::TokenStream;
use quote::quote;

use crate::implementations::ComponentIdentities as _;

use super::{FieldCodeGenerator, GeneratedSubscription};

pub struct InputCodeGenerator;

impl FieldCodeGenerator for InputCodeGenerator {
    fn generate_cx_new_call(
        &self,
        field: &FieldVariant,
        component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        let form_components_struct_ident = component.struct_form_components_ident();
        let var_name_ident = field.field_ident_with_behaviour();
        let fn_name_ident = var_name_ident.clone();

        Some(quote! {
            let #var_name_ident =
                cx.new(|cx| #form_components_struct_ident::#fn_name_ident(window, cx));
        })
    }

    fn generate_field_initializers(
        &self,
        field: &FieldVariant,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        let field_var_name_ident = field.field_ident_with_behaviour();

        Some(quote! { #field_var_name_ident, })
    }

    fn generate_render_child(
        &self,
        field: &FieldVariant,
        component: &GpuiFormShape,
    ) -> TokenStream {
        let ftl_label_ident = component.ftl_label_ident();
        let ftl_description_ident = component.ftl_description_ident();
        let field_name_pascal_case_ident = field.field_ident_pascal();
        let field_name_ident = field.field_ident();

        let component_gpui_type = field.behaviour.as_component_ident();

        let field_in_struct_name_ident = field.field_ident_with_behaviour();

        // Show description always, and error below it when present (hidden when empty)
        quote! {
            .child(
                field()
                    .label(#ftl_label_ident::#field_name_pascal_case_ident.to_fluent_string())
                    .description_fn({
                        let error = self.errors.#field_name_ident.clone();
                        let description = #ftl_description_ident::#field_name_pascal_case_ident.to_fluent_string();
                        move |_, _| {
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(div().child(description.clone()))
                                .when(!error.is_empty(), |this| {
                                    this.child(
                                        div()
                                            .text_color(gpui::red())
                                            .child(error.clone())
                                    )
                                })
                        }
                    })
                    .child(#component_gpui_type::new(&self.fields.#field_in_struct_name_ident))
            )
        }
    }

    fn generate_focusable_cycle(
        &self,
        field: &FieldVariant,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        let field_var_name_ident = field.field_ident_with_behaviour();
        let x = quote! {
          self.fields.#field_var_name_ident.focus_handle(cx),
        };
        Some(x)
    }

    fn generate_subscription(
        &self,
        field: &FieldVariant,
        component: &GpuiFormShape,
    ) -> Option<GeneratedSubscription> {
        let field_var_name_ident = field.field_ident_with_behaviour();

        let event_handler_fn_name = format!("on_{}_input_event", field.field_name);
        let event_handler_fn_name_ident =
            syn::parse_str::<syn::Ident>(&event_handler_fn_name).unwrap();

        let calls = vec![
            quote! { cx.subscribe_in(&#field_var_name_ident, window, Self::#event_handler_fn_name_ident) },
        ];

        let field_name_ident = field.field_ident();

        // Get the type to use for parsing (item_type if set, otherwise field_type)
        let parse_type_str = field.parse_type();
        let parse_type_path = syn::parse_str::<syn::Type>(parse_type_str).unwrap();

        // Get the type to use for final validation (always field_type)
        let validation_type_str = field.validation_type();
        let validation_type_path = syn::parse_str::<syn::Type>(validation_type_str).unwrap();

        // Get the FTL errors enum ident for this form
        let ftl_errors_ident = component.ftl_errors_ident();
        let field_name_pascal_case_ident = field.field_ident_pascal();

        // Generate different assignment logic based on whether we have two-phase validation
        let handler = if field.has_item_type() {
            // Two-phase validation: parse with item_type, validate with field_type
            // On Change: parse with item_type, use try_new to create the nutype
            // On Blur: validate with field_type, show error if invalid
            quote! {
                fn #event_handler_fn_name_ident(
                    &mut self,
                    state: &Entity<InputState>,
                    event: &InputEvent,
                    _window: &mut Window,
                    _cx: &mut Context<Self>,
                ) {
                    match event {
                        InputEvent::Change => {
                            let text = state.read(_cx).value();
                            // Parse using the intermediate type (e.g., String for Username)
                            // Then try to create the nutype - if it fails, that's ok during typing
                            if let Ok(parsed_value) = text.parse::<#parse_type_path>() {
                                // Try to create the nutype from the parsed value
                                if let Ok(validated) = #validation_type_path::try_new(parsed_value) {
                                    self.current_data.#field_name_ident = validated.into();
                                    self.errors.#field_name_ident.clear();
                                }
                                // If try_new fails, we keep the old value - error shown on blur
                            }
                        }
                        InputEvent::Blur => {
                            let text = state.read(_cx).value();
                            // On blur, try to parse and validate
                            if let Ok(parsed_value) = text.parse::<#parse_type_path>() {
                                match #validation_type_path::try_new(parsed_value) {
                                    Ok(validated_value) => {
                                        self.current_data.#field_name_ident = validated_value.into();
                                        self.errors.#field_name_ident.clear();
                                    }
                                    Err(e) => {
                                        // Use FTL enum for localized error message
                                        self.errors.#field_name_ident = #ftl_errors_ident::#field_name_pascal_case_ident {
                                            value: format!("{:?}", e)
                                        }.to_fluent_string();
                                    }
                                }
                            } else {
                                self.errors.#field_name_ident = #ftl_errors_ident::#field_name_pascal_case_ident {
                                    value: format!("Invalid {} format", stringify!(#parse_type_path))
                                }.to_fluent_string();
                            }
                            _cx.notify();
                        }
                        _ => {}
                    }
                }
            }
        } else {
            // Single-phase: parse and assign directly (like regular types)
            quote! {
                fn #event_handler_fn_name_ident(
                    &mut self,
                    state: &Entity<InputState>,
                    event: &InputEvent,
                    _window: &mut Window,
                    _cx: &mut Context<Self>,
                ) {
                    match event {
                        InputEvent::Change => {
                            let text = state.read(_cx).value();
                            self.current_data.#field_name_ident = text.to_owned().into();
                        }
                        _ => {}
                    }
                }
            }
        };

        Some(GeneratedSubscription {
            calls,
            handlers: vec![handler],
        })
    }
}
