use gpui_form_schema::components::{ComponentsBehaviour, NumberInputKind};
use gpui_form_schema::registry::{FieldVariant, GpuiFormShape};
use proc_macro2::TokenStream;
use quote::quote;

use crate::imports::ImportItem;

use super::{
    FieldCodeGenerator, GeneratedSubscription, ResolvedField, generate_entity_creation,
    generate_entity_field_initializer, generate_entity_focus, generate_text_value_prefill,
    render_component_entity_field,
};

pub struct NumberInputCodeGenerator;

const IMPORTS: &[ImportItem] = &[
    ImportItem::path("gpui_component::input::InputEvent"),
    ImportItem::path("gpui_component::input::InputState"),
    ImportItem::path("gpui_component::input::NumberInput"),
    ImportItem::path("gpui_component::input::NumberInputEvent"),
    ImportItem::path("gpui_component::input::StepAction"),
];

impl FieldCodeGenerator for NumberInputCodeGenerator {
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

        let on_input_event_handler_fn_name_ident = field.event_handler_ident("input_event");
        let on_number_input_event_handler_fn_name_ident =
            field.event_handler_ident("number_input_event");

        let calls = vec![
            quote! { cx.subscribe_in(&#field_var_name_ident, window, Self::#on_input_event_handler_fn_name_ident) },
            quote! { cx.subscribe_in(&#field_var_name_ident, window, Self::#on_number_input_event_handler_fn_name_ident) },
        ];

        let mut handlers = vec![];

        let field_name_ident = field.field_ident();

        let field_type_path = field.value_type();

        let on_input_event_handler = quote! {
            fn #on_input_event_handler_fn_name_ident(
                &mut self,
                state: &Entity<InputState>,
                event: &InputEvent,
                _window: &mut Window,
                _cx: &mut Context<Self>,
            ) {
                match event {
                    InputEvent::Change => {
                        let text = state.read(_cx).value();
                        self.current_data.#field_name_ident = text.parse::<#field_type_path>().ok();
                    }
                    _ => {}
                }
            }
        };
        handlers.push(on_input_event_handler);

        // Generate increment/decrement logic - value holder always wraps numeric fields in Option
        let behaviour = match field.behaviour() {
            ComponentsBehaviour::NumberInput(behaviour) => behaviour,
            _ => panic!("Expected NumberInput behaviour"),
        };

        let (decrement_logic, increment_logic) = match behaviour.kind {
            NumberInputKind::Float => (
                quote! {
                    let new_value = self.current_data.#field_name_ident.unwrap_or_default() - 1.0;
                    self.current_data.#field_name_ident = Some(new_value);
                },
                quote! {
                    let new_value = self.current_data.#field_name_ident.unwrap_or_default() + 1.0;
                    self.current_data.#field_name_ident = Some(new_value);
                },
            ),
            NumberInputKind::SignedInteger | NumberInputKind::UnsignedInteger => (
                quote! {
                    let new_value = self.current_data.#field_name_ident.unwrap_or_default().saturating_sub(1);
                    self.current_data.#field_name_ident = Some(new_value);
                },
                quote! {
                    let new_value = self.current_data.#field_name_ident.unwrap_or_default().saturating_add(1);
                    self.current_data.#field_name_ident = Some(new_value);
                },
            ),
            NumberInputKind::Custom => (
                quote! {
                    let new_value = self.current_data.#field_name_ident.unwrap_or_default().saturating_sub(1u8.into());
                    self.current_data.#field_name_ident = Some(new_value.into());
                },
                quote! {
                    let new_value = self.current_data.#field_name_ident.unwrap_or_default().saturating_add(1u8.into());
                    self.current_data.#field_name_ident = Some(new_value.into());
                },
            ),
        };

        let on_number_input_event_handler = quote! {
            fn #on_number_input_event_handler_fn_name_ident(
                &mut self,
                this: &Entity<InputState>,
                event: &NumberInputEvent,
                window: &mut Window,
                cx: &mut Context<Self>,
            ) {
                match event {
                    NumberInputEvent::Step(step_action) => match step_action {
                        StepAction::Decrement => {
                            #decrement_logic
                            this.update(cx, |input, cx| {
                                input.set_value(new_value.to_string(), window, cx);
                            });
                        }
                        StepAction::Increment => {
                            #increment_logic
                            this.update(cx, |input, cx| {
                                input.set_value(new_value.to_string(), window, cx);
                            });
                        }
                    },
                }
            }
        };
        handlers.push(on_number_input_event_handler);

        Some(GeneratedSubscription { calls, handlers })
    }
}

#[cfg(test)]
mod tests {
    use super::NumberInputCodeGenerator;
    use crate::implementations::FieldCodeGenerator as _;
    use gpui_form_schema::{
        components::{ComponentsBehaviour, NumberInputBehaviour, NumberInputKind},
        registry::{FieldVariant, GpuiFormShape},
    };

    fn compact(input: &str) -> String {
        input.chars().filter(|c| !c.is_whitespace()).collect()
    }

    #[test]
    fn number_input_generator_emits_match_based_change_handler() {
        const FIELDS: [FieldVariant; 1] = [FieldVariant::new(
            "age",
            "u32",
            false,
            ComponentsBehaviour::NumberInput(NumberInputBehaviour {
                kind: NumberInputKind::UnsignedInteger,
                validation_type: None,
            }),
        )];
        const SHAPE: GpuiFormShape = GpuiFormShape::new("Demo", &FIELDS, "src/demo.rs", false);

        let generator = NumberInputCodeGenerator;
        let field = crate::implementations::ResolvedField::new(&FIELDS[0]).unwrap();
        let generated = generator
            .generate_subscription(&field, &SHAPE)
            .expect("number input fields should generate subscriptions");
        let compact = compact(&generated.handlers[0].to_string());

        assert!(
            compact.contains("matchevent{InputEvent::Change=>"),
            "number input text-change handlers should keep explicit match arms: {compact}"
        );
        assert!(
            compact.contains("_=>{}"),
            "number input text-change handlers should include a noop fallback arm: {compact}"
        );
        assert!(
            !compact.contains("ifletInputEvent::Change=event"),
            "number input text-change handlers should not collapse to if let: {compact}"
        );
    }
}
