#![recursion_limit = "1024"]

// `gpui_kit` re-exports itself as `gpui` (0.7.3); the leading `::` keeps this
// import bound to the extern crate under the glob import below.
use ::gpui::{
    App, AppContext as _, Context, Div, Entity, FocusHandle, Focusable, InteractiveElement as _,
    IntoElement, MouseButton, ParentElement as _, Render, SharedString, Styled as _, Subscription,
    Window, div,
};
use gpui_form::runtime::date_picker::{DatePicker, DatePickerEvent, DatePickerState};
use gpui_form::runtime::file_picker::{FilePicker, FilePickerEvent, FilePickerState};
use gpui_form::{
    AsyncFieldValidation, FieldPath, GpuiForm, SelectItem, SubmitAction, SubmitTracker,
};
use gpui_kit::assets::Assets;
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Sizable as _,
    button::{Button, ButtonVariants as _},
    checkbox::Checkbox,
    form::{field, v_form},
    h_flex, v_flex, *,
};
use gpui_kit::component::{
    input::{Input, InputEvent, InputState, NumberInput, NumberInputEvent, StepAction},
    scroll::ScrollableElement as _,
    select::{SearchableVec, Select, SelectEvent, SelectState},
    switch::Switch,
};
use gpui_kit::*;
use koruma::{Koruma, KorumaAllDisplay, ValidateExt};
use koruma_collection::{
    collection::{LenValidation, NonEmptyValidation},
    format::EmailValidation,
    numeric::RangeValidation,
};
use rust_i18n::t;
use std::time::Duration;
use strum::EnumIter;
use unic_langid::LanguageIdentifier;

rust_i18n::i18n!("locales", fallback = "en");

#[derive(Clone, Debug, Default, EnumIter, PartialEq, SelectItem)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[select_item(fluent)]
pub enum Country {
    #[default]
    UnitedStates,
    France,
    China,
}

#[derive(Clone, Debug, Default, EnumIter, PartialEq, SelectItem)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[select_item(fluent)]
pub enum Language {
    #[default]
    English,
    French,
    Chinese,
}

#[derive(Clone, Debug, Default, EnumIter, PartialEq, SelectItem)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[select_item(fluent)]
pub enum UsState {
    #[default]
    California,
    NewYork,
    Texas,
}

#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    koruma::Koruma,
    koruma::KorumaAllDisplay,
    derive_more::Display,
    derive_more::From,
    derive_more::Into,
    derive_more::Deref,
    derive_more::AsRef,
    derive_more::FromStr,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[display("{value}")]
#[koruma(try_new, newtype)]
pub struct RequestCode {
    #[koruma(LenValidation::<_>::builder().min(2).max(8))]
    pub value: String,
}

impl std::fmt::Display for RequestCodeKorumaValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Clone, Debug, Default, GpuiForm, Koruma)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Address {
    #[gpui_form(component(input))]
    #[koruma(NonEmptyValidation::<_>::builder())]
    pub city: String,

    #[gpui_form(component(input))]
    pub zip: Option<String>,
}

#[derive(Clone, Debug, Default, GpuiForm, Koruma, KorumaAllDisplay)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[gpui_form(koruma(fluent), submit)]
pub struct Signup {
    #[gpui_form(label = "Username", placeholder = "ada", component(input))]
    #[koruma(NonEmptyValidation::<_>::builder())]
    pub username: String,

    #[gpui_form(
        label = "Email",
        placeholder = "you@example.com",
        component(input),
        required_if = newsletter
    )]
    #[koruma(EmailValidation::<_>::builder())]
    pub email: Option<String>,

    #[gpui_form(label = "Age", component(number_input))]
    #[koruma(RangeValidation::<_>::builder().min(1).max(130))]
    pub age: Option<u32>,

    #[gpui_form(component(select(searchable)), default = Country::France)]
    pub country: Option<Country>,

    #[gpui_form(component(select), depends_on = country)]
    pub state: Option<UsState>,

    #[gpui_form(component(select), default = Language::English)]
    pub language: Language,

    #[gpui_form(component(checkbox))]
    pub newsletter: bool,

    #[gpui_form(label = "Enable notifications", component(switch))]
    pub notifications: bool,

    #[gpui_form(label = "Invite code", placeholder = "AB12", component(input))]
    #[koruma(newtype)]
    pub code: RequestCode,

    #[gpui_form(component(date_picker))]
    pub birth_date: Option<chrono::NaiveDate>,

    #[gpui_form(component(nested))]
    #[koruma(nested)]
    pub address: Address,

    #[gpui_form(component(list(item = input)))]
    #[koruma(each(LenValidation::<_>::builder().min(2).max(32)))]
    pub tags: Vec<String>,

    #[gpui_form(skip)]
    pub session_id: u32,
}

#[derive(Clone, Debug, Default, PartialEq)]
enum EmailCheck {
    #[default]
    Idle,
    Pending,
    Failed(&'static str),
}

struct EmailAvailability;

impl AsyncFieldValidation<SignupFormValueHolder> for EmailAvailability {
    type Error = &'static str;

    async fn validate_field(
        &self,
        holder: &SignupFormValueHolder,
        field: &FieldPath,
    ) -> Result<(), Self::Error> {
        if field.segments() != [gpui_form::PathSegment::Name("email")] {
            return Ok(());
        }
        match holder.email.as_deref() {
            Some(email) if email.eq_ignore_ascii_case("taken@example.com") => {
                Err("app.email_taken")
            },
            _ => Ok(()),
        }
    }
}

struct SignupSubmit;

impl SubmitAction<SignupFormValueHolder> for SignupSubmit {
    type Output = ();
    type Error = <SignupFormValueHolder as ValidateExt>::Error;

    async fn submit(&mut self, holder: &SignupFormValueHolder) -> Result<(), Self::Error> {
        match holder.validate() {
            Ok(()) => Ok(()),
            Err(errors) => Err(errors),
        }
    }
}

struct FormsDemo {
    form: SignupFormState,
    errors_shown: bool,
    submission: SignupFormSubmission,
    email_check: EmailCheck,
    email_check_generation: u64,
    fields: SignupFormFields,
    address_city_input: Entity<InputState>,
    address_zip_input: Entity<InputState>,
    new_tag_input: Entity<InputState>,
    standalone_picker: Entity<DatePickerState>,
    file_picker: Entity<FilePickerState>,
    picked_date: Option<SharedString>,
    status: Option<SharedString>,
    show_schema: bool,
    focus_handle: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl FormsDemo {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut form = SignupFormState::new(SignupFormValueHolder::default());
        form.validate();
        let username_input = cx.new(|cx| SignupFormComponents::username_input(window, cx));
        let email_input = cx.new(|cx| SignupFormComponents::email_input(window, cx));
        let age_number_input = cx.new(|cx| SignupFormComponents::age_number_input(window, cx));
        let country_select = cx.new(|cx| SignupFormComponents::country_select(window, cx));
        let state_select = cx.new(|cx| SignupFormComponents::state_select(window, cx));
        let language_select = cx.new(|cx| SignupFormComponents::language_select(window, cx));
        let code_input = cx.new(|cx| SignupFormComponents::code_input(window, cx));
        let birth_date_date_picker =
            cx.new(|cx| SignupFormComponents::birth_date_date_picker(window, cx));
        let address_city_input = cx.new(|cx| InputState::new(window, cx));
        let address_zip_input = cx.new(|cx| InputState::new(window, cx));
        let new_tag_input = cx.new(|cx| InputState::new(window, cx));
        let standalone_picker = cx.new(|cx| DatePickerState::new(window, cx));
        let file_picker = cx.new(|cx| FilePickerState::new(window, cx));

        let subscriptions = vec![
            cx.subscribe_in(&username_input, window, Self::on_username_change),
            cx.subscribe_in(&email_input, window, Self::on_email_change),
            cx.subscribe_in(&age_number_input, window, Self::on_age_change),
            cx.subscribe_in(&age_number_input, window, Self::on_age_step),
            cx.subscribe_in(&country_select, window, Self::on_country_confirm),
            cx.subscribe_in(&state_select, window, Self::on_state_confirm),
            cx.subscribe_in(&language_select, window, Self::on_language_confirm),
            cx.subscribe_in(&code_input, window, Self::on_code_change),
            cx.subscribe_in(&birth_date_date_picker, window, Self::on_birth_date_change),
            cx.subscribe_in(&address_city_input, window, Self::on_address_city_change),
            cx.subscribe_in(&address_zip_input, window, Self::on_address_zip_change),
            cx.subscribe_in(&standalone_picker, window, Self::on_standalone_pick),
            cx.subscribe_in(&file_picker, window, Self::on_file_pick),
        ];

        Self {
            form,
            errors_shown: false,
            submission: SubmitTracker::new(),
            email_check: EmailCheck::Idle,
            email_check_generation: 0,
            fields: SignupFormFields {
                username_input,
                email_input,
                age_number_input,
                country_select,
                state_select,
                language_select,
                code_input,
                birth_date_date_picker,
            },
            address_city_input,
            address_zip_input,
            new_tag_input,
            standalone_picker,
            file_picker,
            picked_date: None,
            status: None,
            show_schema: false,
            focus_handle: cx.focus_handle(),
            _subscriptions: subscriptions,
        }
    }

