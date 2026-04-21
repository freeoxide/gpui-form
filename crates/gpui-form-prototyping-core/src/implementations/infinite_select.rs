use gpui_form_schema::{
    components::ComponentsBehaviour,
    registry::{FieldVariant, GpuiFormShape},
};
use proc_macro2::TokenStream;
use quote::quote;

use crate::imports::ImportItem;

use super::{
    FieldCodeGenerator, GeneratedSubscription, ResolvedField, generate_entity_field_initializer,
    generate_entity_focus,
};

pub struct InfiniteSelectCodeGenerator;

const IMPORTS_BASE: &[ImportItem] = &[
    ImportItem::path("gpui_component::select::Select"),
    ImportItem::path("gpui_form::infinite_select::InfiniteSelectEvent"),
];

impl FieldCodeGenerator for InfiniteSelectCodeGenerator {
    fn generate_imports(&self, field: &FieldVariant) -> Vec<ImportItem> {
        let mut items = IMPORTS_BASE.to_vec();
        if let ComponentsBehaviour::InfiniteSelect(opts) = &field.behaviour {
            if opts.searchable {
                items.push(ImportItem::path(
                    "gpui_form::infinite_select::SearchableInfiniteSelectState",
                ));
            } else {
                items.push(ImportItem::path(
                    "gpui_form::infinite_select::InfiniteSelectState",
                ));
            }
        }
        items
    }

    fn generate_cx_new_call(
        &self,
        field: &ResolvedField<'_>,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        let field_type = field.value_type();
        let field_var_name_ident = field.field_ident_with_behaviour();
        let field_name_ident = field.field_ident();

        let searchable = if let ComponentsBehaviour::InfiniteSelect(config) = field.behaviour() {
            config.searchable
        } else {
            panic!("Expected InfiniteSelect behaviour")
        };

        let state_constructor = if searchable {
            quote! { SearchableInfiniteSelectState::<#field_type> }
        } else {
            quote! { InfiniteSelectState::<#field_type> }
        };

        let options_expr = infinite_select_options(field);

        Some(quote! {
            let #field_var_name_ident = cx.new(|cx| {
                #state_constructor::new_with_options(
                    current_data.#field_name_ident.clone(),
                    #options_expr,
                    window,
                    cx,
                )
            });
        })
    }

    fn generate_field_initializers(
        &self,
        field: &ResolvedField<'_>,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        Some(generate_entity_field_initializer(field))
    }

    fn generate_render_child(
        &self,
        field: &ResolvedField<'_>,
        _component: &GpuiFormShape,
    ) -> TokenStream {
        let field_state_ident = field.field_ident_with_behaviour();

        quote! {
            .children({
                let levels = self.fields.#field_state_ident.read(cx).levels();
                levels.into_iter().map(|level| {
                    field()
                        .label(level.label().clone())
                        .description_fn({
                            let description = level.description().clone();
                            move |_, _| {
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(div().child(description.clone()))
                            }
                        })
                        .child(Select::new(&level.select()))
                })
            })
        }
    }

    fn generate_focusable_cycle(
        &self,
        field: &ResolvedField<'_>,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        Some(generate_entity_focus(field))
    }

    fn generate_subscription(
        &self,
        field: &ResolvedField<'_>,
        _component: &GpuiFormShape,
    ) -> Option<GeneratedSubscription> {
        let field_type = field.value_type();
        let field_var_name_ident = field.field_ident_with_behaviour();
        let event_handler_fn_name_ident = field.event_handler_ident("infinite_select_event");
        let field_name_ident = field.field_ident();

        let searchable = if let ComponentsBehaviour::InfiniteSelect(config) = field.behaviour() {
            config.searchable
        } else {
            panic!("Expected InfiniteSelect behaviour")
        };

        let state_type = if searchable {
            quote! { SearchableInfiniteSelectState<#field_type> }
        } else {
            quote! { InfiniteSelectState<#field_type> }
        };

        let calls = vec![
            quote! { cx.subscribe_in(&#field_var_name_ident, window, Self::#event_handler_fn_name_ident) },
        ];

        let handler = quote! {
            fn #event_handler_fn_name_ident(
                &mut self,
                _this: &Entity<#state_type>,
                event: &InfiniteSelectEvent<#field_type>,
                _window: &mut Window,
                _cx: &mut Context<Self>,
            ) {
                self.current_data.#field_name_ident = event.value().clone();
            }
        };

        Some(GeneratedSubscription {
            calls,
            handlers: vec![handler],
        })
    }
}

fn infinite_select_options(field: &ResolvedField<'_>) -> TokenStream {
    let behaviour = match field.behaviour() {
        ComponentsBehaviour::InfiniteSelect(config) => config,
        _ => panic!("Expected InfiniteSelect behaviour"),
    };

    if let Some(max_depth) = behaviour.max_depth {
        let searchable = behaviour.searchable;
        quote! {
            gpui_form::infinite_select::InfiniteSelectStateOptions::default()
                .searchable(#searchable)
                .max_depth(#max_depth)
        }
    } else {
        let searchable = behaviour.searchable;
        quote! {
            gpui_form::infinite_select::InfiniteSelectStateOptions::default()
                .searchable(#searchable)
        }
    }
}
