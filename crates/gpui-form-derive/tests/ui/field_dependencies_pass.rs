// M4 (backlog #12/#13): dependency metadata + conditional requiredness
// compile end to end — `depends_on` records metadata, `required_if`
// desugars to a generated koruma validator so the violation is a typed,
// path-addressable error. No `koruma(fluent)` here, so no trybuild locale
// seeding is needed.
use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct Signup {
    #[gpui_form(component(switch))]
    subscribe: bool,

    #[gpui_form(component(input), required_if = subscribe)]
    email: Option<String>,
}

#[derive(GpuiForm)]
struct Region {
    #[gpui_form(component(input))]
    country: Option<u8>,

    #[gpui_form(component(input), depends_on = country)]
    state: Option<u8>,
}

fn main() {
    let failing = SignupFormValueHolder {
        subscribe: true,
        email: None,
    };
    let errors = failing.validate_fields();
    assert!(!errors.is_empty());
    let error = errors
        .error_for(&SignupFormPath::email())
        .expect("controller on + absent email carries the conditional error");
    assert_eq!(error.issue_keys(), vec!["validation.required"]);

    let ok = SignupFormValueHolder {
        subscribe: false,
        email: None,
    };
    assert!(ok.validate().is_ok());

    // `depends_on` is metadata-only at the derive layer; the region form
    // still compiles and validates nothing special.
    let region = RegionFormValueHolder {
        country: None,
        state: None,
    };
    assert_eq!(region.country, None);
}
