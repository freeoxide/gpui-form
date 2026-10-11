//! Field dependency metadata and conditional requiredness (M4, backlog
//! #12/#13): `depends_on` records sibling metadata, and `required_if`
//! desugars to a generated koruma validator closure so the violation
//! surfaces through the M1 typed error surface (`validate_fields()`,
//! `error_for`, `issue_keys`) — never as display-only metadata.
//!
//! Covers all three controlling-sibling shapes the derive resolves
//! (`RequiredIfCondition`): plain `bool` (switch/checkbox),
//! `Option<bool>` (an explicit off must NOT require), and a generic
//! `Option<T>` (required when set). Also proves that a `required_if` field
//! never gains the synthetic unconditional `RequiredValidation` (the
//! conditional rule supersedes it).

use gpui_form::core::FormFieldErrors as _;
use gpui_form_derive::GpuiForm;

/// Plain-`bool` controller (the switch/checkbox shape): required when
/// `true`. No other koruma opt-in — `required_if` alone forces the koruma
/// path (otherwise the rule would be display-only).
#[derive(GpuiForm)]
struct Subscribe {
    #[gpui_form(component(switch))]
    subscribe: bool,

    #[gpui_form(component(input), required_if = subscribe)]
    email: Option<String>,
}

/// `Option<bool>` controller (an originally-optional boolean field):
/// required when `Some(true)`; an explicit `Some(false)` must NOT require.
/// Uses the quoted spelling from backlog #13.
#[derive(GpuiForm)]
struct OptIn {
    #[gpui_form(component(checkbox))]
    marketing: Option<bool>,

    #[gpui_form(component(input), required_if = "marketing")]
    email: Option<String>,
}

/// Generic `Option<T>` controller (presence semantics): required when the
/// controller is set. Also exercises `depends_on` riding on the same field
/// as `required_if` — realistic for cascading selects (reset the dependent
/// field AND require it once the controller is set).
#[derive(GpuiForm)]
struct Region {
    #[gpui_form(component(input))]
    country: Option<u8>,

    #[gpui_form(component(input), depends_on = country, required_if = country)]
    state: Option<u8>,
}

#[test]
fn required_if_fails_only_when_controller_requires_and_value_absent() {
    let holder = |subscribe, email: Option<&str>| SubscribeFormValueHolder {
        subscribe,
        email: email.map(str::to_string),
    };

    // Controller off: absent email passes (the field is optional).
    assert!(holder(false, None).validate().is_ok());
    // Controller on + present: passes.
    assert!(holder(true, Some("a@b.c")).validate().is_ok());
    // Controller on + absent: fails.
    assert!(holder(true, None).validate().is_err());
    // Controller off + present: still passes (no unconditional required).
    assert!(holder(false, Some("a@b.c")).validate().is_ok());
}

#[test]
fn required_if_surfaces_as_a_typed_path_addressable_error() {
    let failing = SubscribeFormValueHolder {
        subscribe: true,
        email: None,
    };
    let errors = failing.validate_fields();
    assert!(!errors.is_empty(), "controller on + absent email must fail");
    assert_eq!(
        errors.failing_paths(),
        vec![SubscribeFormPath::email().into_path()],
        "the conditional-required violation is path-addressable"
    );

    let error = errors
        .error_for(&SubscribeFormPath::email())
        .expect("email must carry the conditional-required error");
    assert!(
        errors.error_for(&SubscribeFormPath::subscribe()).is_none(),
        "the controller itself passes"
    );
    assert_eq!(
        error.issue_keys(),
        vec!["validation.required"],
        "the desugared rule reuses the localized required key"
    );
    assert_eq!(error.path(), SubscribeFormPath::email());
}

#[test]
fn optional_bool_controller_requires_only_on_some_true() {
    let holder = |marketing, email: Option<&str>| OptInFormValueHolder {
        marketing,
        email: email.map(str::to_string),
    };

    assert!(holder(None, None).validate().is_ok());
    assert!(
        holder(Some(false), None).validate().is_ok(),
        "an explicit Some(false) must NOT require the value"
    );
    assert!(holder(Some(true), Some("a@b.c")).validate().is_ok());
    assert!(
        holder(Some(true), None).validate().is_err(),
        "Some(true) + absent must fail"
    );
}

#[test]
fn generic_option_controller_requires_when_set() {
    let holder = |country, state: Option<u8>| RegionFormValueHolder { country, state };

    assert!(holder(None, None).validate().is_ok());
    assert!(holder(Some(1), Some(2)).validate().is_ok());
    assert!(
        holder(Some(1), None).validate().is_err(),
        "a set controller requires the dependent value"
    );
    assert!(
        holder(None, Some(9)).validate().is_ok(),
        "an unset controller leaves the dependent field optional"
    );
}

#[test]
fn required_if_supersedes_the_synthetic_unconditional_required() {
    // Controller off + absent passes: `required_if` must suppress the
    // unconditional `RequiredValidation` a wrapped field would otherwise
    // receive (requiring the field always would defeat the condition).
    let holder = SubscribeFormValueHolder {
        subscribe: false,
        email: None,
    };
    assert!(holder.validate().is_ok());
}

#[test]
fn required_if_composes_with_existing_validators() {
    // `required_if` rides in the same koruma attr list as mirrored field
    // validators: each rule reports its own key.
    #[derive(GpuiForm)]
    struct Composed {
        #[gpui_form(component(switch))]
        subscribe: bool,

        #[gpui_form(component(input), required_if = subscribe)]
        #[koruma(koruma_collection::string::PrefixValidation::<_>::builder().prefix("a"))]
        email: Option<String>,
    }

    // Controller on + a present-but-invalid value: required_if passes (the
    // value exists), the prefix rule fails.
    let invalid = ComposedFormValueHolder {
        subscribe: true,
        email: Some("not-a-prefix".to_string()),
    };
    let errors = invalid.validate_fields();
    let error = errors
        .error_for(&ComposedFormPath::email())
        .expect("composed validators must still report");
    let mut keys = error.issue_keys();
    keys.sort_unstable();
    assert_eq!(keys, vec!["validation.prefix"]);

    // Controller on + absent: required_if fails (the prefix rule sees no
    // value and passes).
    let absent = ComposedFormValueHolder {
        subscribe: true,
        email: None,
    };
    let errors = absent.validate_fields();
    let error = errors
        .error_for(&ComposedFormPath::email())
        .expect("the conditional rule must report");
    assert_eq!(error.issue_keys(), vec!["validation.required"]);
}
