use some_lib::structs::empty::*;
use gpui::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement,
    IntoElement, ParentElement as _, Render, Styled, Subscription, Window, div,
    prelude::FluentBuilder as _,
};
use gpui_component::{
    ActiveTheme as _, IndexPath, checkbox::Checkbox,
    date_picker::{DatePicker, DatePickerEvent, DatePickerState},
    divider::Divider, form::{field, v_form},
    input::{Input, InputEvent, InputState, NumberInput, NumberInputEvent, StepAction},
    select::{SearchableVec, Select, SelectEvent, SelectState},
    switch::Switch, v_flex,
};
use gpui_form_component::infinite_select::InfiniteSelect;
use es_fluent::{ThisFtl as _, ToFluentString as _};
use rust_decimal::Decimal;
const CONTEXT: &str = "EmptyForm";
#[gpui_storybook::story_init]
pub fn init(cx: &mut App) {}
#[gpui_storybook::story]
pub struct EmptyForm {
    fields: EmptyFormFields,
    focus_handle: FocusHandle,
}
impl Focusable for EmptyForm {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
impl gpui_storybook::Story for EmptyForm {
    fn title() -> String {
        Empty::this_ftl()
    }
    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        cx.new(|cx| Self::new(window, cx))
    }
}
impl EmptyForm {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let current_data = EmptyFormValueHolder::default();
        Self {
            current_data,
            fields: EmptyFormFields,
            focus_handle: cx.focus_handle(),
        }
    }
}
impl Render for EmptyForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .key_context(CONTEXT)
            .id("empty-form")
            .size_full()
            .p_4()
            .justify_start()
            .gap_3()
            .child(Divider::horizontal())
            .child(v_form())
            .child(Divider::horizontal())
    }
}
