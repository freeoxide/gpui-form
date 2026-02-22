//! Runtime contract for custom components used by `#[derive(GpuiForm)]`.
//!
//! Users define a zero-sized "shape" type that implements [`CustomComponentShape`].
//! The derive macro uses that shape to generate:
//! - `FormFields` entity state type
//! - `FormComponents` constructor function body
//!
//! Prefer using [`custom_component_shape!`] to define shape types.

/// Shape contract for user-defined components.
///
/// Implementations provide the component state type and how to construct it.
pub trait CustomComponentShape {
    /// Backing gpui component state type.
    type State: 'static;

    /// Build the component state.
    fn new(window: &mut gpui::Window, cx: &mut gpui::Context<'_, Self::State>) -> Self::State;

    /// Optional path to the UI component type (e.g. `"TagsInput"`).
    ///
    /// When set here – via [`custom_component_shape!`] `component = …` or
    /// `#[gpui_form_custom(component = …)]` – the prototyping code generator
    /// can emit `Component::new(&entity)` without requiring `component = …`
    /// to be repeated on every field annotation.
    ///
    /// A `component = …` on the field attribute always takes precedence.
    const COMPONENT_PATH: Option<&'static str> = None;
}

/// Define a custom component shape with minimal boilerplate.
///
/// # Example
///
/// ```ignore
/// gpui_form_component::custom_component_shape!(
///     pub EmailInputShape,
///     state = gpui_component::input::InputState,
///     new = gpui_component::input::InputState::new,
///     component = gpui_component::input::Input,
/// );
/// ```
#[macro_export]
macro_rules! custom_component_shape {
    // With explicit component path
    ($vis:vis $shape:ident, state = $state:ty, new = $new:expr, component = $component:path $(,)?) => {
        $vis struct $shape;

        impl $crate::custom::CustomComponentShape for $shape {
            type State = $state;

            fn new(
                window: &mut ::gpui::Window,
                cx: &mut ::gpui::Context<'_, Self::State>,
            ) -> Self::State {
                ($new)(window, cx)
            }

            const COMPONENT_PATH: Option<&'static str> = Some(stringify!($component));
        }
    };
    // Without component path (original form)
    ($vis:vis $shape:ident, state = $state:ty, new = $new:expr $(,)?) => {
        $vis struct $shape;

        impl $crate::custom::CustomComponentShape for $shape {
            type State = $state;

            fn new(
                window: &mut ::gpui::Window,
                cx: &mut ::gpui::Context<'_, Self::State>,
            ) -> Self::State {
                ($new)(window, cx)
            }
        }
    };
}
