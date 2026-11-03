use gpui_form::FormFieldErrors;
use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct Settings {
    #[gpui_form(component(input))]
    username: String,

    #[gpui_form(component(number_input))]
    age: Option<u32>,
}

#[test]
fn facade_patch_and_errors_compile_and_behave_without_serde() {
    let mut state = SettingsFormState::new(SettingsFormValueHolder::default());
    assert!(!state.is_dirty());

    state.current_mut().username = Some("ada".to_string());
    assert!(state.is_dirty());
    assert!(state.is_field_dirty(&SettingsFormPath::username()));
    assert!(!state.is_field_dirty(&SettingsFormPath::age()));

    let patch = state.patch();
    assert!(!patch.is_empty());
    let changed: Vec<String> = state
        .changed_fields()
        .iter()
        .map(|path| path.to_string())
        .collect();
    assert_eq!(changed, vec!["username"]);

    assert_eq!(SettingsFormPath::FIELD_COUNT, 2);
    assert_eq!(SettingsFormPath::all_paths().count(), 2);
}

#[test]
fn touched_tracking_has_no_serde_dependency() {
    let mut state = SettingsFormState::new(SettingsFormValueHolder::default());
    state.touch(&SettingsFormPath::age());
    assert!(state.is_touched(&SettingsFormPath::age()));
    assert_eq!(state.touched_fields(), vec![SettingsFormPath::age()]);
    state.reset_to_baseline();
    assert!(!state.is_any_touched());
}

#[test]
fn error_surface_trait_is_reachable_without_serde() {
    struct NoErrors;
    impl FormFieldErrors for NoErrors {
        fn is_empty(&self) -> bool {
            true
        }
        fn failing_paths(&self) -> Vec<::gpui_form::FieldPath> {
            Vec::new()
        }
    }
    assert!(NoErrors.is_empty());
}
