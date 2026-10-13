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

use std::sync::atomic::{AtomicUsize, Ordering};

use gpui_form_derive::GpuiForm;
use koruma::validator;

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

// ── Hit-side contracts (core-hit-tests) ──────────────────────────────────────
//
// A fresh cache must serve `error_for`/`validation()`/`is_valid` reads WITHOUT
// revalidation; `is_valid`'s non-caching miss path stays intentional. A plain
// form cannot observe a recompute (re-running koruma on the same value is
// invisible), so these tests drive a counting validator: a koruma
// `#[validator]` whose `Validate::validate` bumps a static counter on every
// run.

/// Counts every `Validate::validate` call koruma's generated `validate()`
/// makes. Always passes; the count is the only observable.
#[validator]
#[derive(Clone, Debug)]
struct CallCountValidation<T> {
    /// Presence-only slot; never captured — the validated value arrives as
    /// `validate`'s argument (mirrors the derive's `required_if` validator
    /// shape: `value` + `skip_capture` on an `Option<T>` slot). The slot
    /// exists to keep the validator shape koruma-shaped, not to be read;
    /// the static counter is the only observable.
    #[koruma(value, skip_capture)]
    #[allow(dead_code)] // presence-only slot; see doc comment
    actual: Option<T>,
}

impl<T> koruma::Validate<T> for CallCountValidation<T> {
    fn validate(&self, _value: &T) -> bool {
        // The previous count is irrelevant; only the bump is observable.
        let _ = VALIDATION_CALLS.fetch_add(1, Ordering::Relaxed);
        true
    }
}

/// Only the `CountedForm` test below touches this counter, so parallel test
/// threads cannot interfere with it.
static VALIDATION_CALLS: AtomicUsize = AtomicUsize::new(0);

#[derive(GpuiForm, koruma::Koruma)]
struct CountedForm {
    #[gpui_form(component(number_input))]
    #[koruma(CallCountValidation::<_>::builder())]
    score: u32,
}

#[test]
fn fresh_cache_makes_error_for_and_validation_reads_revalidate_zero_times() {
    // core-hit-tests: pins the DOCUMENTED cache-on-hit contract of
    // `error_for` ("serves the cached verdict when fresh"). If round 4
    // (core-error-cache) re-shapes the cache internals, this is the test
    // that must keep asserting zero revalidation on hits.
    let mut state = CountedFormFormState::new(CountedFormFormValueHolder { score: Some(7) });

    // Warm the cache: a full validation has run.
    state.validate();
    let warm = VALIDATION_CALLS.load(Ordering::Relaxed);
    assert!(warm >= 1, "validate() runs the validators");

    // Hit side: `error_for` and `validation()` serve the cache with ZERO
    // revalidation.
    assert!(state.error_for(&CountedFormFormPath::score()).is_none());
    assert!(state.validation().is_some());
    assert_eq!(
        VALIDATION_CALLS.load(Ordering::Relaxed),
        warm,
        "hit-side reads must not re-run any validator"
    );

    // Mutator -> read ordering: the edit disarms the cache; `is_valid`'s
    // miss path recomputes WITHOUT caching (pinned by
    // `default_holder_is_invalid_and_the_cache_starts_empty`, and forced by
    // the `&self` signature — caching needs `&mut`), so it costs exactly one
    // revalidation and leaves the cache empty.
    state.current_mut().score = Some(3);
    assert!(state.validation().is_none());
    assert!(state.is_valid());
    let recomputed = VALIDATION_CALLS.load(Ordering::Relaxed);
    assert!(recomputed > warm, "the stale-cache read revalidated");
    assert!(
        state.validation().is_none(),
        "is_valid's miss path does not arm the cache"
    );

    // The next `error_for` is therefore a miss: it revalidates exactly once,
    // arms the cache, and every read after THAT is a zero-revalidation hit.
    assert!(state.error_for(&CountedFormFormPath::score()).is_none());
    let rearmed = VALIDATION_CALLS.load(Ordering::Relaxed);
    assert_eq!(
        rearmed,
        recomputed + 1,
        "error_for's miss revalidates once to arm the cache"
    );

    // ...and the re-armed cache serves without revalidation.
    assert!(state.error_for(&CountedFormFormPath::score()).is_none());
    assert_eq!(
        VALIDATION_CALLS.load(Ordering::Relaxed),
        rearmed,
        "the re-armed hit path must not re-run any validator"
    );
}

#[test]
fn hit_side_error_for_leaves_the_cached_snapshot_intact() {
    // core-hit-tests: the read must not disturb the cached snapshot. This
    // pins the documented contract but cannot by itself distinguish a
    // re-store of an equal outcome (the outcome lives inline in the cache
    // slot, so its address never moves) — the counting test above is the
    // zero-revalidation oracle.
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
    // core-hit-tests: mutator -> read ordering keeps the hit path honest —
    // a cached-invalid verdict serves reads, a mutator disarms it, the next
    // read recomputes, and the re-armed cache keeps serving the fresh
    // verdict.
    let mut state = SeatsFormFormState::new(SeatsFormFormValueHolder::default());
    state.validate();
    assert!(!state.is_valid());

    // Cached-invalid serves the failure through the hit path.
    assert!(state.error_for(&SeatsFormFormPath::seats()).is_some());

    // The mutator disarms the cache; the next `error_for` recomputes None.
    state.current_mut().seats = Some(7);
    assert!(state.error_for(&SeatsFormFormPath::seats()).is_none());
    assert!(
        state.is_valid(),
        "is_valid stays accurate after the recompute"
    );

    // The re-armed cache keeps serving the fresh verdict on hits.
    assert!(state.error_for(&SeatsFormFormPath::seats()).is_none());
}
