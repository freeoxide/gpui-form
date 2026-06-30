use gpui::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ParentElement as _, Render, Styled, Subscription, Window, div,
};
use gpui_component::ActiveTheme as _;
use gpui_component::form::{field, v_form};
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::select::{Select, SelectEvent, SelectItem, SelectState};
use gpui_component::separator::Separator;
use gpui_component::v_flex;
use gpui_form::phone::{
    PhoneCountry as PhoneCountryExt, PhoneNumberValidation, country, validate_phone_number_for,
    validate_optional_phone_number,
};
use strum::IntoEnumIterator as _;

const CONTEXT: &str = "PhoneVerificationForm";

#[derive(Clone, Debug, Default, Eq, PartialEq, strum::EnumIter)]
enum PhoneCountry {
    #[default]
    UnitedStates,
    France,
    China,
}

// Implement the library's `PhoneCountry` trait so the validation helpers can map
// the example enum to a libphonenumber id and label without an ad hoc `match` at
// each call site.
impl PhoneCountryExt for PhoneCountry {
    fn phone_country_id(&self) -> country::Id {
        match self {
            Self::UnitedStates => country::US,
            Self::France => country::FR,
            Self::China => country::CN,
        }
    }

    fn phone_country_label(&self) -> String {
        self.to_string()
    }
}

impl core::fmt::Display for PhoneCountry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnitedStates => f.write_str("United States"),
            Self::France => f.write_str("France"),
            Self::China => f.write_str("China"),
        }
    }
}

impl SelectItem for PhoneCountry {
    type Value = Self;

    fn title(&self) -> gpui::SharedString {
        self.to_string().into()
    }

    fn value(&self) -> &Self::Value {
        self
    }
}

/// A parsed breakdown of a phone validation result for separate, polished
/// display: a status line, the parsed country, and the normalized E.164 form.
struct PhoneBreakdown {
    is_valid: bool,
    status: String,
    parsed_country: String,
    e164: String,
}

impl PhoneBreakdown {
    fn from_validation(validation: &PhoneNumberValidation) -> Self {
        let status = match validation {
            PhoneNumberValidation::Empty => "Waiting for input".to_string(),
            PhoneNumberValidation::Valid(_) => "Valid".to_string(),
            PhoneNumberValidation::Invalid(error) => error.to_string(),
        };
        let parsed_country = validation
            .country()
            .map(|id| format!("{id:?}"))
            .unwrap_or_else(|| "—".to_string());
        let e164 = validation.e164().map(str::to_string).unwrap_or_else(|| "—".to_string());

        Self {
            is_valid: validation.is_valid(),
            status,
            parsed_country,
            e164,
        }
    }
}

fn validate_country_phone(country: &PhoneCountry, raw: &str) -> PhoneNumberValidation {
    validate_phone_number_for(raw, country)
}

fn validate_global_phone(raw: &str) -> PhoneNumberValidation {
    validate_optional_phone_number(raw, None)
}

#[gpui_storybook::story_init]
pub fn init(_cx: &mut App) {}

#[gpui_storybook::story]
pub struct PhoneVerificationForm {
    country: PhoneCountry,
    country_phone: String,
    global_phone: String,
    country_select: Entity<SelectState<Vec<PhoneCountry>>>,
    country_phone_input: Entity<InputState>,
    global_phone_input: Entity<InputState>,
    focus_handle: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl Focusable for PhoneVerificationForm {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl gpui_storybook::Story for PhoneVerificationForm {
    fn title(_cx: &gpui::App) -> String {
        "Phone Verification".into()
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        cx.new(|cx| Self::new(window, cx))
    }
}

impl PhoneVerificationForm {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let country = PhoneCountry::UnitedStates;
        let country_select = cx.new(|cx| {
            SelectState::new(
                PhoneCountry::iter().collect::<Vec<_>>(),
                Some(gpui_component::IndexPath::new(0)),
                window,
                cx,
            )
        });
        let country_phone_input = cx.new(|cx| InputState::new(window, cx));
        let global_phone_input = cx.new(|cx| InputState::new(window, cx));

        let _subscriptions = vec![
            cx.subscribe_in(&country_select, window, Self::on_country_select_event),
            cx.subscribe_in(
                &country_phone_input,
                window,
                Self::on_country_phone_input_event,
            ),
            cx.subscribe_in(
                &global_phone_input,
                window,
                Self::on_global_phone_input_event,
            ),
        ];

        Self {
            country,
            country_phone: String::new(),
            global_phone: String::new(),
            country_select,
            country_phone_input,
            global_phone_input,
            focus_handle: cx.focus_handle(),
            _subscriptions,
        }
    }

    fn on_country_select_event(
        &mut self,
        _this: &Entity<SelectState<Vec<PhoneCountry>>>,
        event: &SelectEvent<Vec<PhoneCountry>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            SelectEvent::Confirm(Some(country)) => {
                self.country = country.clone();
                cx.notify();
            },
            SelectEvent::Confirm(None) => {},
        }
    }

