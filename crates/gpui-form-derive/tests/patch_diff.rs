//! Field-level state model (M1, feature: per-field dirty/changed-fields +
//! patch, backlog #9): the derive emits `<Name>FormPatch` (one
//! `Option<holder-typed>` slot per non-skipped field) and the facade's
//! `is_field_dirty` / `changed_fields` / `patch` accessors.
//!
//! Storage tri-state coverage: `WrappedOption` (input on a plain field),
//! `OriginallyOptional` (input on an `Option` field), `Plain` (checkbox on a
//! `bool`), plus a `type`/`from`/`into` conversion field (comparison happens
//! on holder types, so conversions need no special handling).
//!
//! Deliberately NOT gated by `#![cfg(feature = "serde")]`: it must compile
//! under DEFAULT features, mirroring `form_state_dirty_without_serde.rs`
//! (the patch serde derives are gated in the derive crate, not here).

use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct Account {
    #[gpui_form(component(input))]
    name: String,

    #[gpui_form(component(number_input))]
    age: Option<u32>,

    #[gpui_form(component(checkbox))]
    newsletter: bool,
}

#[derive(Debug, GpuiForm)]
struct Converted {
    #[gpui_form(
        type = String,
        from = |raw: u32| format!("id-{raw}"),
        into = |text: String| text.trim_start_matches("id-").parse::<u32>().unwrap(),
        component(input)
    )]
    count: u32,

    #[gpui_form(skip)]
    #[allow(dead_code)] // skipped fields are intentionally absent from the patch
    audit: bool,
}

#[test]
fn two_field_holder_mutating_one_marks_only_that_field() {
    let mut state = AccountFormState::new(AccountFormValueHolder::default());
    assert!(state.patch().is_empty());

    state.current_mut().age = Some(42);
    assert!(state.is_dirty());
    assert!(state.is_field_dirty(&AccountFormPath::age()));
    assert!(!state.is_field_dirty(&AccountFormPath::name()));
    assert!(!state.is_field_dirty(&AccountFormPath::newsletter()));

    assert_eq!(
        state.changed_fields(),
        vec![AccountFormPath::age()],
        "changed_fields lists exactly the mutated field, in declaration order"
    );

    let patch = state.patch();
    assert!(patch.age.is_some(), "only the changed slot is set");
    assert!(patch.name.is_none());
    assert!(patch.newsletter.is_none());
}

#[test]
fn wrapped_option_slot_reads_as_option_option() {
    // `name: String` + input => holder field `Option<String>` => patch slot
    // `Option<Option<String>>`. Some(None) means "changed to cleared".
    let mut state = AccountFormState::new(AccountFormValueHolder::default());
    state.current_mut().name = Some("ada".to_string());
    let slot: Option<Option<String>> = state.patch().name;
    assert_eq!(slot, Some(Some("ada".to_string())));

    let mut cleared = AccountFormState::new(AccountFormValueHolder {
        name: Some("ada".to_string()),
        age: None,
        newsletter: false,
    });
    cleared.current_mut().name = None;
    let cleared_slot: Option<Option<String>> = cleared.patch().name;
    assert_eq!(cleared_slot, Some(None), "Some(None) = changed to cleared");
}

#[test]
fn originally_optional_and_plain_slots_are_holder_typed() {
    // `age: Option<u32>` (originally optional) => slot Option<Option<u32>>;
    // `newsletter: bool` (plain) => slot Option<bool>.
    let mut state = AccountFormState::new(AccountFormValueHolder::default());
    state.current_mut().age = Some(7);
    state.current_mut().newsletter = true;

    let patch = state.patch();
    let age_slot: Option<Option<u32>> = patch.age;
    let newsletter_slot: Option<bool> = patch.newsletter;
    assert_eq!(age_slot, Some(Some(7)));
    assert_eq!(newsletter_slot, Some(true));

    let changed: Vec<String> = state
        .changed_fields()
        .iter()
        .map(|path| path.to_string())
        .collect();
    assert_eq!(changed, vec!["age", "newsletter"]);
}

#[test]
fn conversion_fields_patch_on_holder_types() {
    let mut state = ConvertedFormState::new(ConvertedFormValueHolder::default());
    assert!(state.patch().is_empty());
    assert!(!state.is_field_dirty(&ConvertedFormPath::count()));

    state.current_mut().count = Some("id-7".to_string());
    assert!(state.is_field_dirty(&ConvertedFormPath::count()));
    let patch = state.patch();
    let slot: Option<Option<String>> = patch.count;
    assert_eq!(slot, Some(Some("id-7".to_string())));
    // Skipped field has no slot at all (this file only referencing `count`
    // proves the patch has no `audit` member).
    assert_eq!(state.changed_fields(), vec![ConvertedFormPath::count()]);
}

#[test]
fn apply_to_writes_only_changed_fields() {
    let mut state = AccountFormState::new(AccountFormValueHolder::default());
    state.current_mut().name = Some("ada".to_string());
    state.current_mut().newsletter = true;
    let patch = state.patch();

    let mut fresh = AccountFormValueHolder {
        name: None,
        age: Some(99),
        newsletter: false,
    };
    patch.apply_to(&mut fresh);
    assert_eq!(fresh.name.as_deref(), Some("ada"));
    assert_eq!(fresh.age, Some(99), "unchanged slot left untouched");
    assert!(fresh.newsletter);
}

#[test]
fn patch_between_is_pure_and_uses_declaration_order() {
    let from = AccountFormValueHolder::default();
    let to = AccountFormValueHolder {
        newsletter: true,
        ..Default::default()
    };
    let patch = AccountFormPatch::between(&from, &to);
    assert!(!patch.is_empty());
    assert_eq!(patch.changed_fields(), vec![AccountFormPath::newsletter()]);
    // `from` is untouched by the diff.
    assert_eq!(from, AccountFormValueHolder::default());
}

#[test]
fn patch_is_empty_right_after_sync_baseline_and_full_after_edits() {
    let mut state = AccountFormState::new(AccountFormValueHolder::default());
    state.current_mut().age = Some(3);
    assert!(!state.patch().is_empty());
    state.sync_baseline();
    assert!(state.patch().is_empty(), "matches is_dirty semantics");
    assert!(!state.is_dirty());
}
