use es_fluent::{ThisFtl as _, ToFluentString as _};
use gpui::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ParentElement as _, Render, Styled, Subscription, Window, div, prelude::FluentBuilder as _,
};
use gpui_component::{
    ActiveTheme as _, IndexPath,
    checkbox::Checkbox,
    date_picker::{DatePicker, DatePickerEvent, DatePickerState},
    divider::Divider,
    form::{field, v_form},
    input::{Input, InputEvent, InputState, NumberInput, NumberInputEvent, StepAction},
    select::{SearchableVec, Select, SelectEvent, SelectState},
    switch::Switch,
    v_flex,
};
use gpui_form::component::infinite_select::InfiniteSelect;
use rust_decimal::Decimal;
use some_lib::structs::cfg_attr_example::*;
use std::sync::Arc;
const CONTEXT: &str = "CfgAttrExampleForm";
#[gpui_storybook::story_init]
pub fn init(cx: &mut App) {}
#[gpui_storybook::story]
pub struct CfgAttrExampleForm {
    original_data: Arc<CfgAttrExample>,
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
        CfgAttrExample::this_ftl()
    }
    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx, CfgAttrExampleFormValueHolder::default().into())
    }
}
impl CfgAttrExampleForm {
    pub fn view(window: &mut Window, cx: &mut App, original_data: CfgAttrExample) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx, original_data))
    }
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
                    (<chrono::NaiveDate as std::str::FromStr>::from_str(&date.to_string())).ok();
            },
        }
    }
    fn new(window: &mut Window, cx: &mut Context<Self>, original_data: CfgAttrExample) -> Self {
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
        Self {
            original_data: Arc::new(original_data.clone()),
            current_data: original_data.into(),
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
            .child(Divider::horizontal())
            .child(
                v_form()
                    .child(
                        field()
                            .label(CfgAttrExampleLabelVariants::Username.to_fluent_string())
                            .description_fn({
                                let description =
                                    CfgAttrExampleDescriptionVariants::Username.to_fluent_string();
                                let error = {
                                    validation_errors.as_ref().and_then(|e| {
                                        let errs = e.username().all();
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
                            .child(Input::new(&self.fields.username_input)),
                    )
                    .child(
                        field()
                            .label(CfgAttrExampleLabelVariants::Email.to_fluent_string())
                            .description_fn({
                                let description =
                                    CfgAttrExampleDescriptionVariants::Email.to_fluent_string();
                                let error = {
                                    validation_errors.as_ref().and_then(|e| {
                                        let errs = e.email().all();
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
                            .child(Input::new(&self.fields.email_input)),
                    )
                    .child(
                        field()
                            .label(CfgAttrExampleLabelVariants::Age.to_fluent_string())
                            .description_fn({
                                let description =
                                    CfgAttrExampleDescriptionVariants::Age.to_fluent_string();
                                let error = {
                                    validation_errors.as_ref().and_then(|e| {
                                        let errs = e.age().all();
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
                            .child(NumberInput::new(&self.fields.age_number_input)),
                    )
                    .child(
                        field()
                            .label(CfgAttrExampleLabelVariants::Balance.to_fluent_string())
                            .description_fn({
                                let description =
                                    CfgAttrExampleDescriptionVariants::Balance.to_fluent_string();
                                let error = {
                                    validation_errors.as_ref().and_then(|e| {
                                        let errs = e.balance().all();
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
                            .child(NumberInput::new(&self.fields.balance_number_input)),
                    )
                    .child(
                        field()
                            .label(CfgAttrExampleLabelVariants::Active.to_fluent_string())
                            .description_fn({
                                let description =
                                    CfgAttrExampleDescriptionVariants::Active.to_fluent_string();
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
                            .label(CfgAttrExampleLabelVariants::Enabled.to_fluent_string())
                            .description_fn({
                                let description =
                                    CfgAttrExampleDescriptionVariants::Enabled.to_fluent_string();
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
                            .label(CfgAttrExampleLabelVariants::AccountType.to_fluent_string())
                            .description_fn({
                                let description = CfgAttrExampleDescriptionVariants::AccountType
                                    .to_fluent_string();
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
                            .label(CfgAttrExampleLabelVariants::CreatedAt.to_fluent_string())
                            .description_fn({
                                let description =
                                    CfgAttrExampleDescriptionVariants::CreatedAt.to_fluent_string();
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                }
                            })
                            .child(DatePicker::new(&self.fields.created_at_date_picker)),
                    ),
            )
            .child(Divider::horizontal())
            .child(format!("{:?}", self.current_data))
    }
}
