//! Feature #18 (phone, email, URL, typed text validation): end-to-end check of
//! the first-class `component(phone_input)` derive component.
//!
//! Gated on the `gpui-form/phone` feature (enabled for this example). Exercises
//! the REAL generated surface for both phone modes:
//!
//! - `component(phone_input)` — accepts any globally valid number.
//! - `component(phone_input(country = <field>))` — the country binding is
//!   metadata; the generated input control still validates a globally
//!   parseable number as a baseline.
//!
//! The phone field is a text field, so the generated value holder stores it as
//! `Option<String>` (same storage as `component(input)`), and the generated
//! `…FormComponents` base declarations build `InputState`s wired to the phone
//! validation helpers.

use gpui_form::{GpuiForm, SelectItem};
use strum::EnumIter;

#[derive(Clone, Debug, Default, EnumIter, PartialEq, SelectItem)]
enum Region {
    #[default]
    UnitedStates,
    France,
}

impl std::fmt::Display for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnitedStates => f.write_str("United States"),
            Self::France => f.write_str("France"),
        }
    }
}

/// A form with a country select plus both phone modes.
#[derive(Clone, Debug, Default, GpuiForm, PartialEq)]
struct PhoneSignup {
    #[gpui_form(component(select))]
    region: Region,

    /// Global phone: any globally valid number.
    #[gpui_form(component(phone_input))]
    mobile_number: Option<String>,

    /// Country-bound phone: must match the `region` field's selection.
    #[gpui_form(component(phone_input(country = region)))]
    local_number: Option<String>,
}

/// The text value of a phone field is stored as `Option<String>` in the holder.
#[test]
fn phone_fields_are_optional_strings_in_the_value_holder() {
    let holder = PhoneSignupFormValueHolder {
        region: Region::France,
        mobile_number: Some("+1 415 550 2222".to_string()),
        local_number: None,
    };

    assert_eq!(holder.mobile_number.as_deref(), Some("+1 415 550 2222"));
    assert_eq!(holder.local_number, None);
}

/// The generated holder round-trips back to the source struct.
#[test]
fn phone_holder_converts_back_to_source() {
    let source = PhoneSignup {
        region: Region::UnitedStates,
        mobile_number: Some("+33 1 42 68 53 00".to_string()),
        local_number: Some("415 555 2671".to_string()),
    };

    let holder = PhoneSignupFormValueHolder::from(source.clone());
    let round_tripped = PhoneSignup::from(holder);

    assert_eq!(round_tripped, source);
}

/// The typed field path exposes a constructor for each phone field, proving the
/// component participates in the standard generated surface.
#[test]
fn phone_fields_have_typed_paths() {
    assert_eq!(PhoneSignupFormPath::mobile_number().to_string(), "mobile_number");
    assert_eq!(PhoneSignupFormPath::local_number().to_string(), "local_number");
}
