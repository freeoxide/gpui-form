//! Reproduction of the gpui-form bug with `#[gpui_form(koruma)]` and `RequiredValidation`
//!
//! ## The Bug
//!
//! When using `#[gpui_form(koruma)]` on a struct with non-Optional fields, the generated
//! `FormValueHolder` should get `RequiredValidation` added for non-optional fields
//! that get wrapped in `Option`.
//!
//! The issue is that non-Optional fields (like `String`) become `Option<String>` in the
//! value holder, but they don't automatically get `RequiredValidation::<Option<_>>` added,
//! which means they can be `None` when they shouldn't be.

use es_fluent::EsFluent;
use es_fluent_lang::es_fluent_language;
use gpui::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ParentElement as _, Render, Styled, Subscription, Window, div, prelude::FluentBuilder as _,
};
use gpui_component::{
    ActiveTheme as _,
    divider::Divider,
    form::{field, v_form},
    input::{Input, InputEvent, InputState},
    v_flex,
};
use gpui_form::GpuiForm;
use gpui_storybook::{Assets, Gallery};
use koruma::Koruma;
use strum::EnumIter;

/// A form struct demonstrating the bug.
///
/// The `username` and `nickname` fields are non-optional (`String`) but become
/// `Option<String>` in the generated `ReproFormFormValueHolder`. The bug is that
/// they don't get `RequiredValidation::<Option<_>>` added automatically.
#[derive(Clone, Debug, Default, GpuiForm, Koruma)]
#[gpui_form(koruma)]
pub struct ReproForm {
    /// This field is NOT Optional (it's a plain String).
    ///
    /// When converted to ReproFormFormValueHolder, it becomes `Option<String>`.
    /// BUG: RequiredValidation is NOT added automatically for this field.
    #[gpui_form(component(input))]
    pub username: String,

    /// This field IS Optional.
    /// This works correctly - the value holder keeps it as Option<String>.
    #[gpui_form(component(input))]
    pub email: Option<String>,

    /// A non-optional field without any validation - demonstrates the bug clearly.
    /// BUG: RequiredValidation is NOT added automatically for this field.
    #[gpui_form(component(input))]
    pub nickname: String,
}

const CONTEXT: &str = "ReproForm";

#[gpui_storybook::story_init]
pub fn init(_cx: &mut App) {}

#[gpui_storybook::story]
pub struct ReproFormStory {
    original_data: ReproForm,
    current_data: ReproFormFormValueHolder,
    fields: ReproFormFormFields,
    focus_handle: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl Focusable for ReproFormStory {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl gpui_storybook::Story for ReproFormStory {
    fn title() -> String {
        "Repro Form".to_string()
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx, ReproForm::default())
    }
}

impl ReproFormStory {
    pub fn view(
        window: &mut Window,
        cx: &mut App,
        original_data: ReproForm,
    ) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx, original_data))
    }

    fn on_username_input_event(
        &mut self,
        _subscriber: Entity<InputState>,
        _event: &InputEvent,
        cx: &mut Context<Self>,
    ) {
        let text = self.fields.username_input.read(cx).value();
        self.current_data.username = if text.is_empty() {
            None
        } else {
            Some(text.to_string())
        };
    }

    fn on_email_input_event(
        &mut self,
        _subscriber: Entity<InputState>,
        _event: &InputEvent,
        cx: &mut Context<Self>,
    ) {
        let text = self.fields.email_input.read(cx).value();
        self.current_data.email = if text.is_empty() {
            None
        } else {
            Some(text.to_string())
        };
    }

    fn on_nickname_input_event(
        &mut self,
        _subscriber: Entity<InputState>,
        _event: &InputEvent,
        cx: &mut Context<Self>,
    ) {
        let text = self.fields.nickname_input.read(cx).value();
        self.current_data.nickname = if text.is_empty() {
            None
        } else {
            Some(text.to_string())
        };
    }

    fn new(window: &mut Window, cx: &mut Context<Self>, original_data: ReproForm) -> Self {
        let username_input = cx.new(|cx| ReproFormFormComponents::username_input(window, cx));
        let email_input = cx.new(|cx| ReproFormFormComponents::email_input(window, cx));
        let nickname_input = cx.new(|cx| ReproFormFormComponents::nickname_input(window, cx));

        let _subscriptions = vec![
            cx.subscribe(&username_input, Self::on_username_input_event),
            cx.subscribe(&email_input, Self::on_email_input_event),
            cx.subscribe(&nickname_input, Self::on_nickname_input_event),
        ];

        Self {
            original_data: original_data.clone(),
            current_data: original_data.into(),
            fields: ReproFormFormFields {
                username_input,
                email_input,
                nickname_input,
            },
            focus_handle: cx.focus_handle(),
            _subscriptions,
        }
    }
}

