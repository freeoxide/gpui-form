use es_fluent::{EsFluentThis, EsFluentVariants};
use gpui::{AppContext as _, Context, Entity, Window};
use gpui_component::input::InputState;
use gpui_form::{CustomComponentState, GpuiForm};

/// Custom component used by the `Vec<String>` example.
#[derive(Clone, CustomComponentState, Debug)]
#[gpui_form_custom(new = Self::new)]
pub struct TagsInputComponent {
    pub inputs: Vec<Entity<InputState>>,
}

impl TagsInputComponent {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let inputs = vec![
            cx.new(|cx| InputState::new(window, cx)),
            cx.new(|cx| InputState::new(window, cx)),
            cx.new(|cx| InputState::new(window, cx)),
        ];
        Self { inputs }
    }
}

#[derive(Clone, Debug, Default, GpuiForm, EsFluentThis, EsFluentVariants)]
#[fluent_this(origin, members)]
#[fluent_variants(keys = ["description", "label"])]
pub struct VecStringInputList {
    #[gpui_form(component(custom(shape = TagsInputComponent, wraps_in_option = false)))]
    pub tags: Vec<String>,
}
