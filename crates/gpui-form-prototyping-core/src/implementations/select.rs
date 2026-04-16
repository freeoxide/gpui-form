use gpui_form_schema::{
    components::ComponentsBehaviour,
    registry::{FieldVariant, GpuiFormShape},
};
use proc_macro2::TokenStream;
use quote::quote;

use crate::imports::ImportItem;

use super::{
    FieldCodeGenerator, FieldVariantExt as _, GeneratedSubscription, generate_entity_creation,
    generate_entity_field_initializer, generate_entity_focus, render_component_entity_field,
};

pub struct SelectCodeGenerator;

const IMPORTS_BASE: &[ImportItem] = &[
    ImportItem::path("gpui_component::select::Select"),
    ImportItem::path("gpui_component::select::SelectEvent"),
    ImportItem::path("gpui_component::select::SelectState"),
];

impl FieldCodeGenerator for SelectCodeGenerator {
    fn generate_imports(&self, field: &FieldVariant) -> Vec<ImportItem> {
        let mut items = IMPORTS_BASE.to_vec();
        if let ComponentsBehaviour::Select(opts) = &field.behaviour
            && opts.searchable
        {
            items.push(ImportItem::path("gpui_component::select::SearchableVec"));
        }
        items
    }

    fn generate_cx_new_call(
        &self,
        field: &FieldVariant,
        component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        if field.behaviour.partial() {
            return None;
        }

        Some(generate_entity_creation(field, component))
    }

    fn generate_field_initializers(
        &self,
        field: &FieldVariant,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        Some(generate_entity_field_initializer(field))
    }

    fn generate_render_child(
        &self,
        field: &FieldVariant,
        component: &GpuiFormShape,
    ) -> TokenStream {
        render_component_entity_field(field, component)
    }

    fn generate_focusable_cycle(
        &self,
        field: &FieldVariant,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        Some(generate_entity_focus(field))
    }

    fn generate_subscription(
        &self,
        field: &FieldVariant,
        _component: &GpuiFormShape,
    ) -> Option<GeneratedSubscription> {
        let field_type = field.value_type();
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
                    _this: &Entity<SelectState<#vec_type<#field_type>>>,
                    event: &SelectEvent<#vec_type<#field_type>>,
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
                    _this: &Entity<SelectState<#vec_type<#field_type>>>,
                    event: &SelectEvent<#vec_type<#field_type>>,
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

#[cfg(test)]
mod tests {
    use super::SelectCodeGenerator;
    use crate::implementations::FieldCodeGenerator as _;
    use gpui_form_schema::{
        components::{ComponentsBehaviour, SelectBehaviour},
        registry::{FieldVariant, GpuiFormShape},
    };

    fn compact(input: &str) -> String {
        input.chars().filter(|c| !c.is_whitespace()).collect()
    }

    #[test]
    fn select_generator_keeps_qualified_type_paths() {
        const FIELDS: [FieldVariant; 1] = [FieldVariant::new(
            "country",
            "some_lib::country::Country",
            false,
            ComponentsBehaviour::Select(SelectBehaviour {
                partial: false,
                searchable: false,
            }),
        )];
        const SHAPE: GpuiFormShape = GpuiFormShape::new("Demo", &FIELDS, "src/demo.rs", false);

        let generator = SelectCodeGenerator;
        let generated = generator
            .generate_subscription(&FIELDS[0], &SHAPE)
            .expect("select fields should generate subscriptions");
        let compact = compact(&generated.handlers[0].to_string());

        assert!(
            compact.contains("Entity<SelectState<Vec<some_lib::country::Country>>>"),
            "subscription handler should keep the fully-qualified type path: {compact}"
        );
    }
}
