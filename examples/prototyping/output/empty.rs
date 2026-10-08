use ::some_lib::structs::empty::*;
use ::gpui::{
    App, Context, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ParentElement as _, Render, Styled, Window,
};
use ::gpui_kit::component::separator::Separator;
use ::gpui_kit::component::form::v_form;
use ::gpui_kit::component::v_flex;
const CONTEXT: &str = "EmptyForm";
fn localize(cx: &impl ::std::borrow::Borrow<App>, key: &str) -> String {
    crate::i18n::localize_message(cx, key)
}
pub struct EmptyForm {
    fields: EmptyFormFields,
    focus_handle: FocusHandle,
}
impl Focusable for EmptyForm {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
impl EmptyForm {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
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
            .child(v_flex().text_lg().font_semibold().child(localize(cx, "empty_label")))
            .child(Separator::horizontal())
            .child(v_form())
            .child(Separator::horizontal())
    }
}
