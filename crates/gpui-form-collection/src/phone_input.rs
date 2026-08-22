//! Phone-number text input backed by libphonenumber validation.
//!
//! The shape stores the raw text (like [`crate::input::Input`]); syntax
//! validation happens at the widget level so a partially typed or cleared
//! field is never flagged mid-entry. Structured validation — required-ness,
//! country matching, typed [`gpui_form_core::phone::ValidatedPhoneNumber`]
//! conversion — belongs to the validation layer via the helpers in
//! `gpui_form_core::phone`.

use component_shape::ValueChange;
use component_shape_gpui::{GpuiComponentValueBinding, component_shape};
use gpui::{Context, Window};
use gpui_component::input::{InputEvent, InputState};

component_shape! {
    /// Form component for a phone-number `gpui_component::input::Input`.
    ///
    /// Widget-level validation accepts text that libphonenumber can parse
    /// globally (no country binding) or that is empty; required-ness and
    /// country matching are separate validation-layer concerns.
    pub struct PhoneInput {
        state = InputState;
        new = |window, cx| InputState::new(window, cx)
            .validate(|value, _| {
                gpui_form_core::phone::validate_optional_phone_number(value, None)
                    .is_valid_or_empty()
            });
        component = gpui_component::input::Input;
        value = String;
        field_suffix = "phone_input";
        value_binding;

        impl GpuiComponentValueBinding<String> for PhoneInput {
            type Event = InputEvent;

            fn seed_value_binding_state(
                state: &mut Self::State,
                value: Option<&String>,
                window: &mut Window,
                cx: &mut Context<'_, Self::State>,
            ) {
                state.set_value(value.cloned().unwrap_or_default(), window, cx);
            }

            fn value_change(state: &Self::State, event: &Self::Event) -> ValueChange<String> {
                match event {
                    InputEvent::Change => {
                        let value = state.value();
                        if value.is_empty() {
                            ValueChange::Clear
                        } else {
                            // Store the raw text: syntax validation already ran
                            // at the widget level, and typed conversion is a
                            // validation-layer concern.
                            ValueChange::Set(value.to_string())
                        }
                    },
                    _ => ValueChange::Unchanged,
                }
            }
        }
    }
}

impl_form_component_shape!(PhoneInput, gpui_form_runtime::shape::DirectValueStorage);

#[cfg(test)]
mod tests {
    use super::PhoneInput;
    use component_shape::ValueChange;
    use component_shape_gpui::{GpuiComponentShape, GpuiComponentValueBinding};
    use gpui_component::input::{InputEvent, InputState};
    use gpui_form_runtime::shape::{
        DirectValueStorage, GpuiComponentShapeFor, GpuiFormComponentShapePolicy,
    };

    fn assert_shape_contract<Shape>()
    where
        Shape: component_shape_gpui::DeclaredGpuiComponentShape
            + GpuiComponentShape<State = InputState>
            + GpuiComponentShapeFor<String>
            + GpuiFormComponentShapePolicy<ValueStoragePolicy = DirectValueStorage>
            + GpuiComponentValueBinding<String, Event = InputEvent>,
    {
    }

    #[test]
    fn phone_input_publishes_shape_contracts() {
        assert_shape_contract::<PhoneInput>();
    }

    #[test]
    fn phone_input_accepts_globally_parseable_numbers() {
        assert!(
            gpui_form_core::phone::validate_optional_phone_number("+1 415 550 2222", None)
                .is_valid()
        );
        assert!(
            !gpui_form_core::phone::validate_optional_phone_number("not-a-phone", None).is_valid()
        );
        // Empty input is explicitly allowed mid-entry.
        assert!(gpui_form_core::phone::validate_optional_phone_number("", None).is_empty());
    }
}
