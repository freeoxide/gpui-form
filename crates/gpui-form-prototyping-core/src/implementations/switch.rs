use gpui_form_core::registry::{FieldVariant, GpuiFormShape};
use heck::ToSnakeCase as _;
use proc_macro2::TokenStream;
use quote::quote;

use crate::implementations::ComponentIdentities as _;

use super::{FieldCodeGenerator, GeneratedSubscription};

pub struct SwitchCodeGenerator;

impl FieldCodeGenerator for SwitchCodeGenerator {
    fn generate_cx_new_call(
        &self,
        _field: &FieldVariant,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        None
    }

    fn generate_field_initializers(
        &self,
        _field: &FieldVariant,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        None
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

        let checkbox_id_str = field.kebab_id();

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
                    .child(#component_gpui_type::new(#checkbox_id_str)
                    .checked(self.current_data.#field_name_ident)
                    .on_click(cx.listener(move |v, checked, _, cx| {
                        v.current_data.#field_name_ident = *checked;
                        cx.notify();
                    })),
                )
            )
        }
    }

    fn generate_focusable_cycle(
        &self,
        _field: &FieldVariant,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        None
    }

    fn generate_subscription(
        &self,
        _field: &FieldVariant,
        _component: &GpuiFormShape,
    ) -> Option<GeneratedSubscription> {
        None
    }
}
