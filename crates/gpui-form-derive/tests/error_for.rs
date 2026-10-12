//! Field-level state model (M1, feature: per-field typed error access,
//! backlog #6): koruma-gated holders gain `validate_fields()`, the
//! `<Name>FormFieldErrors` map, and the `<Name>FormFieldError` borrowing
//! enum, all keyed by `<Name>FormPath`.
//!
//! Mirrors the struct shapes in `koruma_validation_behavior.rs`, covering
//! BOTH koruma enablement routes:
//! - field-attr-only koruma (`#[koruma(...)]` on fields, no struct-level
//!   `#[gpui_form(koruma)]`) — the SeatsForm-style case that a
//!   `koruma_options.is_some()` gate would silently drop;
//! - struct-level `#[gpui_form(koruma)]` (NameForm-style, required-only).

use gpui_form::core::FormFieldErrors;
use gpui_form_derive::GpuiForm;

/// Field-attr-only koruma (no struct-level `#[gpui_form(koruma)]`).
#[derive(GpuiForm, koruma::Koruma)]
struct SeatsForm {
    #[gpui_form(component(number_input))]
    #[koruma(koruma_collection::numeric::RangeValidation::<_>::builder().min(1).max(12))]
    seats: u32,
}

/// Struct-level koruma with synthetic required only.
#[derive(GpuiForm, koruma::Koruma)]
#[gpui_form(koruma)]
struct NameForm {
    #[gpui_form(component(input))]
    name: String,
}

/// Newtype WITH validators (synthetic required + inner) — the wrapper-shape
/// getter (`&<Holder><Field>KorumaValidationError` with `all()`/`inner()`).
/// Struct-level `#[koruma(newtype)]` mirrors the demo's `RequestCode` shape.
#[derive(Clone, Debug, Default, PartialEq, Eq, koruma::Koruma)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[koruma(newtype)]
pub struct RequestCode {
    #[koruma(koruma_collection::collection::LenValidation::<_>::builder().min(2).max(8))]
    pub value: String,
}

impl ::core::str::FromStr for RequestCode {
    type Err = ();
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            value: text.to_string(),
        })
    }
}

#[derive(GpuiForm, koruma::Koruma)]
struct CodeForm {
    #[gpui_form(component(input))]
    #[koruma(newtype)]
    code: RequestCode,
}

/// Bare newtype (originally optional, no mirrored validators) — the inner
/// error surfaces directly behind an `Option<&..>` getter.
#[derive(Clone, Debug, Default, PartialEq, Eq, koruma::Koruma)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[koruma(newtype)]
pub struct Ticket {
    #[koruma(koruma_collection::collection::NonEmptyValidation::<_>::builder())]
    pub id: String,
}

impl ::core::str::FromStr for Ticket {
    type Err = ();
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            id: text.to_string(),
        })
    }
}

#[derive(GpuiForm, koruma::Koruma)]
struct TicketForm {
    #[gpui_form(component(input))]
    #[koruma(newtype)]
    ticket: Option<Ticket>,
}

#[test]
fn error_for_surfaces_range_failure_and_none_when_valid() {
    // Field-attr-only koruma form: the error surface must still be emitted
    // (enable_koruma is true via parsed_koruma_fields even though
    // koruma_options is None).
    let failing = SeatsFormFormValueHolder { seats: Some(13) };
    let errors = failing.validate_fields();
    assert!(errors.has_errors());
    let field_error = errors
        .error_for(&SeatsFormFormPath::seats())
        .expect("range failure must surface through the typed path");
    assert_eq!(field_error.path().to_string(), "seats");
    assert_eq!(field_error.issue_keys(), vec!["validation.range"]);

    let passing = SeatsFormFormValueHolder { seats: Some(5) };
    let errors = passing.validate_fields();
    assert!(errors.is_empty());
    assert!(errors.error_for(&SeatsFormFormPath::seats()).is_none());
}

#[test]
fn required_failure_reports_required_key() {
    let errors = SeatsFormFormValueHolder::default().validate_fields();
    let field_error = errors
        .error_for(&SeatsFormFormPath::seats())
        .expect("absent default must fail synthetic required");
    assert_eq!(field_error.issue_keys(), vec!["validation.required"]);
}

