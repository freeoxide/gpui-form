use gpui::{AppContext as _, Context, Entity, IntoElement, ParentElement as _, Styled, Window};
use gpui_component::input::{Input, InputState};

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

/// UI component that renders the external tags input list.
#[derive(IntoElement)]
pub struct ExternalTagsInput {
    state: Entity<ExternalTagInputsState>,
}

impl ExternalTagsInput {
    pub fn new(state: &Entity<ExternalTagInputsState>) -> Self {
        Self {
            state: state.clone(),
        }
    }
}

impl gpui::RenderOnce for ExternalTagsInput {
    fn render(self, _: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let inputs = self.state.read(cx).inputs.clone();
        gpui::div()
            .flex()
            .flex_col()
            .gap_1()
            .children(inputs.iter().map(Input::new))
    }
}
