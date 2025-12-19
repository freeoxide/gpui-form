use gpui_form_core::{
    components::ComponentsBehaviour,
    registry::{FieldVariant, GpuiFormShape},
};
use proc_macro2::TokenStream;
use quote::quote;

use crate::implementations::ComponentIdentities as _;

use super::{FieldCodeGenerator, GeneratedSubscription};

pub struct TupleSelectCodeGenerator;

impl FieldCodeGenerator for TupleSelectCodeGenerator {
    fn generate_cx_new_call(
        &self,
        field: &FieldVariant,
        component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        let form_components_struct_ident = component.struct_form_components_ident();

        // Generate the master select field name
        let master_var_name = format!("{}_master_select", field.field_name);
        let master_var_name_ident = syn::parse_str::<syn::Ident>(&master_var_name).unwrap();

        Some(quote! {
            let #master_var_name_ident =
                cx.new(|cx| #form_components_struct_ident::#master_var_name_ident(window, cx));
        })
    }

    fn generate_field_initializers(
        &self,
        field: &FieldVariant,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        // Initialize both the master select and the path
        let master_var_name = format!("{}_master_select", field.field_name);
        let master_var_name_ident = syn::parse_str::<syn::Ident>(&master_var_name).unwrap();

        let path_var_name = format!("{}_path", field.field_name);
        let path_var_name_ident = syn::parse_str::<syn::Ident>(&path_var_name).unwrap();

        Some(quote! {
            #master_var_name_ident,
            #path_var_name_ident: gpui_form_component::TupleSelectPath::new(),
        })
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

        let master_field_name = format!("{}_master_select", field.field_name);
        let master_field_name_ident = syn::parse_str::<syn::Ident>(&master_field_name).unwrap();

        // For TupleSelect, we render the master select
        // Child selects would be dynamically rendered based on the current selection
        // For the prototype, we just show the master select with a note about children
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
                    .child(
                        v_flex()
                            .gap_2()
                            .child(Select::new(&self.fields.#master_field_name_ident))
                            // TODO: Child selects would be rendered dynamically here
                            // based on self.fields.{field}_path and the selected master value
                    )
            )
        }
    }

    fn generate_focusable_cycle(
        &self,
        field: &FieldVariant,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        let master_field_name = format!("{}_master_select", field.field_name);
        let master_field_name_ident = syn::parse_str::<syn::Ident>(&master_field_name).unwrap();

        Some(quote! {
            self.fields.#master_field_name_ident.focus_handle(cx),
        })
    }

    fn generate_subscription(
        &self,
        field: &FieldVariant,
        _component: &GpuiFormShape,
    ) -> Option<GeneratedSubscription> {
        let struct_name_ident = field.struct_name_ident();
        let searchable = if let ComponentsBehaviour::TupleSelect(config) = &field.behaviour {
            config.searchable
        } else {
            panic!("Expected TupleSelect behaviour")
        };

        let master_var_name = format!("{}_master_select", field.field_name);
        let master_var_name_ident = syn::parse_str::<syn::Ident>(&master_var_name).unwrap();

        let event_handler_fn_name = format!("on_{}_tuple_select_event", field.field_name);
        let event_handler_fn_name_ident =
            syn::parse_str::<syn::Ident>(&event_handler_fn_name).unwrap();

        let calls = vec![
            quote! { cx.subscribe_in(&#master_var_name_ident, window, Self::#event_handler_fn_name_ident) },
        ];

        let field_name_ident = field.field_ident();
        let path_field_name = format!("{}_path", field.field_name);
        let path_field_name_ident = syn::parse_str::<syn::Ident>(&path_field_name).unwrap();

        let vec_type = if searchable {
            quote! { SearchableVec }
        } else {
            quote! { Vec }
        };

        // The handler updates the current data with the selected value
        // and updates the selection path
        let handler = quote! {
            fn #event_handler_fn_name_ident(
                &mut self,
                this: &Entity<SelectState<#vec_type<gpui_form_component::TupleSelectItem<#struct_name_ident>>>>,
                event: &SelectEvent<#vec_type<gpui_form_component::TupleSelectItem<#struct_name_ident>>>,
                _window: &mut Window,
                cx: &mut Context<Self>,
            ) {
                match event {
                    SelectEvent::Confirm(value) => {
                        if let Some(item) = value {
                            // Get the selected value from the TupleSelectItem
                            let selected = item.get_value().clone();

                            // Update the selection path at depth 0
                            if let Some(index) = this.read(cx).selected_index() {
                                self.fields.#path_field_name_ident.set(0, index.row());
                            }

                            // Update the form data
                            self.current_data.#field_name_ident = selected;

                            // TODO: If the selected variant has children, we would need to:
                            // 1. Create/update child select states
                            // 2. Clear deeper path selections
                            // This requires dynamic UI updates based on has_inner()
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