    fn revalidate(&mut self) {
        self.form.validate();
    }

    fn start_email_check(&mut self, cx: &mut Context<Self>) {
        self.email_check_generation += 1;
        let generation = self.email_check_generation;
        self.email_check = EmailCheck::Pending;
        let holder = self.form.current().clone();
        let field = FieldPath::new(&["email"]);
        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(400))
                .await;
            let outcome = EmailAvailability.validate_field(&holder, &field).await;
            _ = this.update(cx, |this, cx| {
                if this.email_check_generation != generation {
                    return;
                }
                this.email_check = match outcome {
                    Ok(()) => EmailCheck::Idle,
                    Err(key) => EmailCheck::Failed(key),
                };
                cx.notify();
            });
        })
        .detach();
    }

    fn on_username_change(
        &mut self,
        state: &Entity<InputState>,
        event: &InputEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            InputEvent::Blur => {
                self.form.touch(&SignupFormPath::username());
                cx.notify();
            },
            InputEvent::Change => {
                let text = state.read(cx).value();
                self.form.current_mut().username = if text.is_empty() {
                    None
                } else {
                    Some(text.to_string())
                };
                self.form.touch(&SignupFormPath::username());
                self.revalidate();
                cx.notify();
            },
            _ => {},
        }
    }

    fn on_email_change(
        &mut self,
        state: &Entity<InputState>,
        event: &InputEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            InputEvent::Blur => {
                self.form.touch(&SignupFormPath::email());
                cx.notify();
            },
            InputEvent::Change => {
                let text = state.read(cx).value();
                self.form.current_mut().email = if text.is_empty() {
                    None
                } else {
                    Some(text.to_string())
                };
                self.form.touch(&SignupFormPath::email());
                self.revalidate();
                self.start_email_check(cx);
                cx.notify();
            },
            _ => {},
        }
    }

    fn on_age_change(
        &mut self,
        state: &Entity<InputState>,
        event: &InputEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            InputEvent::Blur => {
                self.form.touch(&SignupFormPath::age());
                cx.notify();
            },
            InputEvent::Change => {
                let text = state.read(cx).value();
                self.form.current_mut().age = text.parse::<u32>().ok();
                self.form.touch(&SignupFormPath::age());
                self.revalidate();
                cx.notify();
            },
            _ => {},
        }
    }

    fn on_age_step(
        &mut self,
        state: &Entity<InputState>,
        event: &NumberInputEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let NumberInputEvent::Step(step_action) = event;
        let next = match step_action {
            StepAction::Increment => self
                .form
                .current()
                .age
                .unwrap_or_default()
                .saturating_add(1),
            StepAction::Decrement => self
                .form
                .current()
                .age
                .unwrap_or_default()
                .saturating_sub(1),
        };
        self.form.current_mut().age = Some(next);
        self.form.touch(&SignupFormPath::age());
        self.revalidate();
        state.update(cx, |input, cx| {
            input.set_value(next.to_string(), window, cx);
        });
        cx.notify();
    }

    fn on_country_confirm(
        &mut self,
        _: &Entity<SelectState<SearchableVec<Country>>>,
        event: &SelectEvent<SearchableVec<Country>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let SelectEvent::Confirm(value) = event;
        self.form.current_mut().country = value.clone();
        self.form.current_mut().state = None;
        self.fields.state_select.update(cx, |state, cx| {
            state.set_selected_index(None, window, cx);
        });
        self.form.touch(&SignupFormPath::country());
        self.revalidate();
        cx.notify();
    }

    fn on_state_confirm(
        &mut self,
        _: &Entity<SelectState<Vec<UsState>>>,
        event: &SelectEvent<Vec<UsState>>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let SelectEvent::Confirm(value) = event;
        if let Some(value) = value {
            self.form.current_mut().state = Some(value.clone());
            self.form.touch(&SignupFormPath::state());
            self.revalidate();
            cx.notify();
        }
    }

    fn on_language_confirm(
        &mut self,
        _: &Entity<SelectState<Vec<Language>>>,
        event: &SelectEvent<Vec<Language>>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let SelectEvent::Confirm(value) = event;
        if let Some(value) = value {
            self.form.current_mut().language = value.clone();
            self.form.touch(&SignupFormPath::language());
            self.revalidate();
            cx.notify();
        }
    }

    fn on_code_change(
        &mut self,
        state: &Entity<InputState>,
        event: &InputEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            InputEvent::Blur => {
                self.form.touch(&SignupFormPath::code());
                cx.notify();
            },
            InputEvent::Change => {
                let text = state.read(cx).value();
                self.form.current_mut().code = if text.is_empty() {
                    None
                } else {
                    text.parse::<RequestCode>().ok()
                };
                self.form.touch(&SignupFormPath::code());
                self.revalidate();
                cx.notify();
            },
            _ => {},
        }
    }

    fn on_birth_date_change(
        &mut self,
        _: &Entity<DatePickerState>,
        event: &DatePickerEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let DatePickerEvent::Change(date) = event;
        self.form.current_mut().birth_date =
            date.and_then(gpui_form::runtime::date_picker::parse_form_date);
        self.form.touch(&SignupFormPath::birth_date());
        self.revalidate();
        cx.notify();
    }

    fn on_address_city_change(
        &mut self,
        state: &Entity<InputState>,
        event: &InputEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            InputEvent::Blur => {
                self.form.touch(&SignupFormPath::address_root());
                cx.notify();
            },
            InputEvent::Change => {
                let text = state.read(cx).value();
                self.form.current_mut().address.city = if text.is_empty() {
                    None
                } else {
                    Some(text.to_string())
                };
                self.form.touch(&SignupFormPath::address_root());
                self.revalidate();
                cx.notify();
            },
            _ => {},
        }
    }

    fn on_address_zip_change(
        &mut self,
        state: &Entity<InputState>,
        event: &InputEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            InputEvent::Blur => {
                self.form.touch(&SignupFormPath::address_root());
                cx.notify();
            },
            InputEvent::Change => {
                let text = state.read(cx).value();
                self.form.current_mut().address.zip = if text.is_empty() {
                    None
                } else {
                    Some(text.to_string())
                };
                self.form.touch(&SignupFormPath::address_root());
                self.revalidate();
                cx.notify();
            },
            _ => {},
        }
    }

    fn on_standalone_pick(
        &mut self,
        _: &Entity<DatePickerState>,
        event: &DatePickerEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let DatePickerEvent::Change(date) = event;
        self.picked_date = date.map(|d| d.to_string().into());
        cx.notify();
    }

    fn on_file_pick(
        &mut self,
        _: &Entity<FilePickerState>,
        event: &FilePickerEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let FilePickerEvent::Change(_) = event {
            cx.notify();
        }
    }

    fn locale_button(
        &self,
        id: &'static str,
        label: &'static str,
        lang: &'static str,
        cx: &mut Context<Self>,
    ) -> Button {
        let active = rust_i18n::locale().to_string() == lang;
        let mut button = Button::new(id).label(label).small();
        if active {
            button = button.primary();
        }
        let root_focus = self.focus_handle.clone();
        button.on_click(cx.listener(move |_, _, window, cx| {
            // gpui-kit buttons take focus on mouse-down; refocus the root so
            // the input's blur/touched fires (github.com/longbridge/gpui-kit/issues/3183).
            root_focus.focus(window, cx);
            let lang_id: LanguageIdentifier = lang.parse().expect("valid language id");
            let _ = gpui_form::i18n::change_locale(cx, lang_id);
            cx.notify();
        }))
    }

    /// Input's padded wrapper tracks a separate frame-focus handle, so
    /// border clicks miss the editor (github.com/longbridge/gpui-kit/issues/3183).
    fn click_to_focus(&self, state: &Entity<InputState>) -> Div {
        let state = state.clone();
        div()
            .w_full()
            .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                state.update(cx, |state, cx| state.focus(window, cx));
            })
    }

    fn reset(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        *self = Self::new(window, cx);
        cx.notify();
    }

    fn submit(&mut self, cx: &mut Context<Self>) {
        self.revalidate();
        self.errors_shown = true;
        if !self.submission.begin() {
            cx.notify();
            return;
        }
        if !self.form.is_valid() {
            self.submission.finish(self.form.current().validate());
            self.status = Some(t!("app.invalid").to_string().into());
            cx.notify();
            return;
        }
        let holder = self.form.current().clone();
        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(700))
                .await;
            let result = SignupSubmit.submit(&holder).await;
            _ = this.update(cx, |this, cx| {
                this.submission.finish(result.map(|_| ()));
                if this.submission.is_submit_successful() {
                    this.form.sync_baseline();
                    this.status = Some(t!("app.saved").to_string().into());
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    fn touched_patch_summary(&self) -> String {
        let touched: std::collections::HashSet<String> = self
            .form
            .touched_fields()
            .iter()
            .map(|path| path.to_string())
            .collect();
        let patch = self.form.patch();
        let mut entries: Vec<String> = Vec::new();

        macro_rules! optional_slot {
            ($field:ident) => {
                if touched.contains(stringify!($field)) {
                    if let Some(value) = &patch.$field {
                        match value {
                            Some(value) => {
                                entries.push(format!("{} = {value:?}", stringify!($field)));
                            },
                            None => {
                                entries.push(format!("{} = null (cleared)", stringify!($field)))
                            },
                        }
                    }
                }
            };
        }
        macro_rules! plain_slot {
            ($field:ident) => {
                if touched.contains(stringify!($field)) {
                    if let Some(value) = &patch.$field {
                        entries.push(format!("{} = {value:?}", stringify!($field)));
                    }
                }
            };
        }

        optional_slot!(username);
        optional_slot!(email);
        optional_slot!(age);
        optional_slot!(country);
        optional_slot!(state);
        plain_slot!(language);
        plain_slot!(newsletter);
        plain_slot!(notifications);
        optional_slot!(code);
        optional_slot!(birth_date);
        plain_slot!(address);
        plain_slot!(tags);

        entries.join(", ")
    }

    fn schema_debug_lines() -> Vec<String> {
        use gpui_form::schema::registry::FormSchemaComponent;

        let schema = SignupFormSchema::schema();
        let mut lines = vec![format!(
            "{} · {} fields · koruma: {} · skipped fields: {}",
            schema.form,
            schema.fields.len(),
            if schema.koruma_enabled { "on" } else { "off" },
            if schema.has_skipped_fields {
                "yes"
            } else {
                "no"
            },
        )];

        for field in &schema.fields {
            let mut line = format!("{}: {}", field.name, field.kind().component_name());
            match &field.component {
                FormSchemaComponent::Nested { form_struct } => {
                    line.push_str(&format!(" ({form_struct})"));
                },
                FormSchemaComponent::List {
                    item, add_label, ..
                } => {
                    line.push_str(&format!(" (item: {item})"));
                    if let Some(add_label) = add_label {
                        line.push_str(&format!(" [add: {add_label}]"));
                    }
                },
                _ => {},
            }
            if field.optional || field.wraps_in_option {
                line.push_str(" (Option)");
            }
            if !field.validations.is_empty() {
                line.push_str(&format!(" [{}]", field.validations.join(", ")));
            }
            if let Some(depends_on) = field.depends_on.as_deref() {
                line.push_str(&format!(" (depends_on: {depends_on})"));
            }
            if let Some(required_if) = field.required_if.as_deref() {
                line.push_str(&format!(" (required_if: {required_if})"));
            }
            if let Some(section) = field.layout.section.as_deref() {
                line.push_str(&format!(" §{section}"));
            }
            lines.push(line);
        }

        lines
    }

    fn localized_path_error(&self, locale: &str, field: &SignupFormPath) -> Option<String> {
        if !self.errors_shown {
            return None;
        }
        let errors = self.form.validation()?;
        let field_error = errors.error_for(field)?;
        if let SignupFormFieldError::Code(wrapper) = &field_error {
            return localized_field_error(&wrapper.all(), locale, |v| match v {
                SignupFormValueHolderCodeKorumaValidator::RequiredValidation(_) => {
                    "validation.required"
                },
                SignupFormValueHolderCodeKorumaValidator::Inner(inner) => {
                    if inner.value().len_validation().is_some() {
                        "validation.len"
                    } else {
                        "validation.invalid"
                    }
                },
            });
        }
        Some(localized_issue_keys(&field_error.issue_keys(), locale))
    }

    fn localized_address_city_error(&self, locale: &str) -> Option<String> {
        if !self.errors_shown {
            return None;
        }
        let errors = self.form.validation()?;
        let nested_errors = errors.address_errors()?;
        let city_error = nested_errors.error_for(&AddressFormPath::city())?;
        Some(localized_issue_keys(&city_error.issue_keys(), locale))
    }

    fn localized_tags_error(&self, locale: &str) -> Option<String> {
        if !self.errors_shown {
            return None;
        }
        let errors = self.form.validation()?;
        let wrapper = errors.tags()?;
        let failing: Vec<usize> = wrapper.element_errors().iter().map(|(i, _)| *i).collect();
        (!failing.is_empty()).then(|| {
            format!(
                "{}: {}",
                t!("validation.len", locale = locale),
                failing
                    .iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
    }
}

fn localized_field_error<E>(
    errs: &[E],
    locale: &str,
    key_of: impl Fn(&E) -> &'static str,
) -> Option<String> {
    (!errs.is_empty()).then(|| {
        errs.iter()
            .map(|v| t!(key_of(v), locale = locale).to_string())
            .collect::<Vec<_>>()
            .join("\n")
    })
}

fn localized_issue_keys(keys: &[&'static str], locale: &str) -> String {
    keys.iter()
        .map(|key| t!(*key, locale = locale).to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

impl Focusable for FormsDemo {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for FormsDemo {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let active_locale = rust_i18n::locale().to_string();
        let username_error = self.localized_path_error(&active_locale, &SignupFormPath::username());
        let email_error = self.localized_path_error(&active_locale, &SignupFormPath::email());
        let email_async = match &self.email_check {
            EmailCheck::Pending => Some(t!("app.checking").to_string()),
            EmailCheck::Failed(key) => Some(t!(*key, locale = &active_locale).to_string()),
            EmailCheck::Idle => None,
        };
        let age_error = self.localized_path_error(&active_locale, &SignupFormPath::age());
        let code_error = self.localized_path_error(&active_locale, &SignupFormPath::code());
        let address_city_error = self.localized_address_city_error(&active_locale);
        let tags_error = self.localized_tags_error(&active_locale);
        let danger = cx.theme().danger;

        let is_valid = self.form.is_valid();
        let submitting = self.submission.is_submitting();
        let submit_attempts = self.submission.submit_count();
        let submit_error_banner = self
            .submission
            .last_error()
            .map(|error| {
                SignupFormFieldErrors::from_koruma_errors(error)
                    .failing_fields()
                    .flat_map(|(_, field_error)| field_error.issue_keys())
                    .map(|key| t!(key, locale = &active_locale).to_string())
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .filter(|text| !text.is_empty());

        let dirty = self.form.is_dirty();
        let changed = self
            .form
            .changed_fields()
            .iter()
            .map(|path| path.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let touched = self
            .form
            .touched_fields()
            .iter()
            .map(|path| path.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        v_flex()
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .p_4()
            .gap_3()
            .overflow_y_scrollbar()
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(
                        v_flex()
                            .gap_1()
                            .child(
                                div()
                                    .text_lg()
                                    .font_semibold()
                                    .child(t!("app.title").to_string()),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(t!("app.subtitle").to_string()),
                            ),
                    )
                    .child(
                        h_flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(t!("app.locale_hint").to_string()),
                            )
                            .child(self.locale_button("locale-en", "EN", "en", cx))
                            .child(self.locale_button("locale-fr", "FR", "fr-FR", cx))
                            .child(self.locale_button("locale-zh", "中文", "zh-CN", cx)),
                    ),
            )
            .child(
                v_form()
                    .child(
                        field()
                            .label(t!(SignupFormValueHolder::USERNAME_LABEL_KEY).to_string())
                            .description_fn(move |_, _| match &username_error {
                                Some(error) => div()
                                    .text_color(danger)
                                    .child(error.clone())
                                    .into_any_element(),
                                None => div().into_any_element(),
                            })
                            .child(
                                self.click_to_focus(&self.fields.username_input)
                                    .child(Input::new(&self.fields.username_input)),
                            ),
                    )
                    .child(
                        field()
                            .label(t!(SignupFormValueHolder::EMAIL_LABEL_KEY).to_string())
                            .description_fn(move |_, _| match (&email_error, &email_async) {
                                (Some(error), _) => div()
                                    .text_color(danger)
                                    .child(error.clone())
                                    .into_any_element(),
                                (None, Some(async_state)) => {
                                    div().child(async_state.clone()).into_any_element()
                                },
                                (None, None) => div().into_any_element(),
                            })
                            .child(
                                self.click_to_focus(&self.fields.email_input)
                                    .child(Input::new(&self.fields.email_input)),
                            ),
                    )
                    .child(
                        field()
                            .label(t!(SignupFormValueHolder::AGE_LABEL_KEY).to_string())
                            .description_fn(move |_, _| match &age_error {
                                Some(error) => div()
                                    .text_color(danger)
                                    .child(error.clone())
                                    .into_any_element(),
                                None => div().into_any_element(),
                            })
                            .child(
                                self.click_to_focus(&self.fields.age_number_input)
                                    .child(NumberInput::new(&self.fields.age_number_input)),
                            ),
                    )
                    .child(
                        field()
                            .label(t!(SignupFormValueHolder::COUNTRY_LABEL_KEY).to_string())
                            .child(Select::new(&self.fields.country_select)),
                    )
                    .child(
                        field()
                            .label(t!(SignupFormValueHolder::STATE_LABEL_KEY).to_string())
                            .child(Select::new(&self.fields.state_select)),
                    )
                    .child(
                        field()
                            .label(t!(SignupFormValueHolder::LANGUAGE_LABEL_KEY).to_string())
                            .child(Select::new(&self.fields.language_select)),
                    )
                    .child(
                        field()
                            .label(t!(SignupFormValueHolder::NEWSLETTER_LABEL_KEY).to_string())
                            .child(
                                Checkbox::new("newsletter-checkbox")
                                    .checked(self.form.current().newsletter)
                                    .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                        this.form.current_mut().newsletter = *checked;
                                        this.form.touch(&SignupFormPath::newsletter());
                                        this.revalidate();
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(
                        field()
                            .label(t!(SignupFormValueHolder::NOTIFICATIONS_LABEL_KEY).to_string())
                            .child(
                                Switch::new("notifications-switch")
                                    .checked(self.form.current().notifications)
                                    .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                        this.form.current_mut().notifications = *checked;
                                        this.form.touch(&SignupFormPath::notifications());
                                        this.revalidate();
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(
                        field()
                            .label(t!(SignupFormValueHolder::CODE_LABEL_KEY).to_string())
                            .description_fn(move |_, _| match &code_error {
                                Some(error) => div()
                                    .text_color(danger)
                                    .child(error.clone())
                                    .into_any_element(),
                                None => div().into_any_element(),
                            })
                            .child(
                                self.click_to_focus(&self.fields.code_input)
                                    .child(Input::new(&self.fields.code_input)),
                            ),
                    )
                    .child(
                        field()
                            .label(t!(SignupFormValueHolder::BIRTH_DATE_LABEL_KEY).to_string())
                            .child(DatePicker::new(&self.fields.birth_date_date_picker)),
                    ),
            )
            .child(
                v_flex()
                    .gap_2()
                    .border_1()
                    .rounded_md()
                    .p_2()
                    .child(
                        div()
                            .font_semibold()
                            .child(t!("signup.address_label").to_string()),
                    )
                    .child(
                        field()
                            .label(t!("signup.address_city_label").to_string())
                            .description_fn({
                                let error = address_city_error.clone();
                                move |_, _| match &error {
                                    Some(text) => div()
                                        .text_color(danger)
                                        .child(text.clone())
                                        .into_any_element(),
                                    None => div().into_any_element(),
                                }
                            })
                            .child(
                                self.click_to_focus(&self.address_city_input)
                                    .child(Input::new(&self.address_city_input)),
                            ),
                    )
                    .child(
                        field()
                            .label(t!("signup.address_zip_label").to_string())
                            .child(
                                self.click_to_focus(&self.address_zip_input)
                                    .child(Input::new(&self.address_zip_input)),
                            ),
                    ),
            )
            .child(
                v_flex()
                    .gap_2()
                    .child(
                        h_flex()
                            .justify_between()
                            .child(
                                div()
                                    .font_semibold()
                                    .child(t!("app.tags_section").to_string()),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!(
                                        "{}: {}",
                                        t!("app.tags_count"),
                                        self.form.current().tags_items().len()
                                    )),
                            ),
                    )
                    .children(
                        tags_error
                            .clone()
                            .map(|text| div().text_color(danger).child(text)),
                    )
                    .children(self.form.current().tags_items().iter().enumerate().map(
                        |(index, tag)| {
                            h_flex()
                                .justify_between()
                                .child(div().child(tag.to_string()))
                                .child(
                                    Button::new(format!("tags-remove-{index}"))
                                        .label(t!("app.remove_tag").to_string())
                                        .on_click(cx.listener(move |this, _, window, cx| {
                                            this.focus_handle.focus(window, cx);
                                            this.form.current_mut().remove_tags(index);
                                            this.form.touch(&SignupFormPath::tags_root());
                                            this.revalidate();
                                            cx.notify();
                                        })),
                                )
                        },
                    ))
                    .child(
                        // The new-tag editor must render: an unrendered input
                        // can never gain focus or text, so Add would read "".
                        h_flex()
                            .gap_2()
                            .child(
                                self.click_to_focus(&self.new_tag_input)
                                    .child(Input::new(&self.new_tag_input)),
                            )
                            .child(
                                Button::new("tags-add")
                                    .label(t!("app.add_tag").to_string())
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.focus_handle.focus(window, cx);
                                        let text = this.new_tag_input.read(cx).value();
                                        if !text.is_empty() {
                                            this.form.current_mut().add_tags(text.to_string());
                                            this.form.touch(&SignupFormPath::tags_root());
                                            this.revalidate();
                                            this.new_tag_input.update(cx, |state, cx| {
                                                state.set_value("", window, cx);
                                            });
                                            cx.notify();
                                        }
                                    })),
                            ),
                    ),
            )
            .child(
                field()
                    .label(t!("app.date_section").to_string())
                    .description_fn({
                        let picked = self.picked_date.clone();
                        move |_, _| match &picked {
                            Some(date) => div()
                                .child(format!("{}: {date}", t!("app.picked")))
                                .into_any_element(),
                            None => div().into_any_element(),
                        }
                    })
                    .child(
                        DatePicker::new(&self.standalone_picker)
                            .cleanable(true)
                            .number_of_months(1),
                    ),
            )
            .child(
                field().label(t!("app.file_section").to_string()).child(
                    FilePicker::new(&self.file_picker)
                        .multiple(true)
                        .cleanable(true),
                ),
            )
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("validate")
                            .primary()
                            .label(t!("app.validate").to_string())
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.focus_handle.focus(window, cx);
                                this.revalidate();
                                this.errors_shown = true;
                                this.status = Some(
                                    if this.form.is_valid() {
                                        t!("app.all_valid").to_string()
                                    } else {
                                        t!("app.invalid").to_string()
                                    }
                                    .into(),
                                );
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("submit")
                            .primary()
                            .disabled(!self.submission.can_submit())
                            .label(if submitting {
                                t!("app.submitting").to_string()
                            } else {
                                t!("app.submit").to_string()
                            })
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.focus_handle.focus(window, cx);
                                this.submit(cx);
                            })),
                    )
                    .child(
                        Button::new("reset")
                            .label(t!("app.reset").to_string())
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.reset(window, cx);
                                this.focus_handle.focus(window, cx);
                            })),
                    )
                    .child(
                        Button::new("save-changes")
                            .label(t!("app.save_changes").to_string())
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.focus_handle.focus(window, cx);
                                let payload = this.touched_patch_summary();
                                this.status = Some(
                                    if payload.is_empty() {
                                        t!("app.nothing_touched").to_string()
                                    } else {
                                        format!("{}: {payload}", t!("app.touched_payload"))
                                    }
                                    .into(),
                                );
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("schema-toggle")
                            .label(t!("app.schema").to_string())
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.focus_handle.focus(window, cx);
                                this.show_schema = !this.show_schema;
                                cx.notify();
                            })),
                    ),
            )
            .children(submit_error_banner.map(|text| {
                div()
                    .text_color(danger)
                    .child(format!("{}: {text}", t!("app.submit_failed")))
            }))
            .child(
                v_flex()
                    .gap_1()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("locale: {}", &*rust_i18n::locale()))
                    .child(format!("{}: {dirty}", t!("app.dirty")))
                    .child(format!("{}: {changed}", t!("app.changed")))
                    .child(format!("{}: {touched}", t!("app.touched")))
                    .child(format!("{}: {is_valid}", t!("app.valid")))
                    .child(format!("{}: {submitting}", t!("app.submitting_state")))
                    .child(format!("{}: {submit_attempts}", t!("app.submit_count")))
                    .child(format!(
                        "{}: {}",
                        t!("app.paths"),
                        self.file_picker.read(cx).paths().len()
                    ))
                    .child(t!("app.paths_note").to_string())
                    .child(
                        [
                            SignupFormPath::username().to_string(),
                            SignupFormPath::email().to_string(),
                            SignupFormPath::age().to_string(),
                            SignupFormPath::country().to_string(),
                            SignupFormPath::state().to_string(),
                            SignupFormPath::language().to_string(),
                            SignupFormPath::newsletter().to_string(),
                            SignupFormPath::notifications().to_string(),
                            SignupFormPath::code().to_string(),
                            SignupFormPath::birth_date().to_string(),
                            SignupFormPath::address_root().to_string(),
                            SignupFormPath::tags_root().to_string(),
                        ]
                        .join(", "),
                    )
                    .children(self.status.clone().map(|s| div().child(s))),
            )
            .children(self.show_schema.then(|| {
                v_flex()
                    .gap_1()
                    .text_sm()
                    .child(format!("{}:", t!("app.schema")))
                    .children(
                        Self::schema_debug_lines()
                            .into_iter()
                            .map(|line| div().child(line)),
                    )
            }))
    }
}

