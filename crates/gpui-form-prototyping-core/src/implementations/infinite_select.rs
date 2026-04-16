use gpui_form_schema::{
    components::ComponentsBehaviour,
    registry::{FieldVariant, GpuiFormShape},
};
use proc_macro2::TokenStream;
use quote::quote;

use crate::imports::ImportItem;

use super::{FieldCodeGenerator, GeneratedSubscription, ResolvedField, ShapeIdentities as _};

pub struct InfiniteSelectCodeGenerator;

const IMPORTS_BASE: &[ImportItem] = &[
    ImportItem::path("gpui_component::IndexPath"),
    ImportItem::path("gpui_component::select::Select"),
    ImportItem::path("gpui_component::select::SelectEvent"),
    ImportItem::path("gpui_component::select::SelectState"),
    ImportItem::path("gpui_form::infinite_select::InfiniteSelect"),
];

impl FieldCodeGenerator for InfiniteSelectCodeGenerator {
    fn generate_imports(&self, field: &FieldVariant) -> Vec<ImportItem> {
        let mut items = IMPORTS_BASE.to_vec();
        if let ComponentsBehaviour::InfiniteSelect(opts) = &field.behaviour
            && opts.searchable
        {
            items.push(ImportItem::path("gpui_component::select::SearchableVec"));
        }

        items
    }

    fn generate_cx_new_call(
        &self,
        field: &ResolvedField<'_>,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        let field_type = field.value_type();
        let field_name_ident = field.field_ident();
        let master_var_name_ident = field.suffixed_ident("master_select");
        let initial_location_ident = field.prefixed_ident("initial");
        let master_variants_ident = field.prefixed_ident("master_variants");
        let initial_variant_name_ident = field.prefixed_ident("initial_variant_name");
        let initial_variant_idx_ident = field.prefixed_ident("initial_variant_idx");
        let master_selected_index_ident = field.prefixed_ident("master_selected_index");

        Some(quote! {
            let #initial_location_ident = &current_data.#field_name_ident;
            let #master_variants_ident = #field_type::variants();
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
                let items: Vec<gpui_form::infinite_select::InfiniteSelectItem<#field_type>> =
                    gpui_form::infinite_select::to_select_items::<#field_type>();
                gpui_component::select::SelectState::new(items, #master_selected_index_ident, window, cx)
            });
        })
    }

    fn generate_field_initializers(
        &self,
        field: &ResolvedField<'_>,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        // Initialize master select, child selects, and path
        let master_var_name_ident = field.suffixed_ident("master_select");
        let child_selects_var_name_ident = field.suffixed_ident("child_selects");
        let path_var_name_ident = field.suffixed_ident("path");

        Some(quote! {
            #master_var_name_ident,
            #child_selects_var_name_ident,
            #path_var_name_ident: gpui_form::infinite_select::InfiniteSelectPath::new(),
        })
    }

    fn generate_render_child(
        &self,
        field: &ResolvedField<'_>,
        _component: &GpuiFormShape,
    ) -> TokenStream {
        let field_name_ident = field.field_ident();
        let master_field_name_ident = field.suffixed_ident("master_select");
        let child_selects_field_name_ident = field.suffixed_ident("child_selects");

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
        field: &ResolvedField<'_>,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        let master_field_name_ident = field.suffixed_ident("master_select");

        Some(quote! {
            self.fields.#master_field_name_ident.focus_handle(cx),
        })
    }

    fn generate_subscription(
        &self,
        field: &ResolvedField<'_>,
        component: &GpuiFormShape,
    ) -> Option<GeneratedSubscription> {
        let field_type = field.value_type();
        let form_components_struct_ident = component.struct_form_components_ident();

        let searchable = if let ComponentsBehaviour::InfiniteSelect(config) = field.behaviour() {
            config.searchable
        } else {
            panic!("Expected InfiniteSelect behaviour")
        };

        let master_var_name_ident = field.suffixed_ident("master_select");
        let master_event_handler_fn_name_ident = field.event_handler_ident("master_select_event");
        let child_event_handler_fn_name_ident = field.event_handler_ident("child_select_event");

        let calls = vec![
            quote! { cx.subscribe_in(&#master_var_name_ident, window, Self::#master_event_handler_fn_name_ident) },
        ];

        let field_name_ident = field.field_ident();

        let child_selects_field_name_ident = field.suffixed_ident("child_selects");
        let child_helper_fn_name_ident = child_selects_field_name_ident.clone();
        let path_field_name_ident = field.suffixed_ident("path");

        let vec_type = if searchable {
            quote! { SearchableVec }
        } else {
            quote! { Vec }
        };

        let master_handler = quote! {
            fn #master_event_handler_fn_name_ident(
                &mut self,
                this: &Entity<SelectState<#vec_type<gpui_form::infinite_select::InfiniteSelectItem<#field_type>>>>,
                event: &SelectEvent<#vec_type<gpui_form::infinite_select::InfiniteSelectItem<#field_type>>>,
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
                this: &Entity<SelectState<#vec_type<gpui_form::infinite_select::InfiniteSelectItem<#field_type>>>>,
                event: &SelectEvent<#vec_type<gpui_form::infinite_select::InfiniteSelectItem<#field_type>>>,
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
        field: &ResolvedField<'_>,
        component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        let form_components_struct_ident = component.struct_form_components_ident();
        let field_name_ident = field.field_ident();
        let child_selects_var_name_ident = field.suffixed_ident("child_selects");
        let path_var_name_ident = field.suffixed_ident("path");
        let child_helper_fn_name_ident = field.suffixed_ident("child_selects");
        let child_event_handler_fn_name_ident = field.event_handler_ident("child_select_event");
        let initial_variant_idx_ident = field.prefixed_ident("initial_variant_idx");

        Some(quote! {
            let mut #path_var_name_ident = gpui_form::infinite_select::InfiniteSelectPath::new();
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
