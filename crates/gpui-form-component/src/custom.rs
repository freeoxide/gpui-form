//! Runtime contract for custom components used by `#[derive(GpuiForm)]`:
//! implement [`crate::custom::CustomComponentShape`] (preferably via
//! [`crate::custom_component_shape!`]) so the derive can wire your component.

/// Shape contract for user-defined components.
///
/// Implementations provide the component state type and how to construct it.
pub trait CustomComponentShape {
    /// Backing gpui component state type.
    type State: 'static;

    /// Build the component state.
    fn new(window: &mut gpui::Window, cx: &mut gpui::Context<'_, Self::State>) -> Self::State;

    /// Optional UI component type path (e.g. `"TagsInput"`), settable via
    /// [`crate::custom_component_shape!`] `component = …` or
    /// `#[gpui_form_custom(component = …)]`; a field-level `component = …` wins.
    const COMPONENT_PATH: Option<&'static str> = None;
}

/// Value update emitted by a custom component adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CustomComponentValueChange<T> {
    /// The component event did not change the form value.
    Unchanged,
    /// Replace the form value with the supplied value.
    Set(T),
    /// Clear an optional form value.
    Clear,
}

impl<T> CustomComponentValueChange<T> {
    pub const fn set(value: T) -> Self {
        Self::Set(value)
    }

    pub const fn clear() -> Self {
        Self::Clear
    }

    pub const fn unchanged() -> Self {
        Self::Unchanged
    }
}

/// Value-binding contract for custom components, opted into with
/// `component(custom(..., value_binding))`. Both hooks are mandatory: a
/// defaulted no-op would silently drop the form value.
pub trait CustomComponentValueAdapter<T>: CustomComponentShape {
    /// Event emitted by the custom component state.
    type Event: 'static;

    /// Seeds component state from the current form value on every value
    /// change or rebuild; intentionally ignoring the value must be an
    /// explicit empty body.
    fn set_state_value(
        _state: &mut Self::State,
        _value: Option<&T>,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<'_, Self::State>,
    );

    /// Convert a component event into a form value update.
    fn value_change(state: &Self::State, event: &Self::Event) -> CustomComponentValueChange<T>;
}

/// Defines a custom component shape with an optional UI component path:
/// `custom_component_shape!(VIS Name, state = T, new = f[, component = P])`.
#[macro_export]
macro_rules! custom_component_shape {
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

#[cfg(test)]
mod tests {
    use super::CustomComponentShape;

    pub(crate) struct TestState;

    fn new_test_state(_: &mut gpui::Window, _: &mut gpui::Context<'_, TestState>) -> TestState {
        TestState
    }

    crate::custom_component_shape!(
        pub(crate) PlainTestShape,
        state = TestState,
        new = new_test_state,
    );

    crate::custom_component_shape!(
        pub(crate) ComponentTestShape,
        state = TestState,
        new = new_test_state,
        component = TestState,
    );

    #[test]
    fn custom_component_shape_macro_generates_shape_impls() {
        fn assert_shape<S: CustomComponentShape>() {}

        assert_shape::<PlainTestShape>();
        assert_shape::<ComponentTestShape>();
        assert_eq!(
            <PlainTestShape as CustomComponentShape>::COMPONENT_PATH,
            None
        );
        assert_eq!(
            <ComponentTestShape as CustomComponentShape>::COMPONENT_PATH,
            Some("TestState")
        );
    }
}
