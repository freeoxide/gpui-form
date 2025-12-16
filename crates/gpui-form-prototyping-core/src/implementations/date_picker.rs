use gpui_form_core::registry::{FieldVariant, GpuiFormShape};
use proc_macro2::TokenStream;
use quote::quote;

use crate::implementations::ComponentIdentities as _;

use super::{FieldCodeGenerator, GeneratedSubscription};

pub struct DatePickerCodeGenerator;

fn parse_date_expr(date_ident: &syn::Ident, field_type: &str) -> TokenStream {
    let type_path = syn::parse_str::<syn::Type>(field_type).expect("valid type path");

    quote! {
        <#type_path as std::str::FromStr>::from_str(&#date_ident.to_string())
    }
}

fn value_assign(field: &FieldVariant, field_name_ident: &syn::Ident) -> TokenStream {
    let date_ident = syn::parse_str::<syn::Ident>("date").expect("date ident");
    let parse_expr = parse_date_expr(&date_ident, field.field_type);

    if field.optional {
        quote! {
            self.current_data.#field_name_ident = (#parse_expr).ok();
        }
    } else {
        quote! {
            self.current_data.#field_name_ident = (#parse_expr).unwrap_or_default();
        }
    }
}

impl FieldCodeGenerator for DatePickerCodeGenerator {
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
        let field_name_ident = field.field_ident();
        let field_name_pascal_case_ident = field.field_ident_pascal();

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
        _component: &GpuiFormShape,
    ) -> Option<GeneratedSubscription> {
        let field_var_name_ident = field.field_ident_with_behaviour();

        let event_handler_fn_name = format!("on_{}_date_picker_event", field.field_name);
        let event_handler_fn_name_ident =
            syn::parse_str::<syn::Ident>(&event_handler_fn_name).unwrap();

        let calls = vec![
            quote! { cx.subscribe_in(&#field_var_name_ident, window, Self::#event_handler_fn_name_ident) },
        ];

        let field_name_ident = field.field_ident();

        let value_assign = value_assign(field, &field_name_ident);

        let handler = quote! {
            fn #event_handler_fn_name_ident(
                &mut self,
                _this: &Entity<DatePickerState>,
                event: &DatePickerEvent,
                _: &mut Window,
                _: &mut Context<Self>,
            ) {
                match event {
                    DatePickerEvent::Change(date) => {
                        #value_assign
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
