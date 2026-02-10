use gpui::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ParentElement as _, Render, Styled, Subscription, Window, div,
};
use gpui_component::{
    divider::Divider,
    form::{field, v_form},
    input::{Input, InputEvent, InputState},
    v_flex,
};
use some_lib::structs::custom_vec_string_external::*;

const CONTEXT: &str = "ExternalShapeVecStringInputListForm";

#[gpui_storybook::story_init]
pub fn init(_cx: &mut App) {}

#[gpui_storybook::story]
pub struct ExternalShapeVecStringInputListForm {
    current_data: ExternalShapeVecStringInputListFormValueHolder,
    fields: ExternalShapeVecStringInputListFormFields,
    focus_handle: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl Focusable for ExternalShapeVecStringInputListForm {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl gpui_storybook::Story for ExternalShapeVecStringInputListForm {
    fn title() -> String {
        "External Declarative Custom Shape".to_string()
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        cx.new(|cx| Self::new(window, cx))
    }
}

impl ExternalShapeVecStringInputListForm {
    fn on_tags_input_event(
        &mut self,
        _state: &Entity<InputState>,
        event: &InputEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            InputEvent::Change => {
                self.current_data.tags = self
                    .fields
                    .tags_custom
                    .read(cx)
                    .inputs
                    .iter()
                    .filter_map(|input| {
                        let value = input.read(cx).value();
                        if value.is_empty() {
                            None
                        } else {
                            Some(value.to_string())
                        }
                    })
                    .collect();
            },
            _ => {},
        }
    }

    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let current_data = ExternalShapeVecStringInputListFormValueHolder::default();
        let tags_custom =
            cx.new(|cx| ExternalShapeVecStringInputListFormComponents::tags_custom(window, cx));
        let tag_inputs = tags_custom.read(cx).inputs.clone();

        let mut _subscriptions = Vec::new();
        for input in &tag_inputs {
            _subscriptions.push(cx.subscribe_in(input, window, Self::on_tags_input_event));
        }

        for (input, value) in tag_inputs.iter().zip(current_data.tags.iter()) {
            input.update(cx, |state, cx| {
                state.set_value(value.to_string(), window, cx);
            });
        }

        Self {
            current_data,
            fields: ExternalShapeVecStringInputListFormFields { tags_custom },
            focus_handle: cx.focus_handle(),
            _subscriptions,
        }
    }
}

impl Render for ExternalShapeVecStringInputListForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tag_inputs = self.fields.tags_custom.read(cx).inputs.clone();

        v_flex()
            .key_context(CONTEXT)
            .id("external-shape-vec-string-input-list-form")
            .size_full()
            .p_4()
            .justify_start()
            .gap_3()
            .child(Divider::horizontal())
            .child(
                v_form().child(
                    field()
                        .label("External Tags")
                        .description_fn({
                            move |_, _| {
                                div().flex().flex_col().gap_1().child(
                                    div().child("custom_component_shape! defined in another crate"),
                                )
                            }
                        })
                        .children(tag_inputs.iter().map(Input::new)),
                ),
            )
            .child(Divider::horizontal())
            .child(format!("{:?}", self.current_data))
    }
}