#[test]
fn validate_fields_agrees_with_raw_koruma_validate() {
    let holder = SeatsFormFormValueHolder { seats: Some(0) };
    assert_eq!(
        holder.validate_fields().is_empty(),
        holder.validate().is_ok()
    );
    let holder = SeatsFormFormValueHolder { seats: Some(2) };
    assert_eq!(
        holder.validate_fields().is_empty(),
        holder.validate().is_ok()
    );
}

#[test]
fn unknown_and_wrong_name_paths_return_none() {
    let errors = SeatsFormFormValueHolder { seats: Some(13) }.validate_fields();
    assert!(errors.has_errors());
    assert!(
        errors
            .error_for(&SeatsFormFormPath::new(&["nope"]))
            .is_none()
    );
    assert!(
        errors
            .error_for(&SeatsFormFormPath::new(&["seats", "deep"]))
            .is_none()
    );
}

#[test]
fn struct_level_koruma_form_gains_the_surface() {
    // NameForm enables koruma via #[gpui_form(koruma)]: required-only.
    let errors = NameFormFormValueHolder::default().validate_fields();
    let field_error = errors
        .error_for(&NameFormFormPath::name())
        .expect("empty name must fail synthetic required");
    assert_eq!(field_error.issue_keys(), vec!["validation.required"]);

    let ok = NameFormFormValueHolder {
        name: Some("ada".to_string()),
    }
    .validate_fields();
    assert!(ok.is_empty());
    assert!(ok.error_for(&NameFormFormPath::name()).is_none());
}

#[test]
fn typed_accessors_mirror_koruma_getters() {
    let errors = SeatsFormFormValueHolder { seats: Some(13) }.validate_fields();
    let typed = errors
        .seats()
        .expect("typed accessor mirrors koruma getter");
    assert!(!typed.is_empty());
    assert!(
        typed.range_validation().is_some(),
        "range getter survived re-keying"
    );
}

#[test]
fn failing_fields_and_failing_paths_are_declaration_ordered() {
    let errors = CodeFormFormValueHolder::default().validate_fields();
    let failing = errors.failing_fields().collect::<Vec<_>>();
    assert_eq!(failing.len(), 1);
    assert_eq!(failing[0].0.to_string(), "code");

    fn generic_paths<E: FormFieldErrors>(errors: &E) -> Vec<String> {
        errors
            .failing_paths()
            .iter()
            .map(|path| path.to_string())
            .collect()
    }
    assert_eq!(generic_paths(&errors), vec!["code"]);
}

#[test]
fn newtype_with_validators_reports_required_and_generic_inner_key() {
    let missing = CodeFormFormValueHolder::default().validate_fields();
    let field_error = missing
        .error_for(&CodeFormFormPath::code())
        .expect("absent newtype must fail synthetic required");
    assert_eq!(field_error.issue_keys(), vec!["validation.required"]);

    // Present but failing inner validators: the derive cannot see inner
    // kinds, so the generic invalid key is reported.
    let holder = CodeFormFormValueHolder {
        code: Some(RequestCode {
            value: "A".to_string(),
        }),
    };
    let failing = holder.validate_fields();
    let field_error = failing
        .error_for(&CodeFormFormPath::code())
        .expect("short code must fail");
    assert_eq!(field_error.issue_keys(), vec!["validation.invalid"]);
}

#[test]
fn bare_newtype_on_optional_field_surfaces_inner_error() {
    let holder = TicketFormFormValueHolder {
        ticket: Some(Ticket { id: String::new() }),
    };
    let errors = holder.validate_fields();
    let field_error = errors
        .error_for(&TicketFormFormPath::ticket())
        .expect("empty ticket id must fail");
    assert_eq!(field_error.issue_keys(), vec!["validation.invalid"]);

    // A passing inner value clears the slot.
    let holder = TicketFormFormValueHolder {
        ticket: Some(Ticket {
            id: "T-1".to_string(),
        }),
    };
    assert!(holder.validate_fields().is_empty());
}

#[test]
fn error_enum_payload_borrows_the_koruma_error() {
    // The Code variant's payload is the koruma wrapper, so koruma's own
    // typed surface (all()/inner()) is reachable through the enum.
    let errors = CodeFormFormValueHolder::default().validate_fields();
    if let Some(CodeFormFormFieldError::Code(wrapper)) = errors.error_for(&CodeFormFormPath::code())
    {
        assert!(wrapper.required_validation().is_some());
    } else {
        panic!("expected the Code variant with the koruma wrapper payload");
    }
}
