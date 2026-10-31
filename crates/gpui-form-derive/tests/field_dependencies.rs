use gpui_form::core::FormFieldErrors as _;
use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct Subscribe {
    #[gpui_form(component(switch))]
    subscribe: bool,

    #[gpui_form(component(input), required_if = subscribe)]
    email: Option<String>,
}

#[derive(GpuiForm)]
struct OptIn {
    #[gpui_form(component(checkbox))]
    marketing: Option<bool>,

    #[gpui_form(component(input), required_if = "marketing")]
    email: Option<String>,
}

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

    assert!(holder(false, None).validate().is_ok());
    assert!(holder(true, Some("a@b.c")).validate().is_ok());
    assert!(holder(true, None).validate().is_err());
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
    let holder = SubscribeFormValueHolder {
        subscribe: false,
        email: None,
    };
    assert!(holder.validate().is_ok());
}

#[test]
fn required_if_composes_with_existing_validators() {
    #[derive(GpuiForm)]
    struct Composed {
        #[gpui_form(component(switch))]
        subscribe: bool,

        #[gpui_form(component(input), required_if = subscribe)]
        #[koruma(koruma_collection::string::PrefixValidation::<_>::builder().prefix("a"))]
        email: Option<String>,
    }

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
