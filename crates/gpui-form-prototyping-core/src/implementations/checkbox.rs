use gpui_form_core::registry::{FieldVariant, GpuiFormShape};
use proc_macro2::TokenStream;
use quote::quote;

use crate::implementations::ComponentIdentities as _;

use super::{FieldCodeGenerator, GeneratedSubscription};

pub struct CheckboxCodeGenerator;

impl FieldCodeGenerator for CheckboxCodeGenerator {
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
        let field_has_validations = !field.validations.is_empty();
        let error_tokens = if field_has_validations {
            quote! {{
                validation_errors.as_ref().and_then(|e| {
                    let errs = e.#field_name_ident().all();
                    if errs.is_empty() {
                        None
                    } else {
                        Some(
                            errs.iter()
                                .map(|v| v.to_fluent_string())
                                .collect::<Vec<_>>()
                                .join("\n"),
                        )
                    }
                })
            }}
        } else {
            quote! {{ None }}
        };
        let error_color_tokens = quote! { cx.theme().danger };

        let description_fn_tokens = if !field_has_validations {
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
                    .on_click(cx.listener(|v, _, _, _| {
                        v.current_data.#field_name_ident = !v.current_data.#field_name_ident;
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
