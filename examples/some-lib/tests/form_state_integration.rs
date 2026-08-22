//! Feature #1 adversarial integration check: drive [`gpui_form::FormState`] over
//! REAL generated holders (the `User` form, a skipped-field form, and a
//! custom-default form) and confirm the generated holders actually provide
//! `PartialEq` + serde under the `serde` feature so `is_dirty` / round-trip
//! compile and behave.
//!
//! Gated on `serde` because it depends on the holder deriving
//! `Serialize`/`Deserialize`/`PartialEq`, which the derive only emits under
//! that feature.
//!
//! Ported to the component-shape architecture: holder storage now follows each
//! shape's `ValueStoragePolicy` — `Input::<_>` over `String` stores `String`
//! directly, `Option<T>` sources stay optional, `NumberInput::<T>` stores `T`.

#![cfg(feature = "serde")]

use gpui_form::FormState;
use gpui_form_derive::GpuiForm;
use some_lib::structs::user::{
    EnumCountry, PreferredLanguage, Timestamp, User, UserFormValueHolder as UserHolder,
};

/// Construct a `User` with non-default values so a fresh holder is observably
/// distinct from a default one. Generated holders live at the crate root of
/// the crate that derives `GpuiForm` (here, `some_lib`).
fn sample_user() -> User {
    User {
        username: "Xalpha xX".into(),
        email: "test@example.com".into(),
        age: Some(42),
        balance: rust_decimal::Decimal::new(12345, 2),
        debt: rust_decimal::Decimal::new(-678, 2),
        rating: Some(5),
        attention_level: 0.5,
        brand_color: None,
        otp_code: "123456".into(),
        uploaded_files: Vec::new(),
        holiday_range: None,
        subscribe_newsletter: true,
        enable_notifications: false,
        preferred: PreferredLanguage::French,
        country: Some(EnumCountry::France),
        birth_date: Timestamp::from_micros_since_unix_epoch(1_700_000_000_000_000),
        skip_me: false,
    }
}

type _UserHolderAlias = UserHolder;

#[test]
fn form_state_over_real_user_holder_is_not_dirty_initially() {
    // Check 2 (PartialEq bound): this only compiles because the generated
    // UserFormValueHolder derives PartialEq under the serde feature.
    let holder = UserHolder::from(sample_user());
    let state = FormState::new(holder);
    assert!(!state.is_dirty());
}

#[test]
fn form_state_over_real_user_holder_tracks_edits_and_resets() {
    let holder = UserHolder::from(sample_user());
    let mut state = FormState::new(holder);

    // Mutate a Direct-storage field (Input::<_> over String) and an
    // Option<u32> field (OriginallyOptional storage).
    state.current_mut().username = "Xbeta xX".into();
    state.current_mut().age = None;
    assert!(state.is_dirty());

    // Reset undoes both edits.
    state.reset_to_baseline();
    assert!(!state.is_dirty());
    assert_eq!(state.current().username, "Xalpha xX");
    assert_eq!(state.current().age, Some(42));
}

#[test]
fn real_user_holder_round_trips_through_serde() {
    // Direct-storage and Option fields (age, country) and the fixed-precision
    // Decimal fields all round-trip. This also proves the holder derives
    // PartialEq (needed for the assert_eq) WITHOUT deriving Eq: genuinely
    // non-Eq field types (f32 slider values) would break an `Eq` derive,
    // which is why the derive emits `PartialEq` only.
    let holder = UserHolder::from(sample_user());

    let json = serde_json::to_string(&holder).expect("serialize");
    let restored: UserHolder = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(restored, holder, "holder must round-trip equal");

    // The skipped field `skip_me` must NOT appear in the payload.
    assert!(
        !json.contains("skip_me"),
        "skipped field leaked into serialized holder: {json}"
    );
}

#[test]
fn real_user_holder_diff_against_external_value() {
    let holder = UserHolder::from(sample_user());
    let state = FormState::new(holder);

    // An externally-built equal holder is not different.
    let same = UserHolder::from(sample_user());
    assert!(!state.diff_against(&same));

    // A different holder is.
    let mut other = UserHolder::from(sample_user());
    other.balance = rust_decimal::Decimal::ZERO;
    assert!(state.diff_against(&other));
}

/// A second form with a skipped field, exercised through FormState to prove
/// the holder (not the source struct) is what dirty tracking sees.
#[derive(GpuiForm)]
#[gpui_form(partial_eq)]
struct NoteForm {
    #[gpui_form(component(gpui_form_collection::input::Input::<_>))]
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

    state.current_mut().body = "world".into();
    assert!(state.is_dirty());

    state.sync_baseline();
    assert!(!state.is_dirty());
}

/// Check 3 (custom-default): the source form has no `Default` impl and no
/// field defaults. FormState must not require `Holder: Default` — only `Clone`
/// (and `PartialEq` for `is_dirty`). This compiles only if those bounds are
/// the sole requirements.
#[derive(GpuiForm)]
#[gpui_form(partial_eq)]
struct CustomDefaultForm {
    #[gpui_form(component(gpui_form_collection::input::Input::<_>))]
    label: String,
    #[gpui_form(component(gpui_form_collection::number_input::NumberInput::<_>))]
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
    state.current_mut().score = rust_decimal::Decimal::new(6, 0);
    assert!(state.is_dirty());

    // replace_current + into_current exercise Clone only (no Default bound).
    let live = state.into_current();
    assert_eq!(live.score, rust_decimal::Decimal::new(6, 0));
}
