//! Validation-state cache for a live `is_valid` (M2, [D6]): koruma-gated
//! `<Name>FormState` facades gain a `validation` slot of
//! `gpui_form::core::ValidationCache<Holder, <Name>FormFieldErrors>` plus
//! `validate`/`validation`/`is_valid`/`invalidate_validation`/`error_for`.
//!
//! Freshness is snapshot-compare (`current == validated snapshot`), so the
//! cache stays sound even under the `state_mut()` escape hatch; the
//! reset-style mutators invalidate explicitly so a reverted-to-same-value
//! form does not keep displaying the old verdict, while `sync_baseline`
//! deliberately keeps the cache (validity depends only on `current`).
//!
//! Mirrors the struct shapes in `koruma_validation_behavior.rs`, covering
//! both koruma enablement routes (field-attr-only and struct-level).

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

/// Struct-level koruma with ZERO participating fields (originally-optional
/// fields, no validators): the error map has no slots and the borrowing
/// enum has no variants, but the facade still emits the cache and its
/// methods — the subtle zero-variant edge of the generated surface.
#[derive(GpuiForm, koruma::Koruma)]
#[gpui_form(koruma)]
struct OptionalOnlyForm {
    note: Option<String>,
    kind: Option<u8>,
}

#[test]
fn default_holder_is_invalid_and_the_cache_starts_empty() {
    let state = SeatsFormFormState::new(SeatsFormFormValueHolder::default());
    assert!(state.validation().is_none(), "nothing validated yet");
    assert!(!state.is_valid(), "absent default fails the range rule");

    // `is_valid` on a stale cache computes fresh WITHOUT caching.
    assert!(
        state.validation().is_none(),
        "is_valid does not populate the cache"
    );
}

#[test]
fn validate_stores_and_reads_serve_the_cache() {
    let mut state = SeatsFormFormState::new(SeatsFormFormValueHolder::default());

    let errors = state.validate().clone();
    assert!(!errors.is_empty());
    assert!(
        state.validation().is_some(),
        "validate() populated the cache"
    );
    assert!(!state.is_valid(), "cached verdict serves is_valid");

    // Fix the field through the facade mutator: the cache invalidates and
    // is_valid recomputes accurately.
    state.current_mut().seats = Some(7);
    assert!(state.validation().is_none(), "edits invalidate the cache");
    assert!(state.is_valid(), "in-range seats are valid (fresh compute)");

    // Validate again: the fresh verdict caches.
    state.validate();
    assert!(state.validation().is_some());
    assert!(state.is_valid(), "cached verdict is the new one");
}

#[test]
fn state_mut_escape_is_caught_by_the_snapshot_comparison() {
    let mut state = SeatsFormFormState::new(SeatsFormFormValueHolder::default());
    state.validate();
    assert!(state.validation().is_some());

    // Mutate through the raw escape hatch: no explicit invalidation runs
    // (that is the documented `state_mut` contract), but the snapshot
    // comparison still reports stale.
    state.state_mut().current_mut().seats = Some(13);
    assert!(
        state.validation().is_none(),
        "snapshot-compare beats the escape hatch"
    );
    assert!(!state.is_valid(), "is_valid stays accurate");

    // Editing BACK to the validated value makes the cache fresh again —
    // correct, because validation is a pure function of the value.
    state.state_mut().current_mut().seats = None;
    assert!(state.validation().is_some(), "same value == same verdict");
    assert!(!state.is_valid());
}

#[test]
fn current_mut_invalidates_even_when_the_value_swings_back() {
    // The explicit invalidation on the facade's own edit path is stricter
    // than pure equality: an edit that lands on the validated value still
    // drops the cache, so a validator with interior state (today: none, by
    // design) could never be served stale.
    let mut state = SeatsFormFormState::new(SeatsFormFormValueHolder::default());
    state.validate();
    assert!(state.validation().is_some());

    state.current_mut().seats = Some(13);
    state.current_mut().seats = None;
    assert!(
        state.validation().is_none(),
        "current_mut invalidates on every edit"
    );
    assert!(!state.is_valid(), "the recomputed verdict is still correct");
}

