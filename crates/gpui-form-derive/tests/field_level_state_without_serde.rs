//! Regression test for the feature-unification matrix (R13), mirroring
//! `form_state_dirty_without_serde.rs`.
//!
//! M1's field-level state emission is unconditional (facade + patch) or
//! koruma-gated (error surface) — never `serde`-gated. The patch's
//! `Serialize`/`Deserialize` derives and per-field serde attrs only appear
//! when the derive crate is built with its `serde` feature. This file is
//! deliberately NOT gated by `#![cfg(feature = "serde")]` and contains no
//! serde usage whatsoever: it must compile and pass under DEFAULT features,
//! proving the new generated surface does not depend on the `serde` feature.
//!
//! (When the workspace builds `--all-features`, the derive crate gains
//! `serde`, the patch gains serde derives — and this file still compiles,
//! proving both sides of the matrix.)

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
    // Facade: touched + dirty over the same holder the old test used.
    let mut state = SettingsFormState::new(SettingsFormValueHolder::default());
    assert!(!state.is_dirty());

    state.current_mut().username = Some("ada".to_string());
    assert!(state.is_dirty());
    assert!(state.is_field_dirty(&SettingsFormPath::username()));
    assert!(!state.is_field_dirty(&SettingsFormPath::age()));

    // Patch: typed diff between baseline and current, no serde involved.
    let patch = state.patch();
    assert!(!patch.is_empty());
    let changed: Vec<String> = state
        .changed_fields()
        .iter()
        .map(|path| path.to_string())
        .collect();
    assert_eq!(changed, vec!["username"]);

    // Path tables: dense index mapping is a pure, featureless surface.
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
    // The generic contract is re-exported unconditionally by the facade;
    // a non-koruma form never emits the concrete map, but the trait itself
    // must be importable and implementable on default features.
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