fn main() {
    gpui_kit::application().with_assets(Assets).run(|cx| {
        gpui_kit::init(cx);
        gpui_form::i18n::init(cx);

        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::centered(size(px(760.), px(860.)), cx)),
            ..Default::default()
        };
        gpui_kit::open_window(window_options, cx, |window, cx| {
            cx.new(|cx| FormsDemo::new(window, cx))
        })
        .expect("failed to open the forms-demo window");
    });
}

#[cfg(test)]
mod test_util {
    use std::sync::Mutex;

    pub(crate) static LOCALE_LOCK: Mutex<()> = Mutex::new(());
}

#[cfg(test)]
mod tests {
    use crate::SignupFormValueHolder;
    use crate::test_util::LOCALE_LOCK;
    use rust_i18n::t;

    const KEYS: &[&str] = &[
        "app.title",
        "app.subtitle",
        "app.locale_hint",
        "app.validate",
        "app.reset",
        "app.all_valid",
        "app.invalid",
        "app.date_section",
        "app.file_section",
        "app.picked",
        "app.paths",
        "app.dirty",
        "app.changed",
        "app.touched",
        "app.paths_note",
        "app.submit",
        "app.submitting",
        "app.submitting_state",
        "app.submit_count",
        "app.saved",
        "app.submit_failed",
        "app.valid",
        "app.email_taken",
        "app.checking",
        "app.tags_section",
        "app.tags_count",
        "app.add_tag",
        "app.remove_tag",
        "app.save_changes",
        "app.nothing_touched",
        "app.touched_payload",
        "app.schema",
        "signup.username_label",
        "signup.email_label",
        "signup.age_label",
        "signup.country_label",
        "signup.state_label",
        "signup.language_label",
        "signup.newsletter_label",
        "signup.notifications_label",
        "signup.code_label",
        "signup.birth_date_label",
        "signup.address_label",
        "signup.address_city_label",
        "signup.address_zip_label",
        "country.united_states",
        "country.france",
        "country.china",
        "us_state.california",
        "us_state.new_york",
        "us_state.texas",
        "language.english",
        "language.french",
        "language.chinese",
        "validation.required",
        "validation.non_empty",
        "validation.range",
        "validation.len",
        "validation.email",
        "validation.invalid",
    ];

