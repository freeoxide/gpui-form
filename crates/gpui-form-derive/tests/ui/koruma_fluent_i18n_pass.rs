use gpui_form_derive::{GpuiForm, SelectItem};
use gpui_kit::component::select::SelectItem as _;

rust_i18n::i18n!("locales", fallback = "en");

#[derive(GpuiForm)]
#[gpui_form(koruma(fluent))]
struct SignupForm {
    #[gpui_form(component(number_input))]
    #[koruma(koruma_collection::numeric::RangeValidation::<_>::builder().min(1).max(12))]
    seats: u32,

    #[gpui_form(component(input))]
    note: String,
}

#[derive(GpuiForm)]
#[gpui_form(koruma(fluent))]
struct OptionalOnlyForm {
    note: Option<String>,
    kind: Option<u8>,
}

#[derive(Clone, Debug, PartialEq, SelectItem)]
#[select_item(fluent)]
enum PlanTier {
    Basic,
    Professional,
}

fn main() {
    assert_eq!(
        SignupFormFormValueHolder::SIGNUP_FORM_I18N_KEY_PREFIX,
        "signup_form"
    );
    assert_eq!(
        SignupFormFormValueHolder::SEATS_LABEL_KEY,
        "signup_form.seats_label"
    );
    assert_eq!(
        SignupFormFormValueHolder::NOTE_LABEL_KEY,
        "signup_form.note_label"
    );
    assert_eq!(
        SignupFormFormValueHolder::validation_issue_key("RangeValidation"),
        "validation.range"
    );
    assert_eq!(
        SignupFormFormValueHolder::validation_issue_key("MysteryValidation"),
        "validation.invalid"
    );
    assert_eq!(
        &*SignupFormFormValueHolder::localized_validation_issue("RangeValidation"),
        "Must be in the range %{left_delimiter}%{min}, %{max}%{right_delimiter}."
    );
    assert_eq!(
        &*rust_i18n::t!("signup_form.note_label"),
        "Additional notes"
    );
    assert_eq!(
        OptionalOnlyFormFormValueHolder::OPTIONAL_ONLY_FORM_I18N_KEY_PREFIX,
        "optional_only_form"
    );
    assert_eq!(
        OptionalOnlyFormFormValueHolder::NOTE_LABEL_KEY,
        "optional_only_form.note_label"
    );
    assert_eq!(
        OptionalOnlyFormFormValueHolder::validation_issue_key("RangeValidation"),
        "validation.invalid",
        "forms without validators emit only the fallback arm"
    );
    assert!(
        OptionalOnlyFormFormValueHolder::default()
            .validate()
            .is_ok(),
        "koruma(fluent) without validators must still derive a working validate()"
    );
    assert_eq!(&*PlanTier::Basic.title(), "Basic plan");
    assert_eq!(&*PlanTier::Professional.title(), "Professional plan");

    let holder: SignupFormFormValueHolder = SignupForm {
        seats: 3,
        note: "hello".to_string(),
    }
    .into();
    let _ = format!("{holder:?}");
}
