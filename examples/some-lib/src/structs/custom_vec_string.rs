use es_fluent::{EsFluentThis, EsFluentVariants};
use gpui::{AppContext as _, Context, Entity, IntoElement, ParentElement as _, Styled, Window};
use gpui_component::input::{Input, InputState};
use gpui_form::{CustomComponentState, GpuiForm};

/// State for the tags input custom component.
#[derive(Clone, CustomComponentState, Debug)]
#[gpui_form_custom(new = Self::new, component = TagsInput)]
pub struct TagsInputState {
    pub inputs: Vec<Entity<InputState>>,
}

impl TagsInputState {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let inputs = vec![
            cx.new(|cx| InputState::new(window, cx)),
            cx.new(|cx| InputState::new(window, cx)),
            cx.new(|cx| InputState::new(window, cx)),
        ];
        Self { inputs }
    }
}

/// UI component that renders the tags input list.
#[derive(IntoElement)]
pub struct TagsInput {
    state: Entity<TagsInputState>,
}

impl TagsInput {
    pub fn new(state: &Entity<TagsInputState>) -> Self {
        Self {
            state: state.clone(),
        }
    }
}

impl gpui::RenderOnce for TagsInput {
    fn render(self, _: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let inputs = self.state.read(cx).inputs.clone();
        gpui::div()
            .flex()
            .flex_col()
            .gap_1()
            .children(inputs.iter().map(Input::new))
    }
}

#[derive(Clone, Debug, Default, GpuiForm, EsFluentThis, EsFluentVariants)]
#[fluent_this(origin, members)]
#[fluent_variants(keys = ["description", "label"])]
pub struct VecStringInputList {
    #[gpui_form(component(custom(shape = TagsInputState, wraps_in_option = false)))]
    pub tags: Vec<String>,
}