    #[test]
    fn every_key_resolves_in_every_locale() {
        let _locale_guard = LOCALE_LOCK.lock().unwrap();
        for locale in ["en", "fr-FR", "zh-CN"] {
            rust_i18n::set_locale(locale);
            for &key in KEYS {
                assert_ne!(&*t!(key), key, "key {key} missing in locale {locale}");
            }
        }
        rust_i18n::set_locale("en");
    }

    #[test]
    fn koruma_issue_kinds_map_to_locale_keys() {
        assert_eq!(
            SignupFormValueHolder::validation_issue_key("NonEmptyValidation"),
            "validation.non_empty"
        );
        assert_eq!(
            SignupFormValueHolder::validation_issue_key("RangeValidation"),
            "validation.range"
        );
        assert_eq!(
            SignupFormValueHolder::validation_issue_key("RequiredValidation"),
            "validation.required"
        );
        assert_eq!(
            SignupFormValueHolder::validation_issue_key("EmailValidation"),
            "validation.email"
        );
        assert_eq!(
            SignupFormValueHolder::validation_issue_key("LenValidation"),
            "validation.len",
            "M3: the tags each(LenValidation) element validator promotes LenValidation into the emitted kind match; kinds that live only inside newtype validators still fall back to validation.invalid"
        );
        assert_eq!(
            SignupFormValueHolder::validation_issue_key("OnlyInsideNewtypeValidation"),
            "validation.invalid",
            "kinds declared inside newtype validators are not in the emitted match — the app maps Inner variants itself"
        );
    }
}

