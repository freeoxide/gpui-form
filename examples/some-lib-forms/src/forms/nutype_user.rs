use es_fluent::ToFluentString as _;
use gpui::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ParentElement as _, Render, Styled, Subscription, Window, div, prelude::FluentBuilder as _,
};
use gpui_component::{
    checkbox::Checkbox,
    date_picker::{DatePicker, DatePickerEvent, DatePickerState},
    divider::Divider,
    form::{field, v_form},
    input::{Input, InputEvent, InputState, NumberInput, NumberInputEvent, StepAction},
    select::{SearchableVec, Select, SelectEvent, SelectState},
    switch::Switch,
    v_flex,
};
use rust_decimal::Decimal;
use some_lib::structs::nutype_user::*;
use std::sync::Arc;
#[derive(Clone, Debug, es_fluent::EsFluent)]
pub enum NutypeUserFormErrorsFtl {
    Username { value: String },
    Email { value: String },
    Age { value: String },
    Balance { value: String },
    SubscribeNewsletter { value: String },
    EnableNotifications { value: String },
    Preferred { value: String },
    Country { value: String },
    BirthDate { value: String },
}
const CONTEXT: &str = "NutypeUserForm";
#[gpui_storybook::story_init]
pub fn init(cx: &mut App) {}
#[gpui_storybook::story]
pub struct NutypeUserForm {
    original_data: Arc<NutypeUser>,
    current_data: NutypeUserFormValueHolder,
    errors: NutypeUserFormErrors,
    fields: NutypeUserFormFields,
    focus_handle: FocusHandle,
    _subscriptions: Vec<Subscription>,
}
impl Focusable for NutypeUserForm {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
impl gpui_storybook::Story for NutypeUserForm {
    fn title() -> String {
        NutypeUser::this_ftl()
    }
    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx, NutypeUser::default())
    }
}
impl NutypeUserForm {
    pub fn view(window: &mut Window, cx: &mut App, original_data: NutypeUser) -> Entity<Self> {
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
                if let Ok(parsed_value) = text.parse::<String>() {
                    if let Ok(validated) = Username::try_new(parsed_value) {
                        self.current_data.username = validated.into();
                        self.errors.username.clear();
                    }
                }
            },
            InputEvent::Blur => {
                let text = state.read(_cx).value();
                if let Ok(parsed_value) = text.parse::<String>() {
                    match Username::try_new(parsed_value) {
                        Ok(validated_value) => {
                            self.current_data.username = validated_value.into();
                            self.errors.username.clear();
                        },
                        Err(e) => {
                            self.errors.username = NutypeUserFormErrorsFtl::Username {
                                value: format!("{:?}", e),
                            }
                            .to_fluent_string();
                        },
                    }
                } else {
                    self.errors.username = NutypeUserFormErrorsFtl::Username {
                        value: format!("Invalid {} format", stringify!(String)),
                    }
                    .to_fluent_string();
                }
                _cx.notify();
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
                if let Ok(parsed_value) = text.parse::<String>() {
                    if let Ok(validated) = Email::try_new(parsed_value) {
                        self.current_data.email = validated.into();
                        self.errors.email.clear();
                    }
                }
            },
            InputEvent::Blur => {
                let text = state.read(_cx).value();
                if let Ok(parsed_value) = text.parse::<String>() {
                    match Email::try_new(parsed_value) {
                        Ok(validated_value) => {
                            self.current_data.email = validated_value.into();
                            self.errors.email.clear();
                        },
                        Err(e) => {
                            self.errors.email = NutypeUserFormErrorsFtl::Email {
                                value: format!("{:?}", e),
                            }
                            .to_fluent_string();
                        },
                    }
                } else {
                    self.errors.email = NutypeUserFormErrorsFtl::Email {
                        value: format!("Invalid {} format", stringify!(String)),
                    }
                    .to_fluent_string();
                }
                _cx.notify();
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
                if let Ok(parsed_value) = text.parse::<u8>() {
                    if let Ok(validated) = Age::try_new(parsed_value) {
                        self.current_data.age = validated.into();
                        self.errors.age.clear();
                    }
                }
            },
            InputEvent::Blur => {
                let text = state.read(_cx).value();
                if let Ok(parsed_value) = text.parse::<u8>() {
                    match Age::try_new(parsed_value) {
                        Ok(validated_value) => {
                            self.current_data.age = validated_value.into();
                            self.errors.age.clear();
                        },
                        Err(e) => {
                            self.errors.age = NutypeUserFormErrorsFtl::Age {
                                value: format!("{:?}", e),
                            }
                            .to_fluent_string();
                        },
                    }
                } else {
                    self.errors.age = NutypeUserFormErrorsFtl::Age {
                        value: format!("Invalid {} format", stringify!(u8)),
                    }
                    .to_fluent_string();
                }
                _cx.notify();
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
                    let new_value = self.current_data.age.saturating_sub(1);
                    if let Ok(validated) = Age::try_new(new_value) {
                        self.current_data.age = validated;
                        self.errors.age.clear();
                    }
                    this.update(cx, |input, cx| {
                        input.set_value(self.current_data.age.to_string(), window, cx);
                    });
                },
                StepAction::Increment => {
                    let new_value = self.current_data.age.saturating_add(1);
                    if let Ok(validated) = Age::try_new(new_value) {
                        self.current_data.age = validated;
                        self.errors.age.clear();
                    }
                    this.update(cx, |input, cx| {
                        input.set_value(self.current_data.age.to_string(), window, cx);
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
                if let Ok(parsed_value) = text.parse::<Decimal>() {
                    if let Ok(validated) = Balance::try_new(parsed_value) {
                        self.current_data.balance = validated.into();
                        self.errors.balance.clear();
                    }
                }
            },
            InputEvent::Blur => {
                let text = state.read(_cx).value();
                if let Ok(parsed_value) = text.parse::<Decimal>() {
                    match Balance::try_new(parsed_value) {
                        Ok(validated_value) => {
                            self.current_data.balance = validated_value.into();
                            self.errors.balance.clear();
                        },
                        Err(e) => {
                            self.errors.balance = NutypeUserFormErrorsFtl::Balance {
                                value: format!("{:?}", e),
                            }
                            .to_fluent_string();
                        },
                    }
                } else {
                    self.errors.balance = NutypeUserFormErrorsFtl::Balance {
                        value: format!("Invalid {} format", stringify!(Decimal)),
                    }
                    .to_fluent_string();
                }
                _cx.notify();
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
                    let new_value = self.current_data.balance.saturating_sub(Decimal::from(1));
                    if let Ok(validated) = Balance::try_new(new_value) {
                        self.current_data.balance = validated;
                        self.errors.balance.clear();
                    }
                    this.update(cx, |input, cx| {
                        input.set_value(self.current_data.balance.to_string(), window, cx);
                    });
                },
                StepAction::Increment => {
                    let new_value = self.current_data.balance.saturating_add(Decimal::from(1));
                    if let Ok(validated) = Balance::try_new(new_value) {
                        self.current_data.balance = validated;
                        self.errors.balance.clear();
                    }
                    this.update(cx, |input, cx| {
                        input.set_value(self.current_data.balance.to_string(), window, cx);
                    });
                },
            },
        }
    }
    fn on_preferred_select_event(
        &mut self,
        _this: &Entity<SelectState<Vec<NutypePreferedLanguage>>>,
        event: &SelectEvent<Vec<NutypePreferedLanguage>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        match event {
            SelectEvent::Confirm(value) => {
                if let Some(value) = value {
                    self.current_data.preferred = value.clone().into();
                }
            },
        }
    }
    fn on_country_select_event(
        &mut self,
        _this: &Entity<SelectState<SearchableVec<NutypeEnumCountry>>>,
        event: &SelectEvent<SearchableVec<NutypeEnumCountry>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        match event {
            SelectEvent::Confirm(value) => {
                if let Some(value) = value {
                    self.current_data.country = value.clone().into();
                }
            },
        }
    }
    fn on_birth_date_date_picker_event(
        &mut self,
        _this: &Entity<DatePickerState>,
        event: &DatePickerEvent,
        _: &mut Window,
        _: &mut Context<Self>,
    ) {
        match event {
            DatePickerEvent::Change(date) => {
                self.current_data.birth_date =
                    (<chrono::NaiveDate as std::str::FromStr>::from_str(&date.to_string())).ok();
            },
        }
    }
    fn new(window: &mut Window, cx: &mut Context<Self>, original_data: NutypeUser) -> Self {
        let username_input = cx.new(|cx| NutypeUserFormComponents::username_input(window, cx));
        let email_input = cx.new(|cx| NutypeUserFormComponents::email_input(window, cx));
        let age_number_input = cx.new(|cx| NutypeUserFormComponents::age_number_input(window, cx));
        let balance_number_input =
            cx.new(|cx| NutypeUserFormComponents::balance_number_input(window, cx));
        let preferred_select = cx.new(|cx| NutypeUserFormComponents::preferred_select(window, cx));
        let country_select = cx.new(|cx| NutypeUserFormComponents::country_select(window, cx));
        let birth_date_date_picker =
            cx.new(|cx| NutypeUserFormComponents::birth_date_date_picker(window, cx));
        let _subscriptions = vec![
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
            cx.subscribe_in(&preferred_select, window, Self::on_preferred_select_event),
            cx.subscribe_in(&country_select, window, Self::on_country_select_event),
            cx.subscribe_in(
                &birth_date_date_picker,
                window,
                Self::on_birth_date_date_picker_event,
            ),
        ];
        Self {
            original_data: Arc::new(original_data.clone()),
            current_data: original_data.into(),
            errors: NutypeUserFormErrors::default(),
            fields: NutypeUserFormFields {
                username_input,
                email_input,
                age_number_input,
                balance_number_input,
                preferred_select,
                country_select,
                birth_date_date_picker,
            },
            focus_handle: cx.focus_handle(),
            _subscriptions,
        }
    }
}
impl Render for NutypeUserForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .key_context(CONTEXT)
            .id("nutype_user-form")
            .size_full()
            .p_4()
            .justify_start()
            .gap_3()
            .child(Divider::horizontal())
            .child(
                v_form()
                    .child(
                        field()
                            .label(NutypeUserLabelKvFtl::Username.to_fluent_string())
                            .description_fn({
                                let error = self.errors.username.clone();
                                let description =
                                    NutypeUserDescriptionKvFtl::Username.to_fluent_string();
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                        .when(!error.is_empty(), |this| {
                                            this.child(
                                                div().text_color(gpui::red()).child(error.clone()),
                                            )
                                        })
                                }
                            })
                            .child(Input::new(&self.fields.username_input)),
                    )
                    .child(
                        field()
                            .label(NutypeUserLabelKvFtl::Email.to_fluent_string())
                            .description_fn({
                                let error = self.errors.email.clone();
                                let description =
                                    NutypeUserDescriptionKvFtl::Email.to_fluent_string();
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                        .when(!error.is_empty(), |this| {
                                            this.child(
                                                div().text_color(gpui::red()).child(error.clone()),
                                            )
                                        })
                                }
                            })
                            .child(Input::new(&self.fields.email_input)),
                    )
                    .child(
                        field()
                            .label(NutypeUserLabelKvFtl::Age.to_fluent_string())
                            .description_fn({
                                let error = self.errors.age.clone();
                                let description =
                                    NutypeUserDescriptionKvFtl::Age.to_fluent_string();
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                        .when(!error.is_empty(), |this| {
                                            this.child(
                                                div().text_color(gpui::red()).child(error.clone()),
                                            )
                                        })
                                }
                            })
                            .child(NumberInput::new(&self.fields.age_number_input)),
                    )
                    .child(
                        field()
                            .label(NutypeUserLabelKvFtl::Balance.to_fluent_string())
                            .description_fn({
                                let error = self.errors.balance.clone();
                                let description =
                                    NutypeUserDescriptionKvFtl::Balance.to_fluent_string();
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                        .when(!error.is_empty(), |this| {
                                            this.child(
                                                div().text_color(gpui::red()).child(error.clone()),
                                            )
                                        })
                                }
                            })
                            .child(NumberInput::new(&self.fields.balance_number_input)),
                    )
                    .child(
                        field()
                            .label(NutypeUserLabelKvFtl::SubscribeNewsletter.to_fluent_string())
                            .description_fn({
                                let error = self.errors.subscribe_newsletter.clone();
                                let description = NutypeUserDescriptionKvFtl::SubscribeNewsletter
                                    .to_fluent_string();
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                        .when(!error.is_empty(), |this| {
                                            this.child(
                                                div().text_color(gpui::red()).child(error.clone()),
                                            )
                                        })
                                }
                            })
                            .child(
                                Checkbox::new("subscribe-newsletter-checkbox")
                                    .checked(self.current_data.subscribe_newsletter)
                                    .on_click(cx.listener(|v, _, _, _| {
                                        v.current_data.subscribe_newsletter =
                                            !v.current_data.subscribe_newsletter;
                                    })),
                            ),
                    )
                    .child(
                        field()
                            .label(NutypeUserLabelKvFtl::EnableNotifications.to_fluent_string())
                            .description_fn({
                                let error = self.errors.enable_notifications.clone();
                                let description = NutypeUserDescriptionKvFtl::EnableNotifications
                                    .to_fluent_string();
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                        .when(!error.is_empty(), |this| {
                                            this.child(
                                                div().text_color(gpui::red()).child(error.clone()),
                                            )
                                        })
                                }
                            })
                            .child(
                                Switch::new("enable-notifications-switch")
                                    .checked(self.current_data.enable_notifications)
                                    .on_click(cx.listener(move |v, checked, _, cx| {
                                        v.current_data.enable_notifications = *checked;
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(
                        field()
                            .label(NutypeUserLabelKvFtl::Preferred.to_fluent_string())
                            .description_fn({
                                let error = self.errors.preferred.clone();
                                let description =
                                    NutypeUserDescriptionKvFtl::Preferred.to_fluent_string();
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                        .when(!error.is_empty(), |this| {
                                            this.child(
                                                div().text_color(gpui::red()).child(error.clone()),
                                            )
                                        })
                                }
                            })
                            .child(Select::new(&self.fields.preferred_select)),
                    )
                    .child(
                        field()
                            .label(NutypeUserLabelKvFtl::Country.to_fluent_string())
                            .description_fn({
                                let error = self.errors.country.clone();
                                let description =
                                    NutypeUserDescriptionKvFtl::Country.to_fluent_string();
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                        .when(!error.is_empty(), |this| {
                                            this.child(
                                                div().text_color(gpui::red()).child(error.clone()),
                                            )
                                        })
                                }
                            })
                            .child(Select::new(&self.fields.country_select)),
                    )
                    .child(
                        field()
                            .label(NutypeUserLabelKvFtl::BirthDate.to_fluent_string())
                            .description_fn({
                                let error = self.errors.birth_date.clone();
                                let description =
                                    NutypeUserDescriptionKvFtl::BirthDate.to_fluent_string();
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                        .when(!error.is_empty(), |this| {
                                            this.child(
                                                div().text_color(gpui::red()).child(error.clone()),
                                            )
                                        })
                                }
                            })
                            .child(DatePicker::new(&self.fields.birth_date_date_picker)),
                    ),
            )
            .child(Divider::horizontal())
            .absolute()
            .child(format!("{:?}", self.current_data))
    }
}
