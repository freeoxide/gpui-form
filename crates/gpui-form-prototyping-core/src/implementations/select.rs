use gpui_form_core::{
    components::ComponentsBehaviour,
    registry::{FieldVariant, GpuiFormShape},
};
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
        let component_gpui_type = field.behaviour.as_component_ident();

        let field_in_struct_name_ident = field.field_ident_with_behaviour();

        let description_fn_tokens = super::generate_description_fn_tokens(field, component);
        let label_tokens = super::generate_label_tokens(field, component);

        // Show description always, and error below it when present (hidden when empty)
        quote! {
            .child(
                field()
                    .label(#label_tokens)
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

        // Generate handler based on whether field is optional
        // Optional fields: direct assignment (value is already Option<T>)
        // Non-optional fields: unwrap with if let Some pattern
        let handler = if field.optional {
            quote! {
                fn #event_handler_fn_name_ident(
                    &mut self,
                    _this: &Entity<SelectState<#vec_type<#struct_name_ident>>>,
                    event: &SelectEvent<#vec_type<#struct_name_ident>>,
                    _window: &mut Window,
                    _cx: &mut Context<Self>,
                ) {
                    match event {
                        SelectEvent::Confirm(value) => {
                            self.current_data.#field_name_ident = value.clone();
                        },
                    }
                }
            }
        } else {
            quote! {
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
                                self.current_data.#field_name_ident = value.clone();
                            }
                        },
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