#[cfg(test)]
mod validation_locale_tests {
    use crate::SignupFormValueHolder;
    use crate::SignupFormValueHolderUsernameKorumaValidator;
    use crate::localized_field_error;

    #[test]
    fn field_errors_render_in_french() {
        let err = SignupFormValueHolder::default().validate().err().unwrap();
        let errs = err.username().all();
        let rendered = localized_field_error(&errs, "fr-FR", |v| match v {
            SignupFormValueHolderUsernameKorumaValidator::RequiredValidation(_) => {
                "validation.required"
            },
            SignupFormValueHolderUsernameKorumaValidator::NonEmptyValidation(_) => {
                "validation.non_empty"
            },
        })
        .unwrap();
        assert_eq!(rendered, "Ce champ est obligatoire.");

        let rendered = localized_field_error(&errs, "zh-CN", |v| match v {
            SignupFormValueHolderUsernameKorumaValidator::RequiredValidation(_) => {
                "validation.required"
            },
            SignupFormValueHolderUsernameKorumaValidator::NonEmptyValidation(_) => {
                "validation.non_empty"
            },
        })
        .unwrap();
        assert_eq!(rendered, "此字段为必填项。");
    }
}

#[cfg(test)]
mod newtype_locale_tests {
    use crate::RequestCode;
    use crate::SignupFormValueHolder;
    use crate::SignupFormValueHolderCodeKorumaValidator;
    use crate::localized_field_error;

