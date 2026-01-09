use gpui_form_core::{
    components::ComponentsBehaviour,
    registry::{FieldVariant, GpuiFormShape},
};
use heck::ToSnakeCase as _;
use proc_macro2::TokenStream;
use quote::quote;

use crate::implementations::ComponentIdentities as _;

use super::{FieldCodeGenerator, GeneratedSubscription};

pub struct SelectCodeGenerator;

impl FieldCodeGenerator for SelectCodeGenerator {
    fn generate_cx_new_call(
        &self,
        field: &FieldVariant,
        component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        if field.behaviour.partial() {
            return None;
        }

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
        let field_name_ident = field.field_ident();
        let field_name_pascal_case_ident = field.field_ident_pascal();

        let component_gpui_type = field.behaviour.as_component_ident();

        let field_in_struct_name_ident = field.field_ident_with_behaviour();

        let description_tokens =
            quote! { #ftl_description_ident::#field_name_pascal_case_ident.to_fluent_string() };
        let error_tokens = if component.has_validations() {
            if let Some(validation_path) = field.first_validation_path() {
                let validation_ident = validation_path
                    .segments
                    .last()
                    .map(|s| s.ident.to_string())
                    .unwrap();
                let validation_method_name = validation_ident.to_snake_case();
                let validation_method_ident =
                    syn::parse_str::<syn::Ident>(&validation_method_name).unwrap();

                quote! {{
                    validation_errors
                        .as_ref()
                        .and_then(|e| e.#field_name_ident().#validation_method_ident())
                        .map(|v| v.to_fluent_string())
                }}
            } else {
                quote! {{ None }}
            }
        } else if let Some(validation_path) = field.first_validation_path() {
            let validation_ident = validation_path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap();
            let validation_method_name = validation_ident.to_snake_case();
            let validation_method_ident =
                syn::parse_str::<syn::Ident>(&validation_method_name).unwrap();

            quote! {{
                validation_errors
                    .as_ref()
                    .and_then(|e| e.#field_name_ident().#validation_method_ident())
                    .map(|v| v.to_fluent_string())
            }}
        } else {
            quote! {{
                let e = self.errors.#field_name_ident.clone();
                if e.is_empty() { None } else { Some(e) }
            }}
        };
        let error_color_tokens = quote! { cx.theme().danger };

        let description_fn_tokens =
            if component.has_validations() && field.first_validation_path().is_none() {
                quote! {
                    .description_fn({
                        let description = #description_tokens;
                        move |_, _| {
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(div().child(description.clone()))
                        }
                    })
                }
            } else {
                quote! {
                    .description_fn({
                        let description = #description_tokens;
                        let error = #error_tokens;
                        let error_color = #error_color_tokens;
                        move |_, _| {
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(div().child(description.clone()))
                                .when(error.is_some(), |this| {
                                    this.child(
                                        div()
                                            .text_color(error_color)
                                            .child(error.clone().unwrap_or_default()),
                                    )
                                })
                        }
                    })
                }
            };

        // Show description always, and error below it when present (hidden when empty)
        quote! {
            .child(
                field()
                    .label(#ftl_label_ident::#field_name_pascal_case_ident.to_fluent_string())
                    #description_fn_tokens
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
        _component: &GpuiFormShape,
    ) -> Option<GeneratedSubscription> {
        let struct_name_ident = field.struct_name_ident();
        let searchable = if let ComponentsBehaviour::Select(dropdown_config) = &field.behaviour {
            dropdown_config.searchable
        } else {
            panic!("Expected Select behaviour")
        };
        let field_var_name_ident = field.field_ident_with_behaviour();

        let event_handler_fn_name = format!("on_{}_select_event", field.field_name);
        let event_handler_fn_name_ident =
            syn::parse_str::<syn::Ident>(&event_handler_fn_name).unwrap();

        let calls = vec![
            quote! { cx.subscribe_in(&#field_var_name_ident, window, Self::#event_handler_fn_name_ident) },
        ];

        let field_name_ident = field.field_ident();

        let vec_type = if searchable {
            quote! { SearchableVec }
        } else {
            quote! { Vec }
        };

        let handler = quote! {
            fn #event_handler_fn_name_ident(
                &mut self,
                _this: &Entity<SelectState<#vec_type<#struct_name_ident>>>,
                event: &SelectEvent<#vec_type<#struct_name_ident>>,
                _window: &mut Window,
                _cx: &mut Context<Self>,
            ) {
                match event {
                    SelectEvent::Confirm(value) => {
                        if let Some(value) = value {
                            self.current_data.#field_name_ident = value.clone().into();
                        }
                    },
                }
            }
        };

        Some(GeneratedSubscription {
            calls,
            handlers: vec![handler],
        })
    }
}
