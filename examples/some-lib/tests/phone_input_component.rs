//! Feature #18 (phone, email, URL, typed text validation): end-to-end check of
//! the `PhoneInput` component shape.
//!
//! Gated on the `gpui-form/phone` feature (enabled for this example).
//! Exercises the REAL generated surface for the phone shape:
//!
//! `component(gpui_form_collection::phone_input::PhoneInput)` — a text input
//! whose widget-level validation accepts any globally parseable number (or an
//! empty field mid-entry). Structured validation — required-ness, country
//! matching, typed [`gpui_form::phone::ValidatedPhoneNumber`] conversion —
//! lives in the `gpui_form::phone` helpers and is exercised by the Phone
//! Verification story.
//!
//! An `Option<String>` phone field stores `Option<String>` in the generated
//! value holder (same storage as `component(Input::<_>)`).

use gpui_form::{GpuiForm, phone::validate_phone_number_for_country_label};
use strum::EnumIter;

#[derive(Clone, Debug, Default, EnumIter, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

impl gpui_component::select::SelectItem for Region {
    type Value = Self;

    fn title(&self) -> gpui::SharedString {
        self.to_string().into()
    }

    fn value(&self) -> &Self::Value {
        self
    }
}

/// A form with a country select plus the phone shape.
#[derive(Clone, Debug, Default, GpuiForm, PartialEq)]
struct PhoneSignup {
    #[gpui_form(component(gpui_form_collection::select::Select::<_>))]
    region: Region,

    /// Global phone: any globally valid number.
    #[gpui_form(component(gpui_form_collection::phone_input::PhoneInput))]
    mobile_number: Option<String>,
}

/// The text value of a phone field is stored as `Option<String>` in the holder.
#[test]
fn phone_fields_are_optional_strings_in_the_value_holder() {
    let holder = PhoneSignupFormValueHolder {
        region: Region::France,
        mobile_number: Some("+1 415 550 2222".to_string()),
    };

    assert_eq!(holder.mobile_number.as_deref(), Some("+1 415 550 2222"));
}

/// The generated holder round-trips back to the source struct.
#[test]
fn phone_holder_converts_back_to_source() {
    let source = PhoneSignup {
        region: Region::UnitedStates,
        mobile_number: Some("+33 1 42 68 53 00".to_string()),
    };

    let holder = PhoneSignupFormValueHolder::from(source.clone());
    let round_tripped = PhoneSignup::try_from(holder).expect("holder converts back");

    assert_eq!(round_tripped, source);
}

/// The typed field path exposes a constructor for the phone field, proving the
/// shape participates in the standard generated surface.
#[test]
fn phone_fields_have_typed_paths() {
    assert_eq!(
        PhoneSignupFormPath::mobile_number().to_string(),
        "mobile_number"
    );
    assert_eq!(
        PhoneSignupFormPath::mobile_number(),
        PhoneSignupFormPath::from_form_field(PhoneSignupFormField::MobileNumber)
    );
}

/// Country matching stays a validation-layer concern: the `gpui_form::phone`
/// helpers bind the same raw text to the selected country.
#[test]
fn country_bound_validation_uses_phone_helpers() {
    let valid = validate_phone_number_for_country_label(
        "01 42 68 53 00",
        gpui_form::phone::country::Id::FR,
        "France",
    );
    assert!(valid.is_valid(), "French number must validate for France");

    let mismatch = validate_phone_number_for_country_label(
        "415 555 2671",
        gpui_form::phone::country::Id::FR,
        "France",
    );
    assert!(
        !mismatch.is_valid(),
        "US number must fail the France binding"
    );
}
