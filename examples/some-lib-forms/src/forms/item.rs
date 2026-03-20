use es_fluent::{ThisFtl as _, ToFluentString as _};
use gpui::prelude::FluentBuilder as _;
use gpui::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ParentElement as _, Render, Styled, Subscription, Window, div,
};
use gpui_component::divider::Divider;
use gpui_component::form::{field, v_form};
use gpui_component::input::{InputEvent, InputState, NumberInput, NumberInputEvent, StepAction};
use gpui_component::{ActiveTheme as _, Disableable as _, v_flex};
use rust_decimal::Decimal;
use some_lib::structs::new_type::*;
const CONTEXT: &str = "ItemForm";
#[gpui_storybook::story_init]
pub fn init(cx: &mut App) {}
#[gpui_storybook::story]
pub struct ItemForm {
    current_data: ItemFormValueHolder,
    fields: ItemFormFields,
    focus_handle: FocusHandle,
    _subscriptions: Vec<Subscription>,
}
impl Focusable for ItemForm {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
impl gpui_storybook::Story for ItemForm {
    fn title() -> String {
        Item::this_ftl()
    }
    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        cx.new(|cx| Self::new(window, cx))
    }
}
impl ItemForm {
    fn on_index_input_event(
        &mut self,
        state: &Entity<InputState>,
        event: &InputEvent,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        match event {
            InputEvent::Change => {
                let text = state.read(_cx).value();
                self.current_data.index = text.parse::<Age>().ok();
            },
            _ => {},
        }
    }
    fn on_index_number_input_event(
        &mut self,
        this: &Entity<InputState>,
        event: &NumberInputEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            NumberInputEvent::Step(step_action) => match step_action {
                StepAction::Decrement => {
                    let new_value = self
                        .current_data
                        .index
                        .unwrap_or_default()
                        .saturating_sub(1u8.into());
                    self.current_data.index = Some(new_value.into());
                    this.update(cx, |input, cx| {
                        input.set_value(new_value.to_string(), window, cx);
                    });
                },
                StepAction::Increment => {
                    let new_value = self
                        .current_data
                        .index
                        .unwrap_or_default()
                        .saturating_add(1u8.into());
                    self.current_data.index = Some(new_value.into());
                    this.update(cx, |input, cx| {
                        input.set_value(new_value.to_string(), window, cx);
                    });
                },
            },
        }
    }
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let current_data = ItemFormValueHolder::default();
        let index_number_input = cx.new(|cx| ItemFormComponents::index_number_input(window, cx));
        let mut _subscriptions = vec![
            cx.subscribe_in(&index_number_input, window, Self::on_index_input_event),
            cx.subscribe_in(
                &index_number_input,
                window,
                Self::on_index_number_input_event,
            ),
        ];
        if let Some(value) = current_data.index.as_ref() {
            index_number_input.update(cx, |state, cx| {
                state.set_value(value.to_string(), window, cx);
            });
        }
        Self {
            current_data,
            fields: ItemFormFields { index_number_input },
            focus_handle: cx.focus_handle(),
            _subscriptions,
        }
    }
    fn reset_form(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        *self = Self::new(window, cx);
        cx.notify();
    }
    fn submit_payload(&self) -> Result<Option<Item>, String> {
        match self.current_data.validate() {
            Ok(_) => Ok(ItemFormValueHolder::try_from(self.current_data.clone()).ok()),
            Err(error) => Err(format!("{error:?}")),
        }
    }
    fn submit_button(
        &self,
        cx: &mut Context<Self>,
        label: impl Into<gpui::SharedString>,
        on_submit: impl Fn(Result<Option<Item>, String>, &mut Window, &mut Context<Self>) + 'static,
    ) -> gpui_component::button::Button {
        gpui_component::button::Button::new(format!("{}-submit-button", "item-form"))
            .label(label)
            .disabled(self.current_data.validate().is_err())
            .on_click(cx.listener(move |this, _, window, cx| {
                on_submit(this.submit_payload(), window, cx);
            }))
    }
    fn reset_button(
        &self,
        cx: &mut Context<Self>,
        label: impl Into<gpui::SharedString>,
    ) -> gpui_component::button::Button {
        gpui_component::button::Button::new(format!("{}-reset-button", "item-form"))
            .label(label)
            .on_click(cx.listener(|this, _, window, cx| {
                this.reset_form(window, cx);
            }))
    }
    fn action_buttons(
        &self,
        cx: &mut Context<Self>,
        on_submit: impl Fn(Result<Option<Item>, String>, &mut Window, &mut Context<Self>) + 'static,
    ) -> impl IntoElement {
        div()
            .flex()
            .gap_2()
            .child(self.submit_button(cx, "Submit", on_submit))
            .child(self.reset_button(cx, "Reset"))
    }
}
impl Render for ItemForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let validation_errors = self.current_data.validate().err();
        v_flex()
            .key_context(CONTEXT)
            .id("item-form")
            .size_full()
            .p_4()
            .justify_start()
            .gap_3()
            .child(Divider::horizontal())
            .child(
                v_form()
                    .child(
                        field()
                            .label(ItemLabelVariants::Index.to_fluent_string())
                            .description_fn({
                                let description = ItemDescriptionVariants::Index.to_fluent_string();
                                let error = {
                                    validation_errors.as_ref().and_then(|e| {
                                        let errs = e.index().all();
                                        if errs.is_empty() {
                                            None
                                        } else {
                                            Some(
                                                errs.iter()
                                                    .map(|v| v.to_fluent_string())
                                                    .collect::<Vec<_>>()
                                                    .join("\n"),
                                            )
                                        }
                                    })
                                };
                                let error_color = cx.theme().danger;
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                        .when(error.is_some(), |this| {
                                            this.child(
                                                div()
                                                    .text_color(error_color)
                                                    .child(error.clone().unwrap_or_default()),
                                            )
                                        })
                                }
                            })
                            .child(NumberInput::new(&self.fields.index_number_input)),
                    )
                    .child(field().label_indent(false).child(self.action_buttons(
                        cx,
                        |payload, _, _| {
                            let _ = payload;
                        },
                    ))),
            )
            .child(Divider::horizontal())
            .child(format!("value_holder: {:?}", self.current_data))
            .child(format!(
                "into_original: {:?}",
                ItemFormValueHolder::try_from(self.current_data.clone())
            ))
    }
}
