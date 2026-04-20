use gpui_form_schema::registry::{FieldVariant, GpuiFormShape};
use proc_macro2::TokenStream;
use quote::quote;

use crate::imports::ImportItem;

use super::{
    FieldCodeGenerator, GeneratedSubscription, ResolvedField, generate_entity_creation,
    generate_entity_field_initializer, generate_entity_focus, generate_text_value_prefill,
    render_component_entity_field,
};

pub struct InputCodeGenerator;

const IMPORTS: &[ImportItem] = &[
    ImportItem::path("gpui_component::input::Input"),
    ImportItem::path("gpui_component::input::InputEvent"),
    ImportItem::path("gpui_component::input::InputState"),
];

impl FieldCodeGenerator for InputCodeGenerator {
    fn generate_imports(&self, _field: &FieldVariant) -> Vec<ImportItem> {
        IMPORTS.to_vec()
    }

    fn generate_cx_new_call(
        &self,
        field: &ResolvedField<'_>,
        component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        Some(generate_entity_creation(field, component))
    }

    fn generate_post_subscription_initialization(
        &self,
        field: &ResolvedField<'_>,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        Some(generate_text_value_prefill(field))
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
        component: &GpuiFormShape,
    ) -> TokenStream {
        render_component_entity_field(field, component)
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
        let field_var_name_ident = field.field_ident_with_behaviour();
        let event_handler_fn_name_ident = field.event_handler_ident("input_event");

        let calls = vec![
            quote! { cx.subscribe_in(&#field_var_name_ident, window, Self::#event_handler_fn_name_ident) },
        ];

        let field_name_ident = field.field_ident();

        let handler = quote! {
            fn #event_handler_fn_name_ident(
                &mut self,
                state: &Entity<InputState>,
                event: &InputEvent,
                _window: &mut Window,
                _cx: &mut Context<Self>,
            ) {
                match event {
                    InputEvent::Change => {
                        let text = state.read(_cx).value();
                        self.current_data.#field_name_ident = if text.is_empty() {
                            None
                        } else {
                            Some(text.to_string())
                        };
                    }
                    _ => {}
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
    use super::InputCodeGenerator;
    use crate::implementations::FieldCodeGenerator as _;
    use gpui_form_schema::{
        components::ComponentsBehaviour,
        registry::{FieldVariant, GpuiFormShape},
    };

    fn compact(input: &str) -> String {
        input.chars().filter(|c| !c.is_whitespace()).collect()
    }

    #[test]
    fn input_generator_emits_match_based_change_handler() {
        const FIELDS: [FieldVariant; 1] = [FieldVariant::new(
            "name",
            "String",
            false,
            ComponentsBehaviour::Input,
        )];
        const SHAPE: GpuiFormShape = GpuiFormShape::new("Demo", &FIELDS, "src/demo.rs", false);

        let generator = InputCodeGenerator;
        let field = crate::implementations::ResolvedField::new(&FIELDS[0]).unwrap();
        let generated = generator
            .generate_subscription(&field, &SHAPE)
            .expect("input fields should generate subscriptions");
        let compact = compact(&generated.handlers[0].to_string());

        assert!(
            compact.contains("matchevent{InputEvent::Change=>"),
            "input handlers should keep explicit match arms for prototyping readability: {compact}"
        );
        assert!(
            compact.contains("_=>{}"),
            "input handlers should include a noop fallback arm: {compact}"
        );
        assert!(
            !compact.contains("ifletInputEvent::Change=event"),
            "input handlers should not collapse to if let: {compact}"
        );
    }
}
