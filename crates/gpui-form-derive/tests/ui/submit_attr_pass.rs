use gpui_form::core::submit::{BoxError, SubmitTracker};
use gpui_form_derive::GpuiForm;

#[derive(GpuiForm, koruma::Koruma)]
#[gpui_form(koruma, submit)]
struct Signup {
    #[gpui_form(component(input))]
    #[koruma(koruma_collection::collection::NonEmptyValidation::<_>::builder())]
    username: String,
}

#[derive(GpuiForm)]
#[gpui_form(submit)]
struct Plain {
    #[gpui_form(component(input))]
    name: String,
}

#[derive(GpuiForm)]
#[gpui_form(empty, submit)]
struct Bare {}

fn main() {
    let mut tracker: SignupFormSubmission = SubmitTracker::new();
    assert!(tracker.can_submit());
    let failure: <SignupFormValueHolder as koruma::ValidateExt>::Error =
        SignupFormValueHolder::default()
            .validate()
            .err()
            .expect("empty username must fail");
    tracker.begin();
    tracker.finish(Err(failure));
    assert!(!tracker.is_submit_successful());
    assert!(tracker.last_error().is_some());

    let mut plain: PlainFormSubmission = SubmitTracker::<BoxError>::new();
    plain.set_error("boom".into());
    assert!(plain.last_error().is_some());

    let bare: BareFormSubmission = SubmitTracker::new();
    assert!(!bare.is_submitted());
}
