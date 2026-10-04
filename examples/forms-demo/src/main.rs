#![recursion_limit = "1024"]

use gpui::{
    App, AppContext as _, Context, Entity, FocusHandle, Focusable, IntoElement, ParentElement as _,
    Render, SharedString, Styled as _, Subscription, Window, div,
};
use gpui_form::runtime::date_picker::{DatePicker, DatePickerEvent, DatePickerState};
use gpui_form::runtime::file_picker::{FilePicker, FilePickerEvent, FilePickerState};
use gpui_form::{GpuiForm, SelectItem};
use gpui_kit::assets::Assets;
use gpui_kit::component::{
    ActiveTheme as _, Sizable as _,
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
use koruma::{Koruma, KorumaAllDisplay};
use koruma_collection::{
    collection::{LenValidation, NonEmptyValidation},
    format::EmailValidation,
    numeric::RangeValidation,
};
use rust_i18n::t;
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

#[derive(Clone, Debug, Default, GpuiForm, Koruma, KorumaAllDisplay)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[gpui_form(koruma(fluent))]
pub struct Signup {
    #[gpui_form(label = "Username", placeholder = "ada", component(input))]
    #[koruma(NonEmptyValidation::<_>::builder())]
    pub username: String,

    #[gpui_form(label = "Email", placeholder = "you@example.com", component(input))]
    #[koruma(EmailValidation::<_>::builder())]
    pub email: Option<String>,

    #[gpui_form(label = "Age", component(number_input))]
    #[koruma(RangeValidation::<_>::builder().min(1).max(130))]
    pub age: Option<u32>,

    #[gpui_form(component(select(searchable)), default = Country::France)]
    pub country: Option<Country>,

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

    #[gpui_form(skip)]
    pub session_id: u32,
}

struct FormsDemo {
    data: SignupFormValueHolder,
    fields: SignupFormFields,
    standalone_picker: Entity<DatePickerState>,
    file_picker: Entity<FilePickerState>,
    picked_date: Option<SharedString>,
    status: Option<SharedString>,
    focus_handle: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl FormsDemo {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let data = SignupFormValueHolder::default();
        let username_input = cx.new(|cx| SignupFormComponents::username_input(window, cx));
        let email_input = cx.new(|cx| SignupFormComponents::email_input(window, cx));
        let age_number_input = cx.new(|cx| SignupFormComponents::age_number_input(window, cx));
        let country_select = cx.new(|cx| SignupFormComponents::country_select(window, cx));
        let language_select = cx.new(|cx| SignupFormComponents::language_select(window, cx));
        let code_input = cx.new(|cx| SignupFormComponents::code_input(window, cx));
        let birth_date_date_picker =
            cx.new(|cx| SignupFormComponents::birth_date_date_picker(window, cx));
        let standalone_picker = cx.new(|cx| DatePickerState::new(window, cx));
        let file_picker = cx.new(|cx| FilePickerState::new(window, cx));

        let subscriptions = vec![
            cx.subscribe_in(&username_input, window, Self::on_username_change),
            cx.subscribe_in(&email_input, window, Self::on_email_change),
            cx.subscribe_in(&age_number_input, window, Self::on_age_change),
            cx.subscribe_in(&age_number_input, window, Self::on_age_step),
            cx.subscribe_in(&country_select, window, Self::on_country_confirm),
            cx.subscribe_in(&language_select, window, Self::on_language_confirm),
            cx.subscribe_in(&code_input, window, Self::on_code_change),
            cx.subscribe_in(&birth_date_date_picker, window, Self::on_birth_date_change),
            cx.subscribe_in(&standalone_picker, window, Self::on_standalone_pick),
            cx.subscribe_in(&file_picker, window, Self::on_file_pick),
        ];

        Self {
            data,
            fields: SignupFormFields {
                username_input,
                email_input,
                age_number_input,
                country_select,
                language_select,
                code_input,
                birth_date_date_picker,
            },
            standalone_picker,
            file_picker,
            picked_date: None,
            status: None,
            focus_handle: cx.focus_handle(),
            _subscriptions: subscriptions,
        }
    }

    fn on_username_change(
        &mut self,
        state: &Entity<InputState>,
        event: &InputEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let InputEvent::Change = event {
            let text = state.read(cx).value();
            self.data.username = if text.is_empty() {
                None
            } else {
                Some(text.to_string())
            };
        }
    }

    fn on_email_change(
        &mut self,
        state: &Entity<InputState>,
        event: &InputEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let InputEvent::Change = event {
            let text = state.read(cx).value();
            self.data.email = if text.is_empty() {
                None
            } else {
                Some(text.to_string())
            };
        }
    }

    fn on_age_change(
        &mut self,
        state: &Entity<InputState>,
        event: &InputEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let InputEvent::Change = event {
            let text = state.read(cx).value();
            self.data.age = text.parse::<u32>().ok();
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
            StepAction::Increment => self.data.age.unwrap_or_default().saturating_add(1),
            StepAction::Decrement => self.data.age.unwrap_or_default().saturating_sub(1),
        };
        self.data.age = Some(next);
        state.update(cx, |input, cx| {
            input.set_value(next.to_string(), window, cx);
        });
    }

    fn on_country_confirm(
        &mut self,
        _: &Entity<SelectState<SearchableVec<Country>>>,
        event: &SelectEvent<SearchableVec<Country>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) {
        let SelectEvent::Confirm(value) = event;
        self.data.country = value.clone();
    }

    fn on_language_confirm(
        &mut self,
        _: &Entity<SelectState<Vec<Language>>>,
        event: &SelectEvent<Vec<Language>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) {
        let SelectEvent::Confirm(value) = event;
        if let Some(value) = value {
            self.data.language = value.clone();
        }
    }

    fn on_code_change(
        &mut self,
        state: &Entity<InputState>,
        event: &InputEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let InputEvent::Change = event {
            let text = state.read(cx).value();
            self.data.code = if text.is_empty() {
                None
            } else {
                text.parse::<RequestCode>().ok()
            };
        }
    }

    fn on_birth_date_change(
        &mut self,
        _: &Entity<DatePickerState>,
        event: &DatePickerEvent,
        _: &mut Window,
        _: &mut Context<Self>,
    ) {
        let DatePickerEvent::Change(date) = event;
        self.data.birth_date = date.and_then(gpui_form::runtime::date_picker::parse_form_date);
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
        button.on_click(cx.listener(move |_, _, _, cx| {
            let lang_id: LanguageIdentifier = lang.parse().expect("valid language id");
            let _ = gpui_form::i18n::change_locale(cx, lang_id);
            cx.notify();
        }))
    }

    fn reset(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        *self = Self::new(window, cx);
        cx.notify();
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

impl Focusable for FormsDemo {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for FormsDemo {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let validation_errors = self.data.validate().err();
        let active_locale = rust_i18n::locale().to_string();
        let username_error = validation_errors.as_ref().and_then(|e| {
            localized_field_error(&e.username().all(), &active_locale, |v| match v {
                SignupFormValueHolderUsernameKorumaValidator::RequiredValidation(_) => {
                    "validation.required"
                },
                SignupFormValueHolderUsernameKorumaValidator::NonEmptyValidation(_) => {
                    "validation.non_empty"
                },
            })
        });
        let email_error = validation_errors.as_ref().and_then(|e| {
            localized_field_error(&e.email().all(), &active_locale, |v| match v {
                SignupFormValueHolderEmailKorumaValidator::EmailValidation(_) => "validation.email",
            })
        });
        let age_error = validation_errors.as_ref().and_then(|e| {
            localized_field_error(&e.age().all(), &active_locale, |v| match v {
                SignupFormValueHolderAgeKorumaValidator::RangeValidation(_) => "validation.range",
            })
        });
        let code_error = validation_errors.as_ref().and_then(|e| {
            localized_field_error(&e.code().all(), &active_locale, |v| match v {
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
        });
        let danger = cx.theme().danger;

        let mut form_state = gpui_form::FormState::new(SignupFormValueHolder::default());
        form_state.replace_current(self.data.clone());
        let dirty = form_state.is_dirty();

        v_flex()
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
                            .child(Input::new(&self.fields.username_input)),
                    )
                    .child(
                        field()
                            .label(t!(SignupFormValueHolder::EMAIL_LABEL_KEY).to_string())
                            .description_fn(move |_, _| match &email_error {
                                Some(error) => div()
                                    .text_color(danger)
                                    .child(error.clone())
                                    .into_any_element(),
                                None => div().into_any_element(),
                            })
                            .child(Input::new(&self.fields.email_input)),
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
                            .child(NumberInput::new(&self.fields.age_number_input)),
                    )
                    .child(
                        field()
                            .label(t!(SignupFormValueHolder::COUNTRY_LABEL_KEY).to_string())
                            .child(Select::new(&self.fields.country_select)),
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
                                    .checked(self.data.newsletter)
                                    .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                        this.data.newsletter = *checked;
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(
                        field()
                            .label(t!(SignupFormValueHolder::NOTIFICATIONS_LABEL_KEY).to_string())
                            .child(
                                Switch::new("notifications-switch")
                                    .checked(self.data.notifications)
                                    .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                        this.data.notifications = *checked;
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
                            .child(Input::new(&self.fields.code_input)),
                    )
                    .child(
                        field()
                            .label(t!(SignupFormValueHolder::BIRTH_DATE_LABEL_KEY).to_string())
                            .child(DatePicker::new(&self.fields.birth_date_date_picker)),
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
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.status = Some(match this.data.validate() {
                                    Ok(_) => t!("app.all_valid").to_string().into(),
                                    Err(_) => t!("app.invalid").to_string().into(),
                                });
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("reset")
                            .label(t!("app.reset").to_string())
                            .on_click(cx.listener(|this, _, window, cx| this.reset(window, cx))),
                    ),
            )
            .child(
                v_flex()
                    .gap_1()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("locale: {}", &*rust_i18n::locale()))
                    .child(format!("{}: {dirty}", t!("app.dirty")))
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
                            SignupFormPath::language().to_string(),
                            SignupFormPath::newsletter().to_string(),
                            SignupFormPath::notifications().to_string(),
                            SignupFormPath::code().to_string(),
                            SignupFormPath::birth_date().to_string(),
                        ]
                        .join(", "),
                    )
                    .children(self.status.clone().map(|s| div().child(s))),
            )
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
        "app.paths_note",
        "signup.username_label",
        "signup.email_label",
        "signup.age_label",
        "signup.country_label",
        "signup.language_label",
        "signup.newsletter_label",
        "signup.notifications_label",
        "signup.code_label",
        "signup.birth_date_label",
        "country.united_states",
        "country.france",
        "country.china",
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
        let mut holder = SignupFormValueHolder::default();
        holder.username = Some("ada".to_string());
        holder.code = Some(RequestCode::from("A".to_string()));
        let err = holder
            .validate()
            .err()
            .expect("short invite code must fail");
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
