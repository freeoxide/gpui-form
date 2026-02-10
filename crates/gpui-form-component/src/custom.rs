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
/// );
/// ```
#[macro_export]
macro_rules! custom_component_shape {
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
