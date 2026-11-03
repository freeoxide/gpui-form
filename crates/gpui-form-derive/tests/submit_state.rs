use gpui_form::core::submit::SubmitTracker;
use gpui_form::{BoxError, SubmitAction};
use gpui_form_derive::GpuiForm;
use std::future::Future;
use std::task::Poll;

#[derive(GpuiForm, koruma::Koruma)]
#[gpui_form(submit)]
struct SeatsForm {
    #[gpui_form(component(number_input))]
    #[koruma(koruma_collection::numeric::RangeValidation::<_>::builder().min(1).max(12))]
    seats: u32,
}

#[derive(GpuiForm, koruma::Koruma)]
#[gpui_form(koruma, submit)]
struct NameForm {
    #[gpui_form(component(input))]
    name: String,
}

#[derive(GpuiForm)]
#[gpui_form(submit)]
struct PlainForm {
    #[gpui_form(component(input))]
    name: String,
}

#[derive(GpuiForm)]
#[gpui_form(empty, submit)]
struct BareForm {}

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
    let mut tracker: SeatsFormFormSubmission = SubmitTracker::new();
    assert!(!tracker.is_submitting());
    assert!(tracker.can_submit());
    assert_eq!(tracker.submit_count(), 0);

    let failure: <SeatsFormFormValueHolder as koruma::ValidateExt>::Error =
        SeatsFormFormValueHolder::default()
            .validate()
            .expect_err("absent default must fail");
    tracker.begin();
    tracker.finish(Err(failure));
    assert!(tracker.last_error().is_some());

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

    tracker.begin();
    assert!(tracker.is_submitting());
    tracker.reset();
    assert!(!tracker.is_submitting());
    assert_eq!(tracker.submit_count(), 0);
    assert!(!tracker.is_submitted());
    assert!(tracker.last_error().is_none());

    tracker.finish(Ok(()));
    assert!(!tracker.is_submitting());
    assert!(!tracker.is_submitted());
    assert!(!tracker.is_submit_successful());
    tracker.finish(Err(
        <SeatsFormFormValueHolder as koruma::ValidateExt>::Error::default(),
    ));
    assert!(tracker.last_error().is_none());

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

    assert_eq!(holder, before);

    let valid = SeatsFormFormValueHolder { seats: Some(5) };
    tracker.begin();
    let result = block_on(action.submit(&valid));
    tracker.finish(result.map(|_| ()));
    assert!(tracker.is_submit_successful());
}

#[test]
fn submit_gate_composes_with_the_facade_validation_cache() {
    let mut state = SeatsFormFormState::new(SeatsFormFormValueHolder::default());
    let mut tracker: SeatsFormFormSubmission = SubmitTracker::new();

    assert!(!state.is_valid());
    if state.is_valid() && tracker.begin() {
        panic!("invalid form must not begin a submit");
    }
    assert_eq!(tracker.submit_count(), 0);

    state.current_mut().seats = Some(7);
    assert!(state.is_valid());
    assert!(tracker.begin());
    let result = block_on(SaveSeats.submit(state.current()));
    tracker.finish(result.map(|_| ()));
    assert!(tracker.is_submit_successful());

    state.sync_baseline();
    assert!(!state.is_dirty());
}
