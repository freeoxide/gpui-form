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
use gpui_form::phone::{PhoneNumberValidation, country, validate_phone_number_for_country_label};
use strum::IntoEnumIterator as _;

const CONTEXT: &str = "PhoneVerificationForm";

#[derive(Clone, Debug, Default, Eq, PartialEq, strum::EnumIter)]
enum PhoneCountry {
    #[default]
    UnitedStates,
    France,
    China,
}

impl PhoneCountry {
    fn country_id(&self) -> country::Id {
        match self {
            Self::UnitedStates => country::US,
            Self::France => country::FR,
            Self::China => country::CN,
        }
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

fn validate_phone(country: PhoneCountry, raw: &str) -> PhoneNumberValidation {
    validate_phone_number_for_country_label(raw, country.country_id(), country.to_string())
}

#[gpui_storybook::story_init]
pub fn init(_cx: &mut App) {}

#[gpui_storybook::story]
pub struct PhoneVerificationForm {
    country: PhoneCountry,
    phone: String,
    country_select: Entity<SelectState<Vec<PhoneCountry>>>,
    phone_input: Entity<InputState>,
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
        let phone_input = cx.new(|cx| InputState::new(window, cx));

        let _subscriptions = vec![
            cx.subscribe_in(&country_select, window, Self::on_country_select_event),
            cx.subscribe_in(&phone_input, window, Self::on_phone_input_event),
        ];

        Self {
            country,
            phone: String::new(),
            country_select,
            phone_input,
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

    fn on_phone_input_event(
        &mut self,
        state: &Entity<InputState>,
        event: &InputEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            InputEvent::Change => {
                self.phone = state.read(cx).value().to_string();
                cx.notify();
            },
            _ => {},
        }
    }
}

impl Render for PhoneVerificationForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let validation = validate_phone(self.country.clone(), &self.phone);
        let status = validation.message();
        let valid = validation.is_valid();
        let status_color = if valid {
            cx.theme().success
        } else {
            cx.theme().danger
        };

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
                            .description("Changing this revalidates the current phone text.")
                            .child(Select::new(&self.country_select)),
                    )
                    .child(
                        field()
                            .label("Phone number")
                            .description(
                                "Try 415 555 2671 for United States, then switch to France.",
                            )
                            .child(Input::new(&self.phone_input)),
                    )
                    .child(
                        field()
                            .label("Parser-backed validation")
                            .child(div().text_color(status_color).child(status)),
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
            validate_phone(PhoneCountry::UnitedStates, number),
            PhoneNumberValidation::Valid(_)
        ));
        assert!(matches!(
            validate_phone(PhoneCountry::France, number),
            PhoneNumberValidation::Invalid(_)
        ));
    }

    #[test]
    fn international_us_number_does_not_validate_when_france_is_selected() {
        let number = "+1 415 550 2222";

        assert!(matches!(
            validate_phone(PhoneCountry::UnitedStates, number),
            PhoneNumberValidation::Valid(_)
        ));
        assert!(matches!(
            validate_phone(PhoneCountry::France, number),
            PhoneNumberValidation::Invalid(_)
        ));
    }

    #[test]
    fn france_number_validates_for_france_not_us() {
        let number = "01 42 68 53 00";

        assert!(matches!(
            validate_phone(PhoneCountry::France, number),
            PhoneNumberValidation::Valid(_)
        ));
        assert!(matches!(
            validate_phone(PhoneCountry::UnitedStates, number),
            PhoneNumberValidation::Invalid(_)
        ));
    }
}
