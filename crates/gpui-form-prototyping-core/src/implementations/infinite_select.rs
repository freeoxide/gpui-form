use gpui_form_core::{
    components::ComponentsBehaviour,
    registry::{FieldVariant, GpuiFormShape},
};
use proc_macro2::TokenStream;
use quote::quote;

use crate::implementations::ComponentIdentities as _;

use super::{FieldCodeGenerator, GeneratedSubscription};

pub struct InfiniteSelectCodeGenerator;

impl FieldCodeGenerator for InfiniteSelectCodeGenerator {
    fn generate_cx_new_call(
        &self,
        field: &FieldVariant,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        let struct_name_ident = field.struct_name_ident();
        let field_name_ident = field.field_ident();

        let master_var_name = format!("{}_master_select", field.field_name);
        let master_var_name_ident = syn::parse_str::<syn::Ident>(&master_var_name).unwrap();

        let initial_location_var = format!("initial_{}", field.field_name);
        let initial_location_ident = syn::parse_str::<syn::Ident>(&initial_location_var).unwrap();

        let master_variants_var = format!("master_variants_{}", field.field_name);
        let master_variants_ident = syn::parse_str::<syn::Ident>(&master_variants_var).unwrap();

        let initial_variant_name_var = format!("initial_variant_name_{}", field.field_name);
        let initial_variant_name_ident =
            syn::parse_str::<syn::Ident>(&initial_variant_name_var).unwrap();

        let initial_variant_idx_var = format!("initial_variant_idx_{}", field.field_name);
        let initial_variant_idx_ident =
            syn::parse_str::<syn::Ident>(&initial_variant_idx_var).unwrap();

        let master_selected_index_var = format!("master_selected_index_{}", field.field_name);
        let master_selected_index_ident =
            syn::parse_str::<syn::Ident>(&master_selected_index_var).unwrap();

        Some(quote! {
            let #initial_location_ident = &current_data.#field_name_ident;
            let #master_variants_ident = #struct_name_ident::variants();
            let #initial_variant_name_ident = #initial_location_ident.variant_name();
            let #initial_variant_idx_ident = #master_variants_ident
                .iter()
                .position(|v| v.variant_name() == #initial_variant_name_ident)
                .unwrap_or(0);

            let #master_selected_index_ident = Some(gpui_component::IndexPath {
                section: 0,
                row: #initial_variant_idx_ident,
                column: 0,
            });

            let #master_var_name_ident = cx.new(|cx| {
                let items: Vec<gpui_form::gpui_form_component::infinite_select::InfiniteSelectItem<#struct_name_ident>> =
                    gpui_form::gpui_form_component::infinite_select::to_select_items::<#struct_name_ident>();
                gpui_component::select::SelectState::new(items, #master_selected_index_ident, window, cx)
            });
        })
    }

    fn generate_field_initializers(
        &self,
        field: &FieldVariant,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        // Initialize master select, child selects, and path
        let master_var_name = format!("{}_master_select", field.field_name);
        let master_var_name_ident = syn::parse_str::<syn::Ident>(&master_var_name).unwrap();

        let child_selects_var_name = format!("{}_child_selects", field.field_name);
        let child_selects_var_name_ident =
            syn::parse_str::<syn::Ident>(&child_selects_var_name).unwrap();

        let path_var_name = format!("{}_path", field.field_name);
        let path_var_name_ident = syn::parse_str::<syn::Ident>(&path_var_name).unwrap();

        Some(quote! {
            #master_var_name_ident,
            #child_selects_var_name_ident,
            #path_var_name_ident: gpui_form::gpui_form_component::infinite_select::InfiniteSelectPath::new(),
        })
    }

    fn generate_render_child(
        &self,
        field: &FieldVariant,
        _component: &GpuiFormShape,
    ) -> TokenStream {
        let field_name_ident = field.field_ident();

        let master_field_name = format!("{}_master_select", field.field_name);
        let master_field_name_ident = syn::parse_str::<syn::Ident>(&master_field_name).unwrap();

        let child_selects_field_name = format!("{}_child_selects", field.field_name);
        let child_selects_field_name_ident =
            syn::parse_str::<syn::Ident>(&child_selects_field_name).unwrap();

        quote! {
            .child({
                field()
                    .label(self.current_data.#field_name_ident.type_label())
                    .description_fn({
                        let description = self.current_data.#field_name_ident.type_description();
                        move |_, _| {
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(div().child(description.clone()))
                        }
                    })
                    .child(Select::new(&self.fields.#master_field_name_ident))
            })
            .children({
                self.fields.#child_selects_field_name_ident.iter().enumerate().map(|(i, child)| {
                    field()
                        .label(self.current_data.#field_name_ident.child_label_at_depth(i).unwrap_or("".into()))
                        .description_fn({
                            let description = self
                                .current_data
                                .#field_name_ident
                                .child_description_at_depth(i)
                                .unwrap_or("".into());
                            move |_, _| {
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(div().child(description.clone()))
                            }
                        })
                        .child(Select::new(child))
                })
            })
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
        component: &GpuiFormShape,
    ) -> Option<GeneratedSubscription> {
        let struct_name_ident = field.struct_name_ident();
        let form_components_struct_ident = component.struct_form_components_ident();

        let searchable = if let ComponentsBehaviour::InfiniteSelect(config) = &field.behaviour {
            config.searchable
        } else {
            panic!("Expected InfiniteSelect behaviour")
        };

        let master_var_name = format!("{}_master_select", field.field_name);
        let master_var_name_ident = syn::parse_str::<syn::Ident>(&master_var_name).unwrap();

        let master_event_handler_fn_name = format!("on_{}_master_select_event", field.field_name);
        let master_event_handler_fn_name_ident =
            syn::parse_str::<syn::Ident>(&master_event_handler_fn_name).unwrap();

        let child_event_handler_fn_name = format!("on_{}_child_select_event", field.field_name);
        let child_event_handler_fn_name_ident =
            syn::parse_str::<syn::Ident>(&child_event_handler_fn_name).unwrap();

        let calls = vec![
            quote! { cx.subscribe_in(&#master_var_name_ident, window, Self::#master_event_handler_fn_name_ident) },
        ];

        let field_name_ident = field.field_ident();

        let child_selects_field_name = format!("{}_child_selects", field.field_name);
        let child_selects_field_name_ident =
            syn::parse_str::<syn::Ident>(&child_selects_field_name).unwrap();

        let child_helper_fn_name = format!("{}_child_selects", field.field_name);
        let child_helper_fn_name_ident =
            syn::parse_str::<syn::Ident>(&child_helper_fn_name).unwrap();

        let path_field_name = format!("{}_path", field.field_name);
        let path_field_name_ident = syn::parse_str::<syn::Ident>(&path_field_name).unwrap();

        let vec_type = if searchable {
            quote! { SearchableVec }
        } else {
            quote! { Vec }
        };

        let master_handler = quote! {
            fn #master_event_handler_fn_name_ident(
                &mut self,
                this: &Entity<SelectState<#vec_type<gpui_form::gpui_form_component::infinite_select::InfiniteSelectItem<#struct_name_ident>>>>,
                event: &SelectEvent<#vec_type<gpui_form::gpui_form_component::infinite_select::InfiniteSelectItem<#struct_name_ident>>>,
                window: &mut Window,
                cx: &mut Context<Self>,
            ) {
                if let SelectEvent::Confirm(Some(selected)) = event {
                    if let Some(index) = this.read(cx).selected_index(cx) {
                        self.fields.#path_field_name_ident.set(0, index.row);
                    }
                    self.current_data.#field_name_ident = selected.clone();

                    // Clear and rebuild all child selects from level 0
                    self.fields.#child_selects_field_name_ident.clear();

                    let new_children =
                        #form_components_struct_ident::#child_helper_fn_name_ident(&selected, 0, window, cx);

                    for child in &new_children {
                        let sub = cx.subscribe_in(child, window, Self::#child_event_handler_fn_name_ident);
                        self._subscriptions.push(sub);
                    }

                    self.fields.#child_selects_field_name_ident = new_children;
                    cx.notify();
                }
            }
        };

        let child_handler = quote! {
            fn #child_event_handler_fn_name_ident(
                &mut self,
                this: &Entity<SelectState<#vec_type<gpui_form::gpui_form_component::infinite_select::InfiniteSelectItem<#struct_name_ident>>>>,
                event: &SelectEvent<#vec_type<gpui_form::gpui_form_component::infinite_select::InfiniteSelectItem<#struct_name_ident>>>,
                window: &mut Window,
                cx: &mut Context<Self>,
            ) {
                if let SelectEvent::Confirm(Some(selected)) = event {
                    // Find which level this child select is at
                    let level = self.fields.#child_selects_field_name_ident
                        .iter()
                        .position(|s| s == this)
                        .map(|pos| pos + 1)  // +1 because master is level 0
                        .unwrap_or(1);

                    if let Some(index) = this.read(cx).selected_index(cx) {
                        self.fields.#path_field_name_ident.set(level, index.row);
                    }
                    self.current_data.#field_name_ident = selected.clone();

                    // Remove child selects after this level and rebuild
                    self.fields.#child_selects_field_name_ident.truncate(level);

                    // Add child selects for remaining levels if the selected value has children
                    if selected.has_inner() {
                        let new_children = #form_components_struct_ident::#child_helper_fn_name_ident(
                            &selected, level, window, cx,
                        );

                        for child in &new_children {
                            let sub = cx.subscribe_in(child, window, Self::#child_event_handler_fn_name_ident);
                            self._subscriptions.push(sub);
                        }

                        self.fields.#child_selects_field_name_ident.extend(new_children);
                    }
                    cx.notify();
                }
            }
        };

        Some(GeneratedSubscription {
            calls,
            handlers: vec![master_handler, child_handler],
        })
    }

    fn generate_post_subscription_initialization(
        &self,
        field: &FieldVariant,
        component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        let form_components_struct_ident = component.struct_form_components_ident();
        let field_name_ident = field.field_ident();

        let child_selects_var_name = format!("{}_child_selects", field.field_name);
        let child_selects_var_name_ident =
            syn::parse_str::<syn::Ident>(&child_selects_var_name).unwrap();

        let path_var_name = format!("{}_path", field.field_name);
        let path_var_name_ident = syn::parse_str::<syn::Ident>(&path_var_name).unwrap();

        let child_helper_fn_name = format!("{}_child_selects", field.field_name);
        let child_helper_fn_name_ident =
            syn::parse_str::<syn::Ident>(&child_helper_fn_name).unwrap();

        let child_event_handler_fn_name = format!("on_{}_child_select_event", field.field_name);
        let child_event_handler_fn_name_ident =
            syn::parse_str::<syn::Ident>(&child_event_handler_fn_name).unwrap();

        let initial_variant_idx_var = format!("initial_variant_idx_{}", field.field_name);
        let initial_variant_idx_ident =
            syn::parse_str::<syn::Ident>(&initial_variant_idx_var).unwrap();

        Some(quote! {
            let mut #path_var_name_ident = gpui_form::gpui_form_component::infinite_select::InfiniteSelectPath::new();
            #path_var_name_ident.set(0, #initial_variant_idx_ident);

            let #child_selects_var_name_ident = #form_components_struct_ident::#child_helper_fn_name_ident(
                &current_data.#field_name_ident,
                0,
                window,
                cx,
            );

            for child in &#child_selects_var_name_ident {
                let sub = cx.subscribe_in(child, window, Self::#child_event_handler_fn_name_ident);
                _subscriptions.push(sub);
            }
        })
    }
}