    fn on_country_phone_input_event(
        &mut self,
        state: &Entity<InputState>,
        event: &InputEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            InputEvent::Change => {
                self.country_phone = state.read(cx).value().to_string();
                cx.notify();
            },
            _ => {},
        }
    }

    fn on_global_phone_input_event(
        &mut self,
        state: &Entity<InputState>,
        event: &InputEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            InputEvent::Change => {
                self.global_phone = state.read(cx).value().to_string();
                cx.notify();
            },
            _ => {},
        }
    }
}

impl PhoneVerificationForm {
    /// Render a validation breakdown as three separate rows — status, parsed
    /// country, and normalized E.164 — instead of a single status string.
    fn render_breakdown(breakdown: &PhoneBreakdown, cx: &Context<Self>) -> impl IntoElement {
        let status_color = if breakdown.is_valid {
            cx.theme().success
        } else {
            cx.theme().danger
        };
        let muted = cx.theme().muted_foreground;

        v_flex()
            .gap_1()
            .child(
                div()
                    .text_color(status_color)
                    .child(format!("Status: {}", breakdown.status)),
            )
            .child(
                div()
                    .text_color(muted)
                    .child(format!("Parsed country: {}", breakdown.parsed_country)),
            )
            .child(
                div()
                    .text_color(muted)
                    .child(format!("E.164: {}", breakdown.e164)),
            )
    }
}

impl Render for PhoneVerificationForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let country_validation = validate_country_phone(&self.country, &self.country_phone);
        let country_breakdown = PhoneBreakdown::from_validation(&country_validation);

        let global_validation = validate_global_phone(&self.global_phone);
        let global_breakdown = PhoneBreakdown::from_validation(&global_validation);

        v_flex()
            .key_context(CONTEXT)
            .id("phone-verification-form")
            .size_full()
            .p_4()
            .justify_start()
            .gap_3()
            .child(Separator::horizontal())
            .child(
                v_form()
                    .child(
                        field()
                            .label("Country")
                            .description(
                                "Only the country-bound phone field uses this selection.",
                            )
                            .child(Select::new(&self.country_select)),
                    )
                    .child(
                        field()
                            .label("Country-bound phone number")
                            .description(
                                "Must match the selected country. Try +1 415 550 2222 with France selected.",
                            )
                            .child(Input::new(&self.country_phone_input)),
                    )
                    .child(
                        field()
                            .label("Country-bound validation")
                            .child(Self::render_breakdown(&country_breakdown, cx)),
                    )
                    .child(
                        field()
                            .label("Global phone number")
                            .description(
                                "No country match required. International numbers like +1 415 550 2222 are accepted if globally valid.",
                            )
                            .child(Input::new(&self.global_phone_input)),
                    )
                    .child(
                        field()
                            .label("Global validation")
                            .child(Self::render_breakdown(&global_breakdown, cx)),
                    ),
            )
            .child(Separator::horizontal())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn us_number_validates_for_us_not_france() {
        let number = "415 555 2671";

        assert!(matches!(
            validate_country_phone(&PhoneCountry::UnitedStates, number),
            PhoneNumberValidation::Valid(_)
        ));
        assert!(matches!(
            validate_country_phone(&PhoneCountry::France, number),
            PhoneNumberValidation::Invalid(_)
        ));
    }

    #[test]
    fn international_us_number_does_not_validate_when_france_is_selected() {
        let number = "+1 415 550 2222";

        assert!(matches!(
            validate_country_phone(&PhoneCountry::UnitedStates, number),
            PhoneNumberValidation::Valid(_)
        ));
        assert!(matches!(
            validate_country_phone(&PhoneCountry::France, number),
            PhoneNumberValidation::Invalid(_)
        ));
    }

    #[test]
    fn france_number_validates_for_france_not_us() {
        let number = "01 42 68 53 00";

        assert!(matches!(
            validate_country_phone(&PhoneCountry::France, number),
            PhoneNumberValidation::Valid(_)
        ));
        assert!(matches!(
            validate_country_phone(&PhoneCountry::UnitedStates, number),
            PhoneNumberValidation::Invalid(_)
        ));
    }

    #[test]
    fn general_mode_accepts_valid_number_from_non_selected_country() {
        let number = "+1 415 550 2222";

        assert!(matches!(
            validate_global_phone(number),
            PhoneNumberValidation::Valid(_)
        ));
    }
}
