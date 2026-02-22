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
/// Implementations provide the component state type, a UI component type,
/// and how to construct the state.
///
/// The pattern mirrors how gpui-component separates state and rendering:
/// - `State`: backing entity data (like `InputState`, `SelectState`)
/// - `Component`: UI struct with `fn new(state: &Entity<State>)` (like `Input`, `Select`)
pub trait CustomComponentShape {
    /// Backing gpui component state type.
    type State: 'static;

    /// UI component type that renders the state.
    ///
    /// Must implement `gpui::IntoElement` and provide a `new(&gpui::Entity<Self::State>)` constructor.
    type Component: gpui::IntoElement;

    /// Build the component state.
    fn new(window: &mut gpui::Window, cx: &mut gpui::Context<'_, Self::State>) -> Self::State;

    /// Build the UI component from a state entity reference.
    fn component(state: &gpui::Entity<Self::State>) -> Self::Component;
}

/// Define a custom component shape with minimal boilerplate.
///
/// # Example
///
/// ```ignore
/// gpui_form_component::custom_component_shape!(
///     pub EmailInputShape,
///     state = gpui_component::input::InputState,
///     component = gpui_component::input::Input,
///     new = gpui_component::input::InputState::new,
/// );
/// ```
#[macro_export]
macro_rules! custom_component_shape {
    ($vis:vis $shape:ident, state = $state:ty, component = $component:ty, new = $new:expr $(,)?) => {
        $vis struct $shape;

        impl $crate::custom::CustomComponentShape for $shape {
            type State = $state;
            type Component = $component;

            fn new(
                window: &mut ::gpui::Window,
                cx: &mut ::gpui::Context<'_, Self::State>,
            ) -> Self::State {
                ($new)(window, cx)
            }

            fn component(state: &::gpui::Entity<Self::State>) -> Self::Component {
                <$component>::new(state)
            }
        }
    };
}
