use gpui::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ParentElement as _, Render, Styled, Subscription, Window, div,
};
use gpui_component::button::Button;
use gpui_component::form::{field, v_form};
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::separator::Separator;
use gpui_component::v_flex;
use gpui_form::FormState;
use some_lib::structs::user::{UserFormPath, UserFormValueHolder};

const CONTEXT: &str = "FeatureAuditForm";

#[gpui_storybook::story_init]
pub fn init(_cx: &mut App) {}

#[gpui_storybook::story]
pub struct FeatureAuditForm {
    form_state: FormState<UserFormValueHolder>,
    username_input: Entity<InputState>,
    focus_handle: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl Focusable for FeatureAuditForm {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl gpui_storybook::Story for FeatureAuditForm {
    fn title(_cx: &gpui::App) -> String {
        "Feature Audit".into()
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        cx.new(|cx| Self::new(window, cx))
    }
}

impl FeatureAuditForm {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let holder = UserFormValueHolder::default();
        let username_input = cx.new(|cx| {
            let mut input = InputState::new(window, cx);
            if let Some(username) = &holder.username {
                input.set_value(username.clone(), window, cx);
            }
            input
        });

        let _subscriptions =
            vec![cx.subscribe_in(&username_input, window, Self::on_username_input_event)];

        Self {
            form_state: FormState::new(holder),
            username_input,
            focus_handle: cx.focus_handle(),
            _subscriptions,
        }
    }

    fn on_username_input_event(
        &mut self,
        state: &Entity<InputState>,
        event: &InputEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if matches!(event, InputEvent::Change) {
            let value = state.read(cx).value();
            self.form_state.current_mut().username = if value.is_empty() {
                None
            } else {
                Some(value.to_string())
            };
            cx.notify();
        }
    }

    fn sync_username_input(&self, window: &mut Window, cx: &mut Context<Self>) {
        let value = self
            .form_state
            .current()
            .username
            .clone()
            .unwrap_or_default();
        self.username_input
            .update(cx, |input, cx| input.set_value(value, window, cx));
    }

    fn reset_button(&self, cx: &mut Context<Self>) -> Button {
        Button::new("feature-audit-reset-button")
            .label("Reset to baseline")
            .on_click(cx.listener(|this, _, window, cx| {
                this.form_state.reset_to_baseline();
                this.sync_username_input(window, cx);
                cx.notify();
            }))
    }

    fn mark_clean_button(&self, cx: &mut Context<Self>) -> Button {
        Button::new("feature-audit-mark-clean-button")
            .label("Mark current clean")
            .on_click(cx.listener(|this, _, _window, cx| {
                this.form_state.sync_baseline();
                cx.notify();
            }))
    }
}

impl Render for FeatureAuditForm {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let current = self.form_state.current();
        let baseline = self.form_state.baseline();
        let paths = [
            UserFormPath::username(),
            UserFormPath::email(),
            UserFormPath::age(),
            UserFormPath::balance(),
            UserFormPath::debt(),
            UserFormPath::country(),
            UserFormPath::birth_date(),
        ]
        .into_iter()
        .map(|path| path.to_string())
        .collect::<Vec<_>>()
        .join(", ");

        v_flex()
            .key_context(CONTEXT)
            .id("feature-audit-form")
            .size_full()
            .p_4()
            .justify_start()
            .gap_3()
            .child(Separator::horizontal())
            .child(
                v_form()
                    .child(
                        field()
                            .label("FormState dirty tracking")
                            .description(
                                "Type below: dirty flips true. Reset restores baseline. Mark current clean snapshots the current value.",
                            )
                            .child(Input::new(&self.username_input)),
                    )
                    .child(
                        field().label("State flags").child(
                            v_flex()
                                .gap_1()
                                .child(format!("is_dirty: {}", self.form_state.is_dirty()))
                                .child(format!(
                                    "diff_against(default): {}",
                                    self.form_state
                                        .diff_against(&UserFormValueHolder::default())
                                )),
                        ),
                    )
                    .child(
                        field().label("Actions").child(
                            div()
                                .flex()
                                .flex_row()
                                .gap_2()
                                .child(self.reset_button(cx))
                                .child(self.mark_clean_button(cx)),
                        ),
                    )
                    .child(
                        field()
                            .label("Typed field paths")
                            .description("Generated UserFormPath constructors, rendered from the real example form.")
                            .child(paths),
                    )
                    .child(
                        field()
                            .label("Generated holder data")
                            .description("Current and baseline are real UserFormValueHolder values used by FormState.")
                            .child(
                                v_flex()
                                    .gap_1()
                                    .child(format!("current: {current:?}"))
                                    .child(format!("baseline: {baseline:?}")),
                            ),
                    )
                    .child(
                        field()
                            .label("Generated User form coverage")
                            .description(
                                "Open the User story to test layout sections and number_input(as = f64) for balance and debt.",
                            )
                            .child(
                                "The User story also renders form_state.is_dirty, field_paths, value_holder, and present_fields_json.",
                            ),
                    )
                    .child(
                        field()
                            .label("Phone verification coverage")
                            .description(
                                "Open Phone Verification to test dynamic country + phone validation through phonenumber.",
                            )
                            .child(
                                "Try US 415 555 2671, then switch to France; then try France 01 42 68 53 00 and switch to United States.",
                            ),
                    ),
            )
            .child(Separator::horizontal())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_story_uses_real_paths_and_form_state() {
        let mut form_state = FormState::new(UserFormValueHolder::default());

        assert_eq!(UserFormPath::username().to_string(), "username");
        assert_eq!(UserFormPath::balance().to_string(), "balance");
        assert!(!form_state.is_dirty());

        form_state.current_mut().username = Some("audit-user".into());
        assert!(form_state.is_dirty());

        form_state.reset_to_baseline();
        assert!(!form_state.is_dirty());
    }
}
