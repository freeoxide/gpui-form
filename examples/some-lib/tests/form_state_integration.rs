#![cfg(feature = "serde")]

use gpui_form::FormState;
use gpui_form_derive::GpuiForm;
use some_lib::structs::user::{
    EnumCountry, PreferredLanguage, User, UserFormValueHolder as UserHolder,
};

fn sample_user() -> User {
    User {
        username: "Xalpha xX".into(),
        email: "test@example.com".into(),
        age: Some(42),
        balance: rust_decimal::Decimal::new(12345, 2),
        debt: rust_decimal::Decimal::new(-678, 2),
        subscribe_newsletter: true,
        enable_notifications: false,
        preferred: PreferredLanguage::French,
        country: Some(EnumCountry::France),
        state: None,
        birth_date: None,
        skip_me: false,
    }
}

type _UserHolderAlias = UserHolder;

#[test]
fn form_state_over_real_user_holder_is_not_dirty_initially() {
    let holder = UserHolder::from(sample_user());
    let state = FormState::new(holder);
    assert!(!state.is_dirty());
}

#[test]
fn form_state_over_real_user_holder_tracks_edits_and_resets() {
    let holder = UserHolder::from(sample_user());
    let mut state = FormState::new(holder);

    state.current_mut().username = Some("Xbeta xX".into());
    state.current_mut().age = None;
    assert!(state.is_dirty());

    state.reset_to_baseline();
    assert!(!state.is_dirty());
    assert_eq!(state.current().username.as_deref(), Some("Xalpha xX"));
    assert_eq!(state.current().age, Some(42));
}

#[test]
fn real_user_holder_round_trips_through_serde() {
    let holder = UserHolder::from(sample_user());

    let json = serde_json::to_string(&holder).expect("serialize");
    let restored: UserHolder = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(restored, holder, "holder must round-trip equal");

    assert!(
        !json.contains("skip_me"),
        "skipped field leaked into serialized holder: {json}"
    );
}

#[test]
fn real_user_holder_diff_against_external_value() {
    let holder = UserHolder::from(sample_user());
    let state = FormState::new(holder);

    let same = UserHolder::from(sample_user());
    assert!(!state.diff_against(&same));

    let mut other = UserHolder::from(sample_user());
    other.balance = Some(rust_decimal::Decimal::ZERO);
    assert!(state.diff_against(&other));
}

#[derive(GpuiForm)]
struct NoteForm {
    #[gpui_form(component(input))]
    body: String,
    #[gpui_form(skip)]
    #[allow(dead_code)]
    internal_id: u32,
}

#[test]
fn form_state_over_skipped_field_holder() {
    let holder = NoteFormFormValueHolder::from(NoteForm {
        body: "hello".into(),
        internal_id: 7,
    });
    let mut state = FormState::new(holder);
    assert!(!state.is_dirty());

    state.current_mut().body = Some("world".into());
    assert!(state.is_dirty());

    state.sync_baseline();
    assert!(!state.is_dirty());
}

#[derive(GpuiForm)]
struct CustomDefaultForm {
    #[gpui_form(component(input))]
    label: String,
    #[gpui_form(component(number_input(as = f64)))]
    score: rust_decimal::Decimal,
}

#[test]
fn form_state_over_non_default_holder() {
    let holder = CustomDefaultFormFormValueHolder::from(CustomDefaultForm {
        label: "x".into(),
        score: rust_decimal::Decimal::new(5, 0),
    });
    let mut state = FormState::new(holder);
    assert!(!state.is_dirty());
    state.current_mut().score = Some(rust_decimal::Decimal::new(6, 0));
    assert!(state.is_dirty());

    let live = state.into_current();
    assert_eq!(live.score, Some(rust_decimal::Decimal::new(6, 0)));
}
