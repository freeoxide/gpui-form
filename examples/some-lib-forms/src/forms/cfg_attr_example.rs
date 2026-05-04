use es_fluent::FluentMessage;
use gpui::prelude::FluentBuilder as _;
use gpui::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ParentElement as _, Render, Styled, Subscription, Window, div,
};
use gpui_component::checkbox::Checkbox;
use gpui_component::form::{field, v_form};
use gpui_component::input::{
    Input, InputEvent, InputState, NumberInput, NumberInputEvent, StepAction,
};
use gpui_component::select::{Select, SelectEvent, SelectState};
use gpui_component::separator::Separator;
use gpui_component::switch::Switch;
use gpui_component::{ActiveTheme as _, Disableable as _, v_flex};
use gpui_form::runtime::date_picker::{DatePicker, DatePickerEvent, DatePickerState};
use rust_decimal::Decimal;
use some_lib::structs::cfg_attr_example::*;
use some_lib::structs::form_action::FormAction;
const CONTEXT: &str = "CfgAttrExampleForm";

fn localize(message: &impl FluentMessage) -> String {
    crate::i18n::localize(message)
}

#[gpui_storybook::story_init]
pub fn init(cx: &mut App) {}
#[gpui_storybook::story]
pub struct CfgAttrExampleForm {
    current_data: CfgAttrExampleFormValueHolder,
    fields: CfgAttrExampleFormFields,
    focus_handle: FocusHandle,
    _subscriptions: Vec<Subscription>,
}
impl Focusable for CfgAttrExampleForm {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
impl gpui_storybook::Story for CfgAttrExampleForm {
    fn title() -> String {
        crate::i18n::localize_label::<CfgAttrExample>()
    }
    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        cx.new(|cx| Self::new(window, cx))
    }
}
impl CfgAttrExampleForm {
    fn on_username_input_event(
        &mut self,
        state: &Entity<InputState>,
        event: &InputEvent,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        match event {
            InputEvent::Change => {
                let text = state.read(_cx).value();
                self.current_data.username = if text.is_empty() {
                    None
                } else {
                    Some(text.to_string())
                };
            },
            _ => {},
        }
    }
    fn on_email_input_event(
        &mut self,
        state: &Entity<InputState>,
        event: &InputEvent,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        match event {
            InputEvent::Change => {
                let text = state.read(_cx).value();
                self.current_data.email = if text.is_empty() {
                    None
                } else {
                    Some(text.to_string())
                };
            },
            _ => {},
        }
    }
    fn on_age_input_event(
        &mut self,
        state: &Entity<InputState>,
        event: &InputEvent,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        match event {
            InputEvent::Change => {
                let text = state.read(_cx).value();
                self.current_data.age = text.parse::<Age>().ok();
            },
            _ => {},
        }
    }
    fn on_age_number_input_event(
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
                        .age
                        .unwrap_or_default()
                        .saturating_sub(1u8.into());
                    self.current_data.age = Some(new_value.into());
                    this.update(cx, |input, cx| {
                        input.set_value(new_value.to_string(), window, cx);
                    });
                },
                StepAction::Increment => {
                    let new_value = self
                        .current_data
                        .age
                        .unwrap_or_default()
                        .saturating_add(1u8.into());
                    self.current_data.age = Some(new_value.into());
                    this.update(cx, |input, cx| {
                        input.set_value(new_value.to_string(), window, cx);
                    });
                },
            },
        }
    }
    fn on_balance_input_event(
        &mut self,
        state: &Entity<InputState>,
        event: &InputEvent,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        match event {
            InputEvent::Change => {
                let text = state.read(_cx).value();
                self.current_data.balance = text.parse::<Decimal>().ok();
            },
            _ => {},
        }
    }
    fn on_balance_number_input_event(
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
                        .balance
                        .unwrap_or_default()
                        .saturating_sub(1u8.into());
                    self.current_data.balance = Some(new_value.into());
                    this.update(cx, |input, cx| {
                        input.set_value(new_value.to_string(), window, cx);
                    });
                },
                StepAction::Increment => {
                    let new_value = self
                        .current_data
                        .balance
                        .unwrap_or_default()
                        .saturating_add(1u8.into());
                    self.current_data.balance = Some(new_value.into());
                    this.update(cx, |input, cx| {
                        input.set_value(new_value.to_string(), window, cx);
                    });
                },
            },
        }
    }
    fn on_account_type_select_event(
        &mut self,
        _this: &Entity<SelectState<Vec<AccountType>>>,
        event: &SelectEvent<Vec<AccountType>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        match event {
            SelectEvent::Confirm(value) => {
                if let Some(value) = value {
                    self.current_data.account_type = value.clone();
                }
            },
        }
    }
    fn on_created_at_date_picker_event(
        &mut self,
        _this: &Entity<DatePickerState>,
        event: &DatePickerEvent,
        _: &mut Window,
        _: &mut Context<Self>,
    ) {
        match event {
            DatePickerEvent::Change(date) => {
                self.current_data.created_at =
                    date.and_then(gpui_form::runtime::date_picker::parse_form_date);
            },
        }
    }
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let current_data = CfgAttrExampleFormValueHolder::default();
        let username_input = cx.new(|cx| CfgAttrExampleFormComponents::username_input(window, cx));
        let email_input = cx.new(|cx| CfgAttrExampleFormComponents::email_input(window, cx));
        let age_number_input =
            cx.new(|cx| CfgAttrExampleFormComponents::age_number_input(window, cx));
        let balance_number_input =
            cx.new(|cx| CfgAttrExampleFormComponents::balance_number_input(window, cx));
        let account_type_select =
            cx.new(|cx| CfgAttrExampleFormComponents::account_type_select(window, cx));
        let created_at_date_picker =
            cx.new(|cx| CfgAttrExampleFormComponents::created_at_date_picker(window, cx));
        let mut _subscriptions = vec![
            cx.subscribe_in(&username_input, window, Self::on_username_input_event),
            cx.subscribe_in(&email_input, window, Self::on_email_input_event),
            cx.subscribe_in(&age_number_input, window, Self::on_age_input_event),
            cx.subscribe_in(&age_number_input, window, Self::on_age_number_input_event),
            cx.subscribe_in(&balance_number_input, window, Self::on_balance_input_event),
            cx.subscribe_in(
                &balance_number_input,
                window,
                Self::on_balance_number_input_event,
            ),
            cx.subscribe_in(
                &account_type_select,
                window,
                Self::on_account_type_select_event,
            ),
            cx.subscribe_in(
                &created_at_date_picker,
                window,
                Self::on_created_at_date_picker_event,
            ),
        ];
        if let Some(value) = current_data.username.as_ref() {
            username_input.update(cx, |state, cx| {
                state.set_value(value.to_string(), window, cx);
            });
        }
        if let Some(value) = current_data.email.as_ref() {
            email_input.update(cx, |state, cx| {
                state.set_value(value.to_string(), window, cx);
            });
        }
        if let Some(value) = current_data.age.as_ref() {
            age_number_input.update(cx, |state, cx| {
                state.set_value(value.to_string(), window, cx);
            });
        }
        if let Some(value) = current_data.balance.as_ref() {
            balance_number_input.update(cx, |state, cx| {
                state.set_value(value.to_string(), window, cx);
            });
        }
        Self {
            current_data,
            fields: CfgAttrExampleFormFields {
                username_input,
                email_input,
                age_number_input,
                balance_number_input,
                account_type_select,
                created_at_date_picker,
            },
            focus_handle: cx.focus_handle(),
            _subscriptions,
        }
    }
    fn reset_form(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        *self = Self::new(window, cx);
        cx.notify();
    }
    fn submit_payload(&self) -> Result<CfgAttrExampleFormValueHolder, String> {
        match self.current_data.validate() {
            Ok(_) => Ok(self.current_data.clone()),
            Err(error) => Err(format!("{error:?}")),
        }
    }
    fn submit_button(
        &self,
        cx: &mut Context<Self>,
        label: impl Into<gpui::SharedString>,
        on_submit: impl Fn(
            Result<CfgAttrExampleFormValueHolder, String>,
            &mut Window,
            &mut Context<Self>,
        ) + 'static,
    ) -> gpui_component::button::Button {
        gpui_component::button::Button::new(format!("{}-submit-button", "cfg_attr_example-form"))
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
        gpui_component::button::Button::new(format!("{}-reset-button", "cfg_attr_example-form"))
            .label(label)
            .on_click(cx.listener(|this, _, window, cx| {
                this.reset_form(window, cx);
            }))
    }
    fn action_buttons(
        &self,
        cx: &mut Context<Self>,
        on_submit: impl Fn(
            Result<CfgAttrExampleFormValueHolder, String>,
            &mut Window,
            &mut Context<Self>,
        ) + 'static,
    ) -> impl IntoElement {
        div()
            .flex()
            .gap_2()
            .child(self.submit_button(cx, localize(&FormAction::Submit), on_submit))
            .child(self.reset_button(cx, localize(&FormAction::Reset)))
    }
}
impl Render for CfgAttrExampleForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let validation_errors = self.current_data.validate().err();
        v_flex()
            .key_context(CONTEXT)
            .id("cfg_attr_example-form")
            .size_full()
            .p_4()
            .justify_start()
            .gap_3()
            .child(Separator::horizontal())
            .child(
                v_form()
                    .child(
                        field()
                            .label(localize(&CfgAttrExampleLabelVariants::Username))
                            .description_fn({
                                let description =
                                    localize(&CfgAttrExampleDescriptionVariants::Username);
                                let error = {
                                    validation_errors.as_ref().and_then(|e| {
                                        let errs = e.username().all();
                                        if errs.is_empty() {
                                            None
                                        } else {
                                            Some(
                                                errs.iter()
                                                    .map(|v| localize(v))
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
                            .child(Input::new(&self.fields.username_input)),
                    )
                    .child(
                        field()
                            .label(localize(&CfgAttrExampleLabelVariants::Email))
                            .description_fn({
                                let description =
                                    localize(&CfgAttrExampleDescriptionVariants::Email);
                                let error = {
                                    validation_errors.as_ref().and_then(|e| {
                                        let errs = e.email().all();
                                        if errs.is_empty() {
                                            None
                                        } else {
                                            Some(
                                                errs.iter()
                                                    .map(|v| localize(v))
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
                            .child(Input::new(&self.fields.email_input)),
                    )
                    .child(
                        field()
                            .label(localize(&CfgAttrExampleLabelVariants::Age))
                            .description_fn({
                                let description = localize(&CfgAttrExampleDescriptionVariants::Age);
                                let error = {
                                    validation_errors.as_ref().and_then(|e| {
                                        e.age().and_then(|inner_error| {
                                            let errs = inner_error.all();
                                            if errs.is_empty() {
                                                None
                                            } else {
                                                Some(
                                                    errs.iter()
                                                        .map(|v| localize(v))
                                                        .collect::<Vec<_>>()
                                                        .join("\n"),
                                                )
                                            }
                                        })
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
                            .child(NumberInput::new(&self.fields.age_number_input)),
                    )
                    .child(
                        field()
                            .label(localize(&CfgAttrExampleLabelVariants::Balance))
                            .description_fn({
                                let description =
                                    localize(&CfgAttrExampleDescriptionVariants::Balance);
                                let error = {
                                    validation_errors.as_ref().and_then(|e| {
                                        let errs = e.balance().all();
                                        if errs.is_empty() {
                                            None
                                        } else {
                                            Some(
                                                errs.iter()
                                                    .map(|v| localize(v))
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
                            .child(NumberInput::new(&self.fields.balance_number_input)),
                    )
                    .child(
                        field()
                            .label(localize(&CfgAttrExampleLabelVariants::Active))
                            .description_fn({
                                let description =
                                    localize(&CfgAttrExampleDescriptionVariants::Active);
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                }
                            })
                            .child(
                                Checkbox::new("active-checkbox")
                                    .checked(self.current_data.active)
                                    .on_click(cx.listener(|v, _, _, _| {
                                        v.current_data.active = !v.current_data.active;
                                    })),
                            ),
                    )
                    .child(
                        field()
                            .label(localize(&CfgAttrExampleLabelVariants::Enabled))
                            .description_fn({
                                let description =
                                    localize(&CfgAttrExampleDescriptionVariants::Enabled);
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                }
                            })
                            .child(
                                Switch::new("enabled-switch")
                                    .checked(self.current_data.enabled)
                                    .on_click(cx.listener(move |v, checked, _, cx| {
                                        v.current_data.enabled = *checked;
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(
                        field()
                            .label(localize(&CfgAttrExampleLabelVariants::AccountType))
                            .description_fn({
                                let description =
                                    localize(&CfgAttrExampleDescriptionVariants::AccountType);
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                }
                            })
                            .child(Select::new(&self.fields.account_type_select)),
                    )
                    .child(
                        field()
                            .label(localize(&CfgAttrExampleLabelVariants::CreatedAt))
                            .description_fn({
                                let description =
                                    localize(&CfgAttrExampleDescriptionVariants::CreatedAt);
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                }
                            })
                            .child(DatePicker::new(&self.fields.created_at_date_picker)),
                    )
                    .child(field().label_indent(false).child(self.action_buttons(
                        cx,
                        |payload, _, _| {
                            let _ = payload;
                        },
                    ))),
            )
            .child(Separator::horizontal())
            .child(format!("value_holder: {:?}", self.current_data))
            .child(format!(
                "into_original: incomplete; present_fields_json: {}",
                self.current_data.present_fields_json()
            ))
    }
}
