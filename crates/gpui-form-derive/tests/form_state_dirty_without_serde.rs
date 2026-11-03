use gpui_form::FormState;
use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct DirtyCheckForm {
    #[gpui_form(component(input))]
    label: String,
}

#[test]
fn is_dirty_compiles_and_behaves_without_serde_feature() {
    let holder = DirtyCheckFormFormValueHolder::from(DirtyCheckForm {
        label: "baseline".to_string(),
    });

    let mut state = FormState::new(holder);
    assert!(!state.is_dirty());

    state.current_mut().label = Some("edited".to_string());
    assert!(state.is_dirty());

    state.reset_to_baseline();
    assert!(!state.is_dirty());
}
