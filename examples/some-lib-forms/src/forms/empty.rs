use es_fluent::FluentMessage;
use gpui::prelude::FluentBuilder as _;
use gpui::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ParentElement as _, Render, Styled, Subscription, Window, div,
};
use gpui_component::divider::Divider;
use gpui_component::form::{field, v_form};
use gpui_component::{ActiveTheme as _, Disableable as _, v_flex};
use rust_decimal::Decimal;
use some_lib::structs::empty::*;
const CONTEXT: &str = "EmptyForm";

fn localize(cx: &impl std::borrow::Borrow<App>, message: &impl FluentMessage) -> String {
    crate::i18n::localize_message(cx, message)
}

#[gpui_storybook::story_init]
pub fn init(_cx: &mut App) {}
#[gpui_storybook::story]
pub struct EmptyForm {
    fields: EmptyFormFields,
    focus_handle: FocusHandle,
}
impl Focusable for EmptyForm {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
impl gpui_storybook::Story for EmptyForm {
    fn title(cx: &gpui::App) -> String {
        crate::i18n::localize_label::<Empty>(cx)
    }
    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        cx.new(|cx| Self::new(window, cx))
    }
}
impl EmptyForm {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
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