    fn rendered_code_error(locale: &str) -> String {
        let holder = SignupFormValueHolder {
            username: Some("ada".to_string()),
            code: Some(RequestCode::from("A".to_string())),
            ..Default::default()
        };
        let err = holder.validate().expect_err("short invite code must fail");
        localized_field_error(&err.code().all(), locale, |v| match v {
            SignupFormValueHolderCodeKorumaValidator::RequiredValidation(_) => {
                "validation.required"
            },
            SignupFormValueHolderCodeKorumaValidator::Inner(inner) => {
                if inner.value().len_validation().is_some() {
                    "validation.len"
                } else {
                    "validation.invalid"
                }
            },
        })
        .expect("short invite code must render an error")
    }

    #[test]
    fn newtype_inner_error_localizes() {
        assert_eq!(
            rendered_code_error("fr-FR"),
            "La longueur doit rester dans les limites autorisées."
        );
        assert_eq!(rendered_code_error("zh-CN"), "长度超出允许范围。");
        assert_eq!(
            rendered_code_error("en"),
            "Length must be between the allowed bounds."
        );
    }
}

#[cfg(test)]
mod label_keys {
    use crate::SignupFormValueHolder;
    use rust_i18n::t;

    #[test]
    fn generated_label_keys_resolve_in_every_locale() {
        for (locale, expected) in [
            ("en", "Username"),
            ("fr-FR", "Nom d'utilisateur"),
            ("zh-CN", "用户名"),
        ] {
            rust_i18n::set_locale(locale);
            assert_eq!(
                t!(SignupFormValueHolder::USERNAME_LABEL_KEY).to_string(),
                expected,
                "username label wrong in locale {locale}"
            );
        }
        rust_i18n::set_locale("en");
    }
}

#[cfg(test)]
mod field_level_state_tests {
    use crate::{RequestCode, SignupFormPath, SignupFormState, SignupFormValueHolder};
    use gpui_form::core::FormFieldErrors;

    #[test]
    fn facade_tracks_dirty_changed_and_touched_together() {
        let mut state = SignupFormState::new(SignupFormValueHolder::default());
        assert!(!state.is_dirty());
        assert!(!state.is_any_touched());

        state.current_mut().username = Some("ada".to_string());
        state.touch(&SignupFormPath::username());

        assert!(state.is_dirty());
        assert!(state.is_field_dirty(&SignupFormPath::username()));
        assert!(!state.is_field_dirty(&SignupFormPath::email()));
        assert_eq!(
            state
                .changed_fields()
                .iter()
                .map(|path| path.to_string())
                .collect::<Vec<_>>(),
            vec!["username"]
        );
        assert_eq!(
            state
                .touched_fields()
                .iter()
                .map(|path| path.to_string())
                .collect::<Vec<_>>(),
            vec!["username"]
        );

        state.reset_to_baseline();
        assert!(!state.is_dirty());
        assert!(!state.is_any_touched());
        assert!(state.changed_fields().is_empty());
    }

    #[test]
    fn patch_reports_only_the_edited_slot() {
        let mut state = SignupFormState::new(SignupFormValueHolder::default());
        state.current_mut().age = Some(42);
        let patch = state.patch();
        assert!(patch.age.is_some());
        assert!(patch.username.is_none());
        let age_slot: Option<Option<u32>> = patch.age;
        assert_eq!(age_slot, Some(Some(42)));
    }

    #[test]
    fn error_for_paths_map_to_localizable_keys() {
        let errors = SignupFormValueHolder::default().validate_fields();
        assert!(errors.has_errors());
        let username = errors
            .error_for(&SignupFormPath::username())
            .expect("username must fail");
        assert_eq!(username.issue_keys(), vec!["validation.required"]);
        assert!(errors.error_for(&SignupFormPath::country()).is_none());

        let holder = SignupFormValueHolder {
            username: Some("ada".to_string()),
            code: Some(RequestCode::from("A".to_string())),
            ..Default::default()
        };
        let errors = holder.validate_fields();
        let code = errors
            .error_for(&SignupFormPath::code())
            .expect("short invite code must fail");
        assert_eq!(code.issue_keys(), vec!["validation.invalid"]);

        let paths = errors.failing_paths();
        let names: Vec<String> = paths.iter().map(|path| path.to_string()).collect();
        assert_eq!(names, vec!["code", "address"]);
    }

    #[test]
    fn path_tables_cover_every_non_skipped_field() {
        assert_eq!(SignupFormPath::FIELD_COUNT, 12);
        assert_eq!(SignupFormPath::all_paths().count(), 12);
        let names: Vec<&str> = SignupFormPath::FIELD_NAMES.to_vec();
        assert!(names.contains(&"address"));
        assert!(names.contains(&"tags"));
        assert!(names.contains(&"state"));
        assert!(!SignupFormPath::FIELD_NAMES.contains(&"session_id"));
    }
}

