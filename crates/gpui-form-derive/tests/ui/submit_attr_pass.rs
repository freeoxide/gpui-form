// `#[gpui_form(submit)]` compiles end to end: a koruma form gets the
// typed-error alias (through ValidateExt), plain and empty forms get the
// BoxError alias. No `koruma(fluent)` here, so no trybuild locale seeding
// is needed.
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
    // Koruma route: the alias's error parameter is the typed koruma error.
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

    // Non-koruma route: the BoxError alias.
    let mut plain: PlainFormSubmission = SubmitTracker::<BoxError>::new();
    plain.set_error("boom".into());
    assert!(plain.last_error().is_some());

    // Empty-form route: the alias still emits.
    let bare: BareFormSubmission = SubmitTracker::new();
    assert!(!bare.is_submitted());
}
