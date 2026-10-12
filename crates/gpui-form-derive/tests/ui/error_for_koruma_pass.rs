use gpui_form::core::FormFieldErrors;
use gpui_form::{FormState, TouchedFields};
use gpui_form_derive::GpuiForm;

#[derive(GpuiForm, koruma::Koruma)]
struct Seats {
    #[gpui_form(component(number_input))]
    #[koruma(koruma_collection::numeric::RangeValidation::<_>::builder().min(1).max(12))]
    seats: u32,

    #[gpui_form(component(input))]
    #[gpui_form(skip)]
    internal: u32,
}

fn main() {
    // Path tables (feature 1).
    assert_eq!(SeatsFormPath::FIELD_COUNT, 1);
    assert_eq!(SeatsFormPath::FIELD_NAMES, &["seats"]);
    assert_eq!(SeatsFormPath::all_paths().count(), 1);

    // Facade + patch (features 1 + 2) over the same core primitives the
    // facade crate re-exports.
    let mut state = SeatsFormState::new(SeatsFormValueHolder::default());
    let _: &FormState<SeatsFormValueHolder> = state.state();
    let standalone = TouchedFields::new(SeatsFormPath::FIELD_COUNT);
    assert_eq!(standalone.field_count(), 1);
    state.touch(&SeatsFormPath::seats());
    assert!(state.is_touched(&SeatsFormPath::seats()));
    state.current_mut().seats = Some(5);
    assert!(state.is_field_dirty(&SeatsFormPath::seats()));
    assert_eq!(state.changed_fields(), vec![SeatsFormPath::seats()]);
    let patch = state.patch();
    assert!(patch.seats.is_some());
    patch.apply_to(&mut SeatsFormValueHolder::default());

    // Typed error access (feature 3).
    let errors = SeatsFormValueHolder { seats: Some(13) }.validate_fields();
    assert!(errors.has_errors());
    let field_error = errors
        .error_for(&SeatsFormPath::seats())
        .expect("range failure surfaces through the typed path");
    assert_eq!(field_error.issue_keys(), vec!["validation.range"]);
    assert_eq!(field_error.path().to_string(), "seats");

    // Generic contract over the shared FieldPath primitive.
    fn failing_names<E: FormFieldErrors>(errors: &E) -> Vec<String> {
        errors
            .failing_paths()
            .iter()
            .map(|path| path.to_string())
            .collect()
    }
    assert_eq!(failing_names(&errors), vec!["seats"]);
}