#[cfg(test)]
mod dependency_tests {
    use crate::{SignupFormPath, SignupFormState, SignupFormValueHolder, UsState};
    use gpui_form::core::FormFieldErrors as _;

    #[test]
    fn email_is_required_only_while_subscribing() {
        let mut holder = SignupFormValueHolder {
            username: Some("ada".to_string()),
            code: Some("AB12".parse().unwrap()),
            age: Some(30),
            country: Some(crate::Country::France),
            ..Default::default()
        };
        holder.address.city = Some("Paris".to_string());

        assert!(
            holder.clone().validate().is_ok(),
            "newsletter off: email stays optional"
        );

        holder.newsletter = true;
        let errors = holder.validate_fields();
        let error = errors
            .error_for(&SignupFormPath::email())
            .expect("newsletter on + absent email must fail");
        assert_eq!(error.issue_keys(), vec!["validation.required"]);
        assert_eq!(errors.failing_paths().len(), 1, "only email fails");

        holder.email = Some("ada@example.com".to_string());
        assert!(
            holder.validate().is_ok(),
            "newsletter on + present email passes"
        );
    }

    #[test]
    fn conditional_required_localizes_through_the_shared_key() {
        let holder = SignupFormValueHolder {
            newsletter: true,
            ..Default::default()
        };
        let errors = holder.validate_fields();
        let error = errors
            .error_for(&SignupFormPath::email())
            .expect("conditional violation");
        let text = crate::localized_issue_keys(&error.issue_keys(), "fr-FR");
        assert_eq!(text, "Ce champ est obligatoire.");
    }

    #[test]
    fn touched_patch_summary_reports_only_touched_changes() {
        let mut state = SignupFormState::new(SignupFormValueHolder::default());
        state.current_mut().email = Some("ada@example.com".to_string());
        state.touch(&SignupFormPath::email());
        state.current_mut().age = Some(42);

        let summary = demo_touched_patch_summary(&mut state);
        assert_eq!(
            summary, "email = \"ada@example.com\"",
            "untouched edits (age) are excluded; touched+changed fields ride along"
        );

        state.current_mut().email = None;
        let summary = demo_touched_patch_summary(&mut state);
        assert_eq!(summary, "", "back to baseline: nothing to send");

        state.sync_baseline();
        state.current_mut().email = Some("ada@example.com".to_string());
        state.sync_baseline();
        state.current_mut().email = None;
        let summary = demo_touched_patch_summary(&mut state);
        assert_eq!(summary, "email = null (cleared)");
    }

    fn demo_touched_patch_summary(state: &mut SignupFormState) -> String {
        let touched: std::collections::HashSet<String> = state
            .touched_fields()
            .iter()
            .map(|path| path.to_string())
            .collect();
        let patch = state.patch();
        let mut entries: Vec<String> = Vec::new();
        if touched.contains("email")
            && let Some(value) = &patch.email
        {
            match value {
                Some(value) => entries.push(format!("email = {value:?}")),
                None => entries.push("email = null (cleared)".to_string()),
            }
        }
        if touched.contains("age")
            && let Some(value) = &patch.age
        {
            match value {
                Some(value) => entries.push(format!("age = {value:?}")),
                None => entries.push("age = null (cleared)".to_string()),
            }
        }
        entries.join(", ")
    }

    #[test]
    fn state_field_tracks_country_dependency_metadata() {
        let mut state = SignupFormState::new(SignupFormValueHolder::default());
        state.current_mut().state = Some(UsState::Texas);
        state.touch(&SignupFormPath::state());
        assert!(state.is_field_dirty(&SignupFormPath::state()));
        assert_eq!(
            state
                .touched_fields()
                .iter()
                .map(|path| path.to_string())
                .collect::<Vec<_>>(),
            vec!["state"]
        );
        state.sync_baseline();
        assert!(!state.is_field_dirty(&SignupFormPath::state()));

        state.current_mut().state = None;
        let patch = state.patch();
        let slot: Option<Option<UsState>> = patch.state;
        assert_eq!(slot, Some(None), "changed-to-cleared tri-state");
    }
}

#[cfg(test)]
mod submit_lifecycle_tests {
    use crate::{
        EmailAvailability, EmailCheck, SignupFormFieldErrors, SignupFormPath, SignupFormState,
        SignupFormSubmission, SignupFormValueHolder,
    };
    use gpui_form::AsyncFieldValidation;
    use gpui_form::core::FormFieldErrors;
    use gpui_form::{FieldPath, SubmitTracker};
    use std::future::Future;
    use std::task::Poll;

    fn block_on<F: Future>(future: F) -> F::Output {
        let mut future = std::pin::pin!(future);
        let waker = std::task::Waker::noop();
        let mut cx = std::task::Context::from_waker(waker);
        loop {
            match future.as_mut().poll(&mut cx) {
                Poll::Ready(output) => return output,
                Poll::Pending => continue,
            }
        }
    }

    #[test]
    fn facade_cache_serves_live_validity() {
        let mut state = SignupFormState::new(SignupFormValueHolder::default());
        assert!(state.validation().is_none());
        assert!(!state.is_valid());

        state.validate();
        assert!(state.validation().is_some());
        assert!(!state.is_valid(), "default holder fails required fields");

        state.current_mut().username = Some("ada".to_string());
        state.current_mut().code = Some("AB12".parse().unwrap());
        state.current_mut().age = Some(30);
        state.current_mut().address.city = Some("Paris".to_string());
        assert!(state.validation().is_none(), "edits invalidate the cache");
        assert!(state.is_valid(), "complete valid holder passes");
        state.validate();
        assert!(state.validation().is_some());

        state.current_mut().age = Some(200);
        state.validate();
        let age = state
            .error_for(&SignupFormPath::age())
            .expect("out-of-range age must fail");
        assert_eq!(age.issue_keys(), vec!["validation.range"]);
    }

    #[test]
    fn every_submit_attempt_counts_even_when_the_form_is_invalid() {
        let mut tracker: SignupFormSubmission = SubmitTracker::new();
        let mut state = SignupFormState::new(SignupFormValueHolder::default());
        state.validate();

        assert!(tracker.begin());
        assert!(!state.is_valid());
        tracker.finish(state.current().validate());
        assert_eq!(tracker.submit_count(), 1);
        assert!(!tracker.is_submit_successful());
        assert!(
            tracker.last_error().is_some(),
            "the invalid attempt's koruma error feeds the failure banner"
        );

        state.current_mut().username = Some("ada".to_string());
        state.current_mut().code = Some("AB12".parse().unwrap());
        state.current_mut().age = Some(30);
        state.current_mut().address.city = Some("Paris".to_string());
        state.validate();
        assert!(state.is_valid());
        assert!(tracker.begin());
        tracker.finish(Ok(()));
        assert_eq!(tracker.submit_count(), 2);
        assert!(tracker.is_submit_successful());
    }

    #[test]
    fn submit_alias_is_a_submit_tracker_over_the_koruma_error() {
        let mut tracker: SignupFormSubmission = SubmitTracker::new();
        assert!(tracker.can_submit());

        let failure = SignupFormValueHolder::default()
            .validate()
            .expect_err("default holder must fail");
        tracker.begin();
        tracker.finish(Err(failure));
        assert!(!tracker.is_submit_successful());

        let last = tracker.last_error().expect("recorded failure");
        let errors = SignupFormFieldErrors::from_koruma_errors(last);
        let paths = errors
            .failing_paths()
            .iter()
            .map(|path| path.to_string())
            .collect::<Vec<_>>();
        assert!(paths.contains(&"username".to_string()));

        tracker.begin();
        tracker.finish(Ok(()));
        assert!(tracker.is_submit_successful());

        tracker.begin();
        tracker.reset();
        tracker.finish(Ok(()));
        assert!(!tracker.is_submitted());
    }

