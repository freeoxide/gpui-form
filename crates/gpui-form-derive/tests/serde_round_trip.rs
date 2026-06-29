//! Feature #1 (form-state persistence): end-to-end serde round-trip of the
//! generated `...FormValueHolder`. Gated behind the `serde` feature, so it is
//! excluded from the default-feature build that proves feature-OFF still works.

#![cfg(feature = "serde")]

use gpui_form_derive::GpuiForm;

/// Plain + `Option` fields, no skipped fields: the holder round-trips through
/// serde and converts both ways via `From`.
#[derive(GpuiForm)]
struct ContactForm {
    #[gpui_form(component(input))]
    name: String,

    #[gpui_form(component(input))]
    nickname: Option<String>,
}

/// A skipped field: the HOLDER still round-trips through serde. Reconstructing
/// the source struct is impossible without the skipped value, which is the
/// documented limitation mirrored by `has_skipped_fields`.
#[derive(GpuiForm)]
struct NoteForm {
    #[gpui_form(component(input))]
    body: String,

    #[gpui_form(skip)]
    #[allow(dead_code)] // skipped fields are intentionally absent from the holder
    internal_id: u32,
}

#[test]
fn holder_round_trips_plain_and_option_fields() {
    let original = ContactForm {
        name: "Ada".to_string(),
        nickname: Some("Countess".to_string()),
    };

    // Source -> holder.
    let holder = ContactFormFormValueHolder::from(original);

    // holder -> JSON -> holder.
    let json = serde_json::to_string(&holder).expect("serialize holder");
    let restored: ContactFormFormValueHolder =
        serde_json::from_str(&json).expect("deserialize holder");

    assert_eq!(restored, holder, "round-tripped holder must equal original");

    // holder -> source still works (no skipped fields).
    let back: ContactForm = restored.into();
    assert_eq!(back.name, "Ada");
    assert_eq!(back.nickname.as_deref(), Some("Countess"));
}

#[test]
fn holder_with_none_option_round_trips() {
    let original = ContactForm {
        name: "Babbage".to_string(),
        nickname: None,
    };
    let holder = ContactFormFormValueHolder::from(original);

    let json = serde_json::to_string(&holder).expect("serialize holder");
    let restored: ContactFormFormValueHolder =
        serde_json::from_str(&json).expect("deserialize holder");

    assert_eq!(restored, holder);
    assert!(restored.nickname.is_none());
}

#[test]
fn skipped_field_holder_round_trips() {
    // Only the non-skipped fields land on the holder, so it serializes and
    // deserializes independently of the skipped field.
    let original = NoteForm {
        body: "hello".to_string(),
        internal_id: 99,
    };
    let holder = NoteFormFormValueHolder::from(original);

    let json = serde_json::to_string(&holder).expect("serialize holder");
    assert!(
        !json.contains("internal_id"),
        "skipped field must not appear in serialized holder: {json}"
    );

    let restored: NoteFormFormValueHolder =
        serde_json::from_str(&json).expect("deserialize holder");
    assert_eq!(restored, holder);
    assert_eq!(restored.body.as_deref(), Some("hello"));
}

#[test]
fn holder_is_partial_eq_comparable() {
    // Confirms the generated holder derives PartialEq (required by
    // FormState::is_dirty). PartialEq is derived unconditionally — not just
    // under the serde feature — so dirty tracking works on default features;
    // this test exercises it under the serde build where this file compiles.
    let a = ContactFormFormValueHolder::from(ContactForm {
        name: "x".to_string(),
        nickname: None,
    });
    let b = ContactFormFormValueHolder::from(ContactForm {
        name: "x".to_string(),
        nickname: None,
    });
    assert_eq!(a, b);
}