#[test]
fn reset_to_baseline_invalidates_even_when_values_match() {
    let mut state = SeatsFormFormState::new(SeatsFormFormValueHolder::default());
    state.validate();
    assert!(state.validation().is_some());

    // Edit away, then reset: `current` becomes the baseline — the default
    // holder, which EQUALS the validated snapshot. The explicit reset
    // invalidation must win over the equality (a full reset is a fresh
    // canvas; error display is suppressed).
    state.current_mut().seats = Some(5);
    state.reset_to_baseline();
    assert_eq!(state.current().seats, None);
    assert!(
        state.validation().is_none(),
        "reset invalidates even a matching snapshot"
    );
    assert!(!state.is_valid(), "and the recomputed verdict is right");

    // The keeping-touched variant invalidates the same way.
    state.validate();
    state.current_mut().seats = Some(5);
    state.reset_to_baseline_keeping_touched();
    assert!(state.validation().is_none());
}

#[test]
fn replace_current_invalidates() {
    let mut state = SeatsFormFormState::new(SeatsFormFormValueHolder::default());
    state.validate();

    // Replace with a value EQUAL to the current one: still a new value
    // generation, so the cache drops.
    let equal = state.current().clone();
    state.replace_current(equal);
    assert!(state.validation().is_none());

    state.validate();
    state.replace_current(SeatsFormFormValueHolder { seats: Some(5) });
    assert!(state.validation().is_none());
    assert!(state.is_valid());
}

#[test]
fn sync_baseline_keeps_the_cache_fresh() {
    let mut state = SeatsFormFormState::new(SeatsFormFormValueHolder::default());
    state.validate();
    assert!(state.validation().is_some());

    // A successful save re-syncs the baseline; `current` did not change, so
    // the validation cache stays fresh by design.
    state.sync_baseline();
    assert!(
        state.validation().is_some(),
        "sync_baseline does not invalidate"
    );
    assert!(!state.is_valid());

    // Also true when the save happens after edits (cache already refreshed).
    state.current_mut().seats = Some(9);
    state.validate();
    state.sync_baseline();
    assert!(state.validation().is_some());
    assert!(state.is_valid());
}

#[test]
fn invalidate_validation_is_the_manual_escape() {
    let mut state = SeatsFormFormState::new(SeatsFormFormValueHolder::default());
    state.validate();
    assert!(state.validation().is_some());

    state.invalidate_validation();
    assert!(state.validation().is_none());

    // It also beats a matching snapshot (the state_mut + edit-back shape).
    state.validate();
    state.state_mut().current_mut().seats = Some(1);
    state.state_mut().current_mut().seats = None;
    assert!(state.validation().is_some(), "equality made it fresh again");
    state.invalidate_validation();
    assert!(state.validation().is_none());
    assert!(!state.is_valid(), "is_valid still recomputes correctly");
}

#[test]
fn error_for_validates_and_caches_on_miss() {
    let mut state = SeatsFormFormState::new(SeatsFormFormValueHolder::default());
    assert!(state.validation().is_none());

    let field_error = state
        .error_for(&SeatsFormFormPath::seats())
        .expect("absent default fails through the cached lookup");
    assert_eq!(field_error.issue_keys(), vec!["validation.required"]);
    assert!(
        state.validation().is_some(),
        "error_for validated and cached on the miss"
    );

    // Valid field through the same cached surface.
    state.current_mut().seats = Some(3);
    assert!(state.error_for(&SeatsFormFormPath::seats()).is_none());
    // Unknown paths stay None and still populate nothing extra.
    assert!(
        state
            .error_for(&SeatsFormFormPath::new(&["nope"]))
            .is_none()
    );
}

#[test]
fn struct_level_koruma_form_gains_the_cache() {
    let mut state = NameFormFormState::new(NameFormFormValueHolder::default());
    assert!(!state.is_valid());
    state.validate();
    assert!(!state.is_valid());

    state.current_mut().name = Some("ada".to_string());
    assert!(state.is_valid());
    state.validate();
    assert!(state.is_valid());
    assert!(state.validation().is_some());
}

#[test]
fn zero_participating_fields_still_get_the_cached_surface() {
    // No field carries koruma attrs, so the map has zero slots and the
    // borrowing enum zero variants — the facade methods must still compile
    // and behave (everything validates trivially).
    let mut state = OptionalOnlyFormFormState::new(OptionalOnlyFormFormValueHolder::default());
    assert!(state.is_valid());
    assert!(state.validation().is_none());

    state.validate();
    assert!(state.validation().is_some());
    assert!(state.is_valid());
    assert!(state.error_for(&OptionalOnlyFormFormPath::note()).is_none());
    assert!(
        state
            .error_for(&OptionalOnlyFormFormPath::new(&["nope"]))
            .is_none()
    );

    // Edits invalidate; the fresh verdict stays valid.
    state.current_mut().note = Some("hi".to_string());
    assert!(state.validation().is_none());
    assert!(state.is_valid());
}
