use gpui::{AppContext as _, Context, Entity, Window};
use gpui_component::input::InputState;

/// External custom state for demonstrating `component(custom(shape = ...))`
/// with a declarative shape macro from another crate.
#[derive(Clone, Debug)]
pub struct ExternalTagInputsState {
    pub inputs: Vec<Entity<InputState>>,
}

impl ExternalTagInputsState {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let inputs = vec![
            cx.new(|cx| InputState::new(window, cx)),
            cx.new(|cx| InputState::new(window, cx)),
            cx.new(|cx| InputState::new(window, cx)),
            cx.new(|cx| InputState::new(window, cx)),
        ];
        Self { inputs }
    }
}
