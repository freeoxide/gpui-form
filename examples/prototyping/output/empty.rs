use ::some_lib::structs::empty::*;
use ::gpui::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement,
    IntoElement, ParentElement as _, Render, Styled, Window,
};
use ::gpui_kit::component::Disableable as _;
use ::gpui_kit::component::separator::Separator;
use ::gpui_kit::component::form::v_form;
use ::gpui_kit::component::v_flex;
const CONTEXT: &str = "EmptyForm";
fn localize(cx: &impl ::std::borrow::Borrow<App>, key: &str) -> String {
    crate::i18n::localize_message(cx, key)
}
#[::gpui_storybook::story_init]
pub fn init(_cx: &mut App) {}
#[::gpui_storybook::story]
pub struct EmptyForm {
    fields: EmptyFormFields,
    focus_handle: FocusHandle,
}
impl Focusable for EmptyForm {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
impl ::gpui_storybook::Story for EmptyForm {
    fn title(cx: &::gpui::App) -> String {
        crate::i18n::localize_label(cx, "empty_label")
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
            .child(Separator::horizontal())
            .child(v_form())
            .child(Separator::horizontal())
    }
}