impl Render for ReproFormStory {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let validation_errors = self.current_data.validate().err();

        v_flex()
            .key_context(CONTEXT)
            .id("repro-form")
            .size_full()
            .p_4()
            .justify_start()
            .gap_3()
            .child(Divider::horizontal())
            .child(
                v_form()
                    .child(
                        field()
                            .label("Username (required String)")
                            .description_fn({
                                let description =
                                    "This is a non-optional String field. It should have RequiredValidation.";
                                let error = validation_errors.as_ref().and_then(|e| {
                                    let errs = e.username().all();
                                    if errs.is_empty() {
                                        None
                                    } else {
                                        Some(
                                            errs.iter()
                                                .map(|v| format!("{:?}", v))
                                                .collect::<Vec<_>>()
                                                .join("\n"),
                                        )
                                    }
                                });
                                let error_color = cx.theme().danger;
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.to_string()))
                                        .when(error.is_some(), |this| {
                                            this.child(
                                                div()
                                                    .text_color(error_color)
                                                    .child(error.clone().unwrap_or_default()),
                                            )
                                        })
                                }
                            })
                            .child(Input::new(&self.fields.username_input)),
                    )
                    .child(
                        field()
                            .label("Email (optional)")
                            .description_fn({
                                let description =
                                    "This is an Option<String> field. No RequiredValidation needed.";
                                let error: Option<String> = None;
                                let error_color = cx.theme().danger;
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.to_string()))
                                        .when(error.is_some(), |this| {
                                            this.child(
                                                div()
                                                    .text_color(error_color)
                                                    .child(error.clone().unwrap_or_default()),
                                            )
                                        })
                                }
                            })
                            .child(Input::new(&self.fields.email_input)),
                    )
                    .child(
                        field()
                            .label("Nickname (required String)")
                            .description_fn({
                                let description = "This is a non-optional String field. It should have RequiredValidation.";
                                let error = validation_errors.as_ref().and_then(|e| {
                                    let errs = e.nickname().all();
                                    if errs.is_empty() {
                                        None
                                    } else {
                                        Some(
                                            errs.iter()
                                                .map(|v| format!("{:?}", v))
                                                .collect::<Vec<_>>()
                                                .join("\n"),
                                        )
                                    }
                                });
                                let error_color = cx.theme().danger;
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.to_string()))
                                        .when(error.is_some(), |this| {
                                            this.child(
                                                div()
                                                    .text_color(error_color)
                                                    .child(error.clone().unwrap_or_default()),
                                            )
                                        })
                                }
                            })
                            .child(Input::new(&self.fields.nickname_input)),
                    ),
            )
            .child(Divider::horizontal())
            .child(format!("{:?}", self.current_data))
    }
}

#[es_fluent_language]
#[derive(Clone, Copy, Debug, EnumIter, EsFluent, PartialEq)]
pub enum Languages {}

fn main() {
    let app = gpui::Application::new().with_assets(Assets);
    let name_arg = std::env::args().nth(1);

    app.run(move |app_cx| {
        gpui_component::init(app_cx);
        gpui_storybook::init(Languages::default(), app_cx);
        gpui_storybook::change_locale(Languages::default());
        app_cx.activate(true);

        gpui_storybook::create_new_window(
            &format!("{} - Stories", env!("CARGO_PKG_NAME")),
            move |window, cx| {
                let all_stories = gpui_storybook::generate_stories(window, cx);
                Gallery::view(all_stories, name_arg.as_deref(), window, cx)
            },
            app_cx,
        );
    });
}
