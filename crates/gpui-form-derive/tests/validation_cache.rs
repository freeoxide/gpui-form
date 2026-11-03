use std::sync::atomic::{AtomicUsize, Ordering};

use gpui_form_derive::GpuiForm;
use koruma::validator;

#[derive(GpuiForm, koruma::Koruma)]
struct SeatsForm {
    #[gpui_form(component(number_input))]
    #[koruma(koruma_collection::numeric::RangeValidation::<_>::builder().min(1).max(12))]
    seats: u32,
}

#[derive(GpuiForm, koruma::Koruma)]
#[gpui_form(koruma)]
struct NameForm {
    #[gpui_form(component(input))]
    name: String,
}

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

    state.current_mut().seats = Some(7);
    assert!(state.validation().is_none(), "edits invalidate the cache");
    assert!(state.is_valid(), "in-range seats are valid (fresh compute)");

    state.validate();
    assert!(state.validation().is_some());
    assert!(state.is_valid(), "cached verdict is the new one");
}

#[test]
fn state_mut_escape_is_caught_by_the_snapshot_comparison() {
    let mut state = SeatsFormFormState::new(SeatsFormFormValueHolder::default());
    state.validate();
    assert!(state.validation().is_some());

    state.state_mut().current_mut().seats = Some(13);
    assert!(
        state.validation().is_none(),
        "snapshot-compare beats the escape hatch"
    );
    assert!(!state.is_valid(), "is_valid stays accurate");

    state.state_mut().current_mut().seats = None;
    assert!(state.validation().is_some(), "same value == same verdict");
    assert!(!state.is_valid());
}

#[test]
fn current_mut_invalidates_even_when_the_value_swings_back() {
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

    state.current_mut().seats = Some(5);
    state.reset_to_baseline();
    state.reset_to_baseline();
    assert_eq!(state.current().seats, None);
    assert!(
        state.validation().is_none(),
        "reset invalidates even a matching snapshot"
    );
    assert!(!state.is_valid(), "and the recomputed verdict is right");

    state.validate();
    state.current_mut().seats = Some(5);
    state.reset_to_baseline_keeping_touched();
    assert!(state.validation().is_none());
}

#[test]
fn replace_current_invalidates() {
    let mut state = SeatsFormFormState::new(SeatsFormFormValueHolder::default());
    state.validate();

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

    state.sync_baseline();
    assert!(
        state.validation().is_some(),
        "sync_baseline does not invalidate"
    );
    assert!(!state.is_valid());

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

    state.current_mut().seats = Some(3);
    assert!(state.error_for(&SeatsFormFormPath::seats()).is_none());
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

    state.current_mut().note = Some("hi".to_string());
    assert!(state.validation().is_none());
    assert!(state.is_valid());
}

#[validator]
#[derive(Clone, Debug)]
struct CallCountValidation<T> {
    #[koruma(value, skip_capture)]
    #[allow(dead_code)]
    actual: Option<T>,
}

impl<T> koruma::Validate<T> for CallCountValidation<T> {
    fn validate(&self, _value: &T) -> bool {
        let _ = VALIDATION_CALLS.fetch_add(1, Ordering::Relaxed);
        true
    }
}

static VALIDATION_CALLS: AtomicUsize = AtomicUsize::new(0);

#[derive(GpuiForm, koruma::Koruma)]
struct CountedForm {
    #[gpui_form(component(number_input))]
    #[koruma(CallCountValidation::<_>::builder())]
    score: u32,
}

#[test]
fn fresh_cache_makes_error_for_and_validation_reads_revalidate_zero_times() {
    let mut state = CountedFormFormState::new(CountedFormFormValueHolder { score: Some(7) });

    state.validate();
    let warm = VALIDATION_CALLS.load(Ordering::Relaxed);
    assert!(warm >= 1, "validate() runs the validators");

    assert!(state.error_for(&CountedFormFormPath::score()).is_none());
    assert!(state.validation().is_some());
    assert_eq!(
        VALIDATION_CALLS.load(Ordering::Relaxed),
        warm,
        "hit-side reads must not re-run any validator"
    );

    state.current_mut().score = Some(3);
    assert!(state.validation().is_none());
    assert!(state.is_valid());
    let recomputed = VALIDATION_CALLS.load(Ordering::Relaxed);
    assert!(recomputed > warm, "the stale-cache read revalidated");
    assert!(
        state.validation().is_none(),
        "is_valid's miss path does not arm the cache"
    );

    assert!(state.error_for(&CountedFormFormPath::score()).is_none());
    let rearmed = VALIDATION_CALLS.load(Ordering::Relaxed);
    assert_eq!(
        rearmed,
        recomputed + 1,
        "error_for's miss revalidates once to arm the cache"
    );

    assert!(state.error_for(&CountedFormFormPath::score()).is_none());
    assert_eq!(
        VALIDATION_CALLS.load(Ordering::Relaxed),
        rearmed,
        "the re-armed hit path must not re-run any validator"
    );
}

#[test]
fn hit_side_error_for_leaves_the_cached_snapshot_intact() {
    let mut state = SeatsFormFormState::new(SeatsFormFormValueHolder::default());
    state.validate();
    let before = state.validation().map(|errors| format!("{errors:?}"));

    let field_error = state
        .error_for(&SeatsFormFormPath::seats())
        .expect("the validated default fails through the hit path");
    assert_eq!(field_error.issue_keys(), vec!["validation.required"]);

    assert!(state.validation().is_some(), "the cache stays populated");
    assert_eq!(
        before,
        state.validation().map(|errors| format!("{errors:?}")),
        "the cached error snapshot is Debug-equal after the hit read"
    );
}

#[test]
fn error_for_hit_reads_stay_accurate_across_mutator_ordering() {
    let mut state = SeatsFormFormState::new(SeatsFormFormValueHolder::default());
    state.validate();
    assert!(!state.is_valid());

    assert!(state.error_for(&SeatsFormFormPath::seats()).is_some());

    state.current_mut().seats = Some(7);
    assert!(state.error_for(&SeatsFormFormPath::seats()).is_none());
    assert!(
        state.is_valid(),
        "is_valid stays accurate after the recompute"
    );

    assert!(state.error_for(&SeatsFormFormPath::seats()).is_none());
}
