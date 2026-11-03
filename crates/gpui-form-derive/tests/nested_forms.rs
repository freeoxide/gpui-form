use gpui_form::PathSegment;
use gpui_form::core::FormFieldErrors;
use gpui_form_derive::GpuiForm;
use koruma_collection::collection::NonEmptyValidation;

#[derive(Clone, Debug, Default, Eq, GpuiForm, PartialEq, koruma::Koruma)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Address {
    #[gpui_form(component(input))]
    #[koruma(NonEmptyValidation::<_>::builder())]
    pub city: String,

    #[gpui_form(component(input))]
    pub zip: Option<String>,
}

#[derive(GpuiForm, koruma::Koruma)]
struct Resident {
    #[gpui_form(component(input))]
    #[koruma(NonEmptyValidation::<_>::builder())]
    name: String,

    #[gpui_form(component(nested))]
    #[koruma(nested)]
    address: Address,
}

#[test]
fn nested_holder_field_is_the_nested_forms_holder() {
    let holder = ResidentFormValueHolder::default();
    let _: AddressFormValueHolder = holder.address;
}

#[test]
fn from_original_composes_through_the_nested_from_impls() {
    let original = Resident {
        name: "ada".to_string(),
        address: Address {
            city: "Paris".to_string(),
            zip: Some("75001".to_string()),
        },
    };
    let holder = ResidentFormValueHolder::from(original);
    assert_eq!(holder.name.as_deref(), Some("ada"));
    assert_eq!(holder.address.city.as_deref(), Some("Paris"));
    assert_eq!(holder.address.zip.as_deref(), Some("75001"));
}

#[test]
fn into_original_composes_back_through_the_nested_from_impls() {
    let holder = ResidentFormValueHolder {
        name: Some("grace".to_string()),
        address: AddressFormValueHolder {
            city: Some("Austin".to_string()),
            zip: None,
        },
    };
    let original: Resident = holder.into();
    assert_eq!(original.name, "grace");
    assert_eq!(original.address.city, "Austin");
    assert_eq!(original.address.zip, None);
}

#[test]
fn nested_koruma_validation_fires_through_the_outer_holder() {
    let holder = ResidentFormValueHolder {
        name: Some("ada".to_string()),
        address: AddressFormValueHolder::default(),
    };
    assert!(holder.validate().is_err());

    let valid = ResidentFormValueHolder {
        name: Some("ada".to_string()),
        address: AddressFormValueHolder {
            city: Some("Paris".to_string()),
            zip: None,
        },
    };
    assert!(valid.validate().is_ok());
}

#[test]
fn validate_fields_agrees_with_koruma_and_keys_the_nested_slot() {
    let holder = ResidentFormValueHolder {
        name: Some("ada".to_string()),
        address: AddressFormValueHolder::default(),
    };
    let errors = holder.validate_fields();
    assert_eq!(
        holder.validate_fields().is_empty(),
        holder.validate().is_ok()
    );
    assert!(errors.has_errors());

    let nested = errors.address().expect("nested slot must be populated");
    assert!(!::koruma::ValidationError::is_empty(nested));
}

#[test]
fn error_for_routes_nested_root_and_composed_paths_to_the_nested_slot() {
    let holder = ResidentFormValueHolder {
        name: Some("ada".to_string()),
        address: AddressFormValueHolder::default(),
    };
    let errors = holder.validate_fields();

    let root_error = errors
        .error_for(&ResidentFormPath::address_root())
        .expect("the nested root path routes to the nested slot");
    assert_eq!(root_error.path().to_string(), "address");
    assert_eq!(root_error.issue_keys(), vec!["validation.invalid"]);

    let composed = ResidentFormPath::address_root().join(&AddressFormPath::city());
    assert_eq!(composed.to_string(), "address.city");
    assert_eq!(
        composed.path().segments(),
        &[PathSegment::Name("address"), PathSegment::Name("city")]
    );
    let deep_error = errors
        .error_for(&composed)
        .expect("composed nested paths route to the nested slot");
    assert_eq!(deep_error.issue_keys(), vec!["validation.invalid"]);

    let valid = ResidentFormValueHolder {
        name: Some("ada".to_string()),
        address: AddressFormValueHolder {
            city: Some("Paris".to_string()),
            zip: None,
        },
    };
    let errors = valid.validate_fields();
    assert!(
        errors
            .error_for(&ResidentFormPath::address_root())
            .is_none()
    );
    assert!(errors.error_for(&composed).is_none());
}

