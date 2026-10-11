//! Submit lifecycle (M2, backlog #7): the opt-in `#[gpui_form(submit)]`
//! attribute emits a `<Name>FormSubmission` alias over the headless core
//! `gpui_form::core::submit::SubmitTracker`. The error parameter is the
//! form's koruma validation error type when the holder derives `Koruma`
//! (reached through `ValidateExt::Error`), and the core `BoxError` alias
//! otherwise. The holder stays plain data — submitting never touches it.
//!
//! Covers BOTH koruma enablement routes for the alias, mirroring
//! `error_for.rs`: field-attr-only koruma (the SeatsForm-style shape a
//! `koruma_options.is_some()` gate would silently drop, hazard 6) and
//! struct-level `#[gpui_form(koruma)]`.

use gpui_form::core::submit::SubmitTracker;
use gpui_form::{BoxError, SubmitAction};
use gpui_form_derive::GpuiForm;
use std::future::Future;
use std::task::Poll;

/// Field-attr-only koruma (no struct-level `#[gpui_form(koruma)]`) + submit:
/// the alias must still resolve its error type through `ValidateExt`.
#[derive(GpuiForm, koruma::Koruma)]
#[gpui_form(submit)]
struct SeatsForm {
    #[gpui_form(component(number_input))]
    #[koruma(koruma_collection::numeric::RangeValidation::<_>::builder().min(1).max(12))]
    seats: u32,
}

/// Struct-level koruma + submit (required-only validation).
#[derive(GpuiForm, koruma::Koruma)]
#[gpui_form(koruma, submit)]
struct NameForm {
    #[gpui_form(component(input))]
    name: String,
}

/// No koruma at all + submit: the alias falls back to `BoxError`.
#[derive(GpuiForm)]
#[gpui_form(submit)]
struct PlainForm {
    #[gpui_form(component(input))]
    name: String,
}

/// Empty form + submit: the alias still emits (nothing to validate).
#[derive(GpuiForm)]
#[gpui_form(empty, submit)]
struct BareForm {}

/// Std-only executor for the `SubmitAction` driving test; the action's
/// future is immediately ready, so spin-polling a noop waker suffices.
fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = std::pin::pin!(future);
    let waker = std::task::Waker::noop();
    let mut cx = std::task::Context::from_waker(waker);
    loop {
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(output) => return output,
            Poll::Pending => continue,
        }
    }
}

struct SaveSeats;

impl SubmitAction<SeatsFormFormValueHolder> for SaveSeats {
    type Output = u32;
    type Error = <SeatsFormFormValueHolder as koruma::ValidateExt>::Error;

    async fn submit(&mut self, holder: &SeatsFormFormValueHolder) -> Result<u32, Self::Error> {
        match holder.validate() {
            Ok(()) => Ok(holder.seats.unwrap_or_default()),
            Err(errors) => Err(errors),
        }
    }
}

#[test]
fn alias_exists_and_keys_on_needs_koruma_derive() {
    // Field-attr-only koruma route: the error parameter resolves to the
    // holder's koruma error type, so a failed validate-then-submit carries
    // the typed field errors.
    let mut tracker: SeatsFormFormSubmission = SubmitTracker::new();
    assert!(!tracker.is_submitting());
    assert!(tracker.can_submit());
    assert_eq!(tracker.submit_count(), 0);

    let failure: <SeatsFormFormValueHolder as koruma::ValidateExt>::Error =
        SeatsFormFormValueHolder::default()
            .validate()
            .err()
            .expect("absent default must fail");
    tracker.begin();
    tracker.finish(Err(failure));
    assert!(tracker.last_error().is_some());

    // Struct-level koruma route gets the same alias shape.
    let _: NameFormFormSubmission = SubmitTracker::new();
}

#[test]
fn transitions_begin_err_then_ok_carry_state() {
    let mut tracker: SeatsFormFormSubmission = SubmitTracker::new();

    assert!(tracker.begin());
    assert!(tracker.is_submitting());
    assert!(!tracker.can_submit());
    assert!(!tracker.begin(), "re-entry guard refuses a second begin");

    tracker.finish(Err(
        <SeatsFormFormValueHolder as koruma::ValidateExt>::Error::default(),
    ));
    assert!(!tracker.is_submitting());
    assert!(tracker.is_submitted());
    assert!(!tracker.is_submit_successful());
    assert_eq!(tracker.submit_count(), 1);

    assert!(tracker.begin());
    tracker.finish(Ok(()));
    assert!(tracker.is_submitted());
    assert!(tracker.is_submit_successful());
    assert!(tracker.last_error().is_none(), "success clears the error");
    assert_eq!(tracker.submit_count(), 2);
}

