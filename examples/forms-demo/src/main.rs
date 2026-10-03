#![recursion_limit = "1024"]

use gpui::{
    App, AppContext as _, Context, Entity, FocusHandle, Focusable, IntoElement, ParentElement as _,
    Render, SharedString, Styled as _, Subscription, Window, div,
};
use gpui_form::runtime::date_picker::{DatePicker, DatePickerEvent, DatePickerState};
use gpui_form::{GpuiForm, SelectItem};
use gpui_kit::assets::Assets;
use gpui_kit::component::{
    ActiveTheme as _, Sizable as _,
    button::{Button, ButtonVariants as _},
    form::{field, v_form},
    h_flex, v_flex, *,
};
use gpui_kit::component::{
    input::{Input, InputEvent, InputState, NumberInput, NumberInputEvent, StepAction},
    select::{SearchableVec, Select, SelectEvent, SelectState},
};
use gpui_kit::*;
use koruma::{Koruma, KorumaAllDisplay};
use koruma_collection::{collection::NonEmptyValidation, numeric::RangeValidation};
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

#[derive(Clone, Debug, Default, GpuiForm, Koruma, KorumaAllDisplay)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[gpui_form(koruma(fluent))]
pub struct Signup {
    #[gpui_form(label = "Username", placeholder = "ada", component(input))]
    #[koruma(NonEmptyValidation::<_>::builder())]
    pub username: String,

    #[gpui_form(label = "Age", component(number_input))]
    #[koruma(RangeValidation::<_>::builder().min(1).max(130))]
    pub age: Option<u32>,

    #[gpui_form(component(select(searchable)), default = Country::France)]
    pub country: Option<Country>,
}

struct FormsDemo {
    data: SignupFormValueHolder,
    fields: SignupFormFields,
    birth_picker: Entity<DatePickerState>,
    picked_date: Option<SharedString>,
    status: Option<SharedString>,
    focus_handle: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl FormsDemo {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let data = SignupFormValueHolder::default();
        let username_input = cx.new(|cx| SignupFormComponents::username_input(window, cx));
        let age_number_input = cx.new(|cx| SignupFormComponents::age_number_input(window, cx));
        let country_select = cx.new(|cx| SignupFormComponents::country_select(window, cx));
        let birth_picker = cx.new(|cx| DatePickerState::new(window, cx));

        let subscriptions = vec![
            cx.subscribe_in(&username_input, window, Self::on_username_change),
            cx.subscribe_in(&age_number_input, window, Self::on_age_change),
            cx.subscribe_in(&age_number_input, window, Self::on_age_step),
            cx.subscribe_in(&country_select, window, Self::on_country_confirm),
            cx.subscribe_in(&birth_picker, window, Self::on_birth_change),
        ];

        Self {
            data,
            fields: SignupFormFields {
                username_input,
                age_number_input,
                country_select,
            },
            birth_picker,
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

    fn on_birth_change(
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

impl Focusable for FormsDemo {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

fn localized_field_error<E>(errs: &[E], key_of: impl Fn(&E) -> &'static str) -> Option<String> {
    (!errs.is_empty()).then(|| {
        errs.iter()
            .map(|v| t!(key_of(v)).to_string())
            .collect::<Vec<_>>()
            .join("\n")
    })
}

impl Render for FormsDemo {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let validation_errors = self.data.validate().err();
        let username_error = validation_errors.as_ref().and_then(|e| {
            localized_field_error(&e.username().all(), |v| match v {
                SignupFormValueHolderUsernameKorumaValidator::RequiredValidation(_) => {
                    "validation.required"
                },
                SignupFormValueHolderUsernameKorumaValidator::NonEmptyValidation(_) => {
                    "validation.non_empty"
                },
            })
        });
        let age_error = validation_errors.as_ref().and_then(|e| {
            localized_field_error(&e.age().all(), |v| match v {
                SignupFormValueHolderAgeKorumaValidator::RangeValidation(_) => "validation.range",
            })
        });
        let danger = cx.theme().danger;

        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .p_4()
            .gap_3()
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
                                DatePicker::new(&self.birth_picker)
                                    .cleanable(true)
                                    .number_of_months(1),
                            ),
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
                    .children(self.status.clone().map(|s| div().child(s))),
            )
    }
}

fn main() {
    gpui_kit::application().with_assets(Assets).run(|cx| {
        gpui_kit::init(cx);
        gpui_form::i18n::init(cx);

        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::centered(size(px(760.), px(720.)), cx)),
            ..Default::default()
        };
        gpui_kit::open_window(window_options, cx, |window, cx| {
            cx.new(|cx| FormsDemo::new(window, cx))
        })
        .expect("failed to open the forms-demo window");
    });
}

#[cfg(test)]
mod tests {
    use crate::SignupFormValueHolder;
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
        "app.picked",
        "signup.username_label",
        "signup.age_label",
        "signup.country_label",
        "country.united_states",
        "country.france",
        "country.china",
        "validation.required",
        "validation.non_empty",
        "validation.range",
        "validation.invalid",
    ];

    #[test]
    fn every_key_resolves_in_every_locale() {
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
    }
}

#[cfg(test)]
mod validation_locale_tests {
    use crate::SignupFormValueHolder;
    use crate::SignupFormValueHolderUsernameKorumaValidator;
    use crate::localized_field_error;

    #[test]
    fn field_errors_render_in_french() {
        rust_i18n::set_locale("fr-FR");
        let err = SignupFormValueHolder::default().validate().err().unwrap();
        let errs = err.username().all();
        let rendered = localized_field_error(&errs, |v| match v {
            SignupFormValueHolderUsernameKorumaValidator::RequiredValidation(_) => {
                "validation.required"
            },
            SignupFormValueHolderUsernameKorumaValidator::NonEmptyValidation(_) => {
                "validation.non_empty"
            },
        })
        .unwrap();
        assert_eq!(rendered, "Ce champ est obligatoire.");

        rust_i18n::set_locale("zh-CN");
        let errs = err.username().all();
        let rendered = localized_field_error(&errs, |v| match v {
            SignupFormValueHolderUsernameKorumaValidator::RequiredValidation(_) => {
                "validation.required"
            },
            SignupFormValueHolderUsernameKorumaValidator::NonEmptyValidation(_) => {
                "validation.non_empty"
            },
        })
        .unwrap();
        assert_eq!(rendered, "此字段为必填项。");
        rust_i18n::set_locale("en");
    }
}