#[test]
fn nested_errors_recurses_into_the_nested_forms_own_error_for() {
    let holder = ResidentFormValueHolder {
        name: Some("ada".to_string()),
        address: AddressFormValueHolder {
            city: Some(String::new()),
            zip: None,
        },
    };
    let errors = holder.validate_fields();

    let nested_map = errors
        .address_errors()
        .expect("nested map accessor rebuilds the nested form's map");
    let city_error = nested_map
        .error_for(&AddressFormPath::city())
        .expect("city fails NonEmpty");
    assert_eq!(city_error.issue_keys(), vec!["validation.non_empty"]);

    fn generic_paths<E: FormFieldErrors>(errors: &E) -> Vec<String> {
        errors
            .failing_paths()
            .iter()
            .map(|path| path.to_string())
            .collect()
    }
    let failing = generic_paths(&errors);
    assert!(failing.contains(&"address".to_string()), "{failing:?}");
}

#[test]
fn flat_field_error_access_still_works_alongside_nested() {
    let errors = ResidentFormValueHolder::default().validate_fields();
    let name_error = errors
        .error_for(&ResidentFormPath::name())
        .expect("flat field keeps exact routing");
    assert_eq!(name_error.issue_keys(), vec!["validation.required"]);
}

#[test]
fn nested_path_constructors_root_and_appender() {
    assert_eq!(ResidentFormPath::address_root().to_string(), "address");
    let appended = AddressFormPath::city().into_path().push_name("x");
    let _ = appended;
    assert_eq!(
        ResidentFormPath::address_root().join(&AddressFormPath::zip()),
        ResidentFormPath::new(&["address", "zip"])
    );
}

#[test]
fn field_state_surface_covers_nested_fields() {
    let mut state = ResidentFormState::new(ResidentFormValueHolder::default());
    assert_eq!(ResidentFormPath::FIELD_COUNT, 2);
    assert!(!state.is_field_dirty(&ResidentFormPath::address_root()));

    state.current_mut().address.city = Some("Paris".to_string());
    assert!(state.is_field_dirty(&ResidentFormPath::address_root()));
    assert!(!state.is_field_dirty(&ResidentFormPath::name()));

    state.touch(&ResidentFormPath::address_root());
    assert!(state.is_touched(&ResidentFormPath::address_root()));
    assert!(!state.is_touched(&ResidentFormPath::name()));

    let patch = state.patch();
    assert!(patch.address.is_some());
    let changed = patch.changed_fields();
    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0].to_string(), "address");
}

#[derive(GpuiForm, koruma::Koruma)]
struct FlatNestedOnlyForm {
    #[koruma(nested)]
    legacy: Address,
}

#[test]
fn flat_koruma_nested_field_stays_unvalidated_like_pre_m3() {
    let holder = FlatNestedOnlyFormFormValueHolder::default();
    assert!(
        holder.validate().is_ok(),
        "flat koruma(nested)-only fields must not gain new validate() failures"
    );
    assert_eq!(
        holder.validate_fields().is_empty(),
        holder.validate().is_ok()
    );
    assert!(
        holder
            .validate_fields()
            .error_for(&FlatNestedOnlyFormFormPath::legacy())
            .is_none()
    );

    let composed = ResidentFormValueHolder {
        name: Some("ada".to_string()),
        address: AddressFormValueHolder::default(),
    };
    assert!(composed.validate().is_err());
}