#[test]
fn stale_generation_completions_are_ignored() {
    let mut tracker: SeatsFormFormSubmission = SubmitTracker::new();

    tracker.begin();
    tracker.finish(Err(
        <SeatsFormFormValueHolder as koruma::ValidateExt>::Error::default(),
    ));
    assert!(tracker.is_submitted());

    // A second attempt goes in flight, then the form is reset mid-flight:
    // the epoch moves and all state clears.
    tracker.begin();
    assert!(tracker.is_submitting());
    tracker.reset();
    assert!(!tracker.is_submitting());
    assert_eq!(tracker.submit_count(), 0);
    assert!(!tracker.is_submitted());
    assert!(tracker.last_error().is_none());

    // The in-flight completion arrives late: ignored entirely (the D7/R12
    // generation guard) — it must not flip submitting, verdicts, or errors.
    tracker.finish(Ok(()));
    assert!(!tracker.is_submitting());
    assert!(!tracker.is_submitted());
    assert!(!tracker.is_submit_successful());
    tracker.finish(Err(
        <SeatsFormFormValueHolder as koruma::ValidateExt>::Error::default(),
    ));
    assert!(tracker.last_error().is_none());

    // The tracker is still usable for a fresh lifecycle afterwards.
    assert!(tracker.begin());
    tracker.finish(Ok(()));
    assert!(tracker.is_submit_successful());
}

#[test]
fn manual_error_injection_and_abandon_do_not_fake_verdicts() {
    let mut tracker: SeatsFormFormSubmission = SubmitTracker::new();
    tracker.set_error(<SeatsFormFormValueHolder as koruma::ValidateExt>::Error::default());
    assert!(tracker.last_error().is_some());
    assert!(!tracker.is_submitted(), "injection is not an attempt");

    tracker.clear_error();
    assert!(tracker.last_error().is_none());

    tracker.begin();
    tracker.abandon();
    assert!(!tracker.is_submitting());
    assert!(
        !tracker.is_submitted(),
        "abandoned attempts have no outcome"
    );
    tracker.finish(Err(
        <SeatsFormFormValueHolder as koruma::ValidateExt>::Error::default(),
    ));
    assert!(
        !tracker.is_submitted(),
        "late completion after abandon is stale"
    );
}

#[test]
fn non_koruma_and_empty_forms_get_the_box_error_alias() {
    let mut plain: PlainFormFormSubmission = SubmitTracker::<BoxError>::new();
    assert!(plain.can_submit());
    plain.set_error("server said no".into());
    assert!(plain.last_error().is_some());
    assert!(!plain.is_submit_successful());

    let bare: BareFormFormSubmission = SubmitTracker::new();
    assert!(!bare.is_submitted());
    assert!(bare.can_submit());
}

#[test]
fn driving_a_submit_action_leaves_the_holder_unchanged() {
    let holder = SeatsFormFormValueHolder { seats: Some(13) };
    let before = holder.clone();
    let mut action = SaveSeats;
    let mut tracker: SeatsFormFormSubmission = SubmitTracker::new();

    tracker.begin();
    let result = block_on(action.submit(&holder));
    tracker.finish(result.map(|_| ()));
    assert!(!tracker.is_submit_successful(), "out-of-range seats fail");
    assert!(tracker.last_error().is_some());

    // The holder flows through `SubmitAction::submit(&H)` untouched.
    assert_eq!(holder, before);

    let valid = SeatsFormFormValueHolder { seats: Some(5) };
    tracker.begin();
    let result = block_on(action.submit(&valid));
    tracker.finish(result.map(|_| ()));
    assert!(tracker.is_submit_successful());
}

#[test]
fn submit_gate_composes_with_the_facade_validation_cache() {
    // The M2 composition: submit gating consumes the facade's live
    // `is_valid()` plus the tracker's re-entry guard.
    let mut state = SeatsFormFormState::new(SeatsFormFormValueHolder::default());
    let mut tracker: SeatsFormFormSubmission = SubmitTracker::new();

    // Invalid form: no submit attempt starts.
    assert!(!state.is_valid());
    if state.is_valid() && tracker.begin() {
        panic!("invalid form must not begin a submit");
    }
    assert_eq!(tracker.submit_count(), 0);

    // Fix the field through the facade (which also keeps the cache honest)
    // and try again.
    state.current_mut().seats = Some(7);
    assert!(state.is_valid());
    assert!(tracker.begin());
    let result = block_on(SaveSeats.submit(state.current()));
    tracker.finish(result.map(|_| ()));
    assert!(tracker.is_submit_successful());

    // A successful save re-syncs the baseline through the facade.
    state.sync_baseline();
    assert!(!state.is_dirty());
}