    #[test]
    fn email_availability_check_rejects_taken_addresses_only() {
        let taken = SignupFormValueHolder {
            email: Some("TAKEN@example.com".to_string()),
            ..Default::default()
        };
        let free = SignupFormValueHolder {
            email: Some("ada@example.com".to_string()),
            ..Default::default()
        };
        let email = FieldPath::new(&["email"]);

        assert_eq!(
            block_on(EmailAvailability.validate_field(&taken, &email)),
            Err("app.email_taken")
        );
        assert_eq!(
            block_on(EmailAvailability.validate_field(&free, &email)),
            Ok(())
        );
        assert_eq!(
            block_on(EmailAvailability.validate_field(&free, &FieldPath::new(&["username"]))),
            Ok(())
        );

        assert_eq!(EmailCheck::default(), EmailCheck::Idle);
    }
}

#[cfg(test)]
mod nested_and_list_tests {
    use crate::{AddressFormPath, SignupFormPath, SignupFormState, SignupFormValueHolder};
    use gpui_form::core::FormFieldErrors as _;

    #[test]
    fn nested_error_access_recurses_through_the_nested_form() {
        let errors = SignupFormValueHolder::default().validate_fields();

        let root = errors
            .error_for(&SignupFormPath::address_root())
            .expect("the nested root routes to the nested slot");
        assert_eq!(root.issue_keys(), vec!["validation.invalid"]);

        let composed = SignupFormPath::address_root().join(&AddressFormPath::city());
        assert_eq!(composed.to_string(), "address.city");
        let deep = errors
            .error_for(&composed)
            .expect("composed paths route through the nested root");
        assert_eq!(deep.issue_keys(), vec!["validation.invalid"]);

        let nested_errors = errors
            .address_errors()
            .expect("the nested map accessor rebuilds the nested form's map");
        let city_error = nested_errors
            .error_for(&AddressFormPath::city())
            .expect("empty city fails the nested rules");
        assert!(city_error.issue_keys().contains(&"validation.required"));

        let mut holder = SignupFormValueHolder::default();
        holder.address.city = Some("Paris".to_string());
        let errors = holder.validate_fields();
        assert!(errors.error_for(&SignupFormPath::address_root()).is_none());
        assert!(errors.error_for(&composed).is_none());
    }

    #[test]
    fn list_element_errors_carry_indices() {
        let mut holder = SignupFormValueHolder {
            username: Some("ada".to_string()),
            code: Some("AB12".parse().unwrap()),
            age: Some(30),
            tags: vec!["ok".to_string(), "x".to_string(), "gpui".to_string()],
            ..Default::default()
        };
        holder.address.city = Some("Paris".to_string());

        let errors = holder.validate_fields();
        let failing: Vec<usize> = errors
            .tags()
            .expect("the tags wrapper slot is populated")
            .element_errors()
            .iter()
            .map(|(index, _)| *index)
            .collect();
        assert_eq!(failing, vec![1], "only the too-short tag fails");

        assert!(errors.error_for(&SignupFormPath::tags(0)).is_none());
        let item = errors
            .error_for(&SignupFormPath::tags(1))
            .expect("item 1 failed its each(..) validators");
        assert_eq!(item.issue_keys(), vec!["validation.len"]);
        assert!(errors.error_for(&SignupFormPath::tags(2)).is_none());

        assert!(errors.tags_element_error(0).is_none());
        assert!(errors.tags_element_error(1).is_some());

        let names: Vec<String> = errors
            .failing_paths()
            .iter()
            .map(|path| path.to_string())
            .collect();
        assert_eq!(names, vec!["tags.1", "tags"]);
    }

    #[test]
    fn nested_inputs_mark_touched_through_the_root_path() {
        let mut state = SignupFormState::new(SignupFormValueHolder::default());
        state.current_mut().address.city = Some("Paris".to_string());
        state.touch(&SignupFormPath::address_root());
        assert!(state.is_touched(&SignupFormPath::address_root()));
        assert_eq!(
            state
                .touched_fields()
                .iter()
                .map(|path| path.to_string())
                .collect::<Vec<_>>(),
            vec!["address"]
        );
        state.touch(&SignupFormPath::address_root());
        assert_eq!(state.touched_fields().len(), 1);

        let composed = SignupFormPath::address_root().join(&AddressFormPath::city());
        assert_eq!(composed.to_string(), "address.city");
        assert_eq!(composed.field_index(), None);
        let mut untouched = SignupFormState::new(SignupFormValueHolder::default());
        untouched.touch(&composed);
        assert!(
            untouched.touched_fields().is_empty(),
            "composed multi-segment paths have no dense index in v1"
        );
    }

    #[test]
    fn list_mutation_api_drives_the_demo_editor() {
        let mut state = SignupFormState::new(SignupFormValueHolder::default());

        state.current_mut().add_tags("gpui".to_string());
        state.current_mut().add_tags("forms".to_string());
        assert_eq!(state.current().tags_items(), &["gpui", "forms"]);

        state.current_mut().remove_tags(0);
        assert_eq!(state.current().tags_items(), &["forms"]);

        assert!(state.is_field_dirty(&SignupFormPath::tags_root()));
        state.touch(&SignupFormPath::tags_root());
        assert!(state.is_touched(&SignupFormPath::tags_root()));

        assert_eq!(SignupFormPath::tags(2).to_string(), "tags.2");
        assert!(SignupFormPath::tags(2).field_index().is_none());
        assert_eq!(SignupFormPath::tags_root().field_index(), Some(11));
    }
}

#[cfg(test)]
mod schema_export_tests {
    use crate::{FormsDemo, SignupFormSchema};
    use gpui_form::FormSchema;
    use gpui_form::schema::registry::FormSchemaComponent;

    #[test]
    fn exported_schema_reads_without_the_gpui_runtime() {
        let schema: FormSchema = SignupFormSchema::schema();

        assert_eq!(schema.form, "Signup");
        assert!(schema.koruma_enabled);
        assert!(schema.has_validations());
        assert_eq!(
            schema.field_names().collect::<Vec<_>>(),
            vec![
                "username",
                "email",
                "age",
                "country",
                "state",
                "language",
                "newsletter",
                "notifications",
                "code",
                "birth_date",
                "address",
                "tags",
            ]
        );

        let state = schema.field("state").expect("state entry");
        assert_eq!(state.depends_on.as_deref(), Some("country"));

        assert_eq!(
            schema.field("address").expect("address entry").component,
            FormSchemaComponent::Nested {
                form_struct: "Address".to_string(),
            }
        );
        assert_eq!(
            schema.field("tags").expect("tags entry").component,
            FormSchemaComponent::List {
                item: gpui_form::schema::components::ComponentKind::Input,
                item_struct: None,
                add_label: None,
            }
        );
    }

    #[test]
    fn footer_debug_lines_render_the_exported_schema() {
        let lines = FormsDemo::schema_debug_lines();

        assert_eq!(lines.len(), 1 + 12);
        assert!(
            lines[0].starts_with("Signup · 12 fields · koruma: on · skipped fields: yes"),
            "{lines:?}"
        );
        assert!(
            lines[1].contains("username: input (Option) [RequiredValidation"),
            "{lines:?}"
        );
        assert!(
            lines[5].contains("state: select (Option) (depends_on: country)"),
            "{lines:?}"
        );
        assert!(
            lines[2].contains("email: input (Option)")
                && lines[2].contains("(required_if: newsletter)"),
            "{lines:?}"
        );
        assert!(
            lines[11].contains("address: nested (Address) [NestedValidation]"),
            "{lines:?}"
        );
        assert!(lines[12].contains("tags: list (item: input)"), "{lines:?}");
    }
}
