//! Feature #8 integration check: drive the REAL generated `UserFormPath` from
//! the existing `User` form, plus a tiny zero-non-skipped-field form to prove
//! the contract still emits the path type when there are no per-field ctors.
//!
//! NOT gated on `serde`: `FieldPath` is a headless primitive with no feature
//! flag of its own, so typed paths must work on the default feature set.
//!
//! Coverage: per-field ctors are named identically to non-skipped fields,
//! Display renders the field name, `Deref`/`AsRef`/`into_path` all reach the
//! shared `gpui_form::core::FieldPath` primitive, the skipped field `skip_me`
//! has NO ctor, and a hand-built multi-segment path still works (FLAT v1
//! boundary — typed nested/list composition lands with #2/#3).

use gpui_form::FieldPath;
use gpui_form_derive::GpuiForm;
use some_lib::structs::user::UserFormPath;

/// A field-ctor path matches an explicit `new()` for the same segment.
#[test]
fn user_path_ctor_matches_explicit_new() {
    let ctor = UserFormPath::username();
    let explicit = UserFormPath::new(&["username"]);
    assert_eq!(ctor.path(), explicit.path());
}

/// Every NON-skipped field on `User` gets a same-named ctor. Covers a plain
/// `String`, an `Option<u32>`, and a `type`/`from`/`into` override field —
/// paths are field-name-only and ignore type overrides.
#[test]
fn user_non_skipped_fields_each_have_ctor() {
    let username = UserFormPath::username();
    let age = UserFormPath::age();
    let birth_date = UserFormPath::birth_date();

    assert_eq!(username.to_string(), "username");
    assert_eq!(age.to_string(), "age");
    assert_eq!(birth_date.to_string(), "birth_date");
}

/// Display renders the single field name (FLAT v1 — one segment per ctor).
#[test]
fn user_path_display_renders_field_name() {
    let email = UserFormPath::email();
    assert_eq!(email.to_string(), "email");

    // `into_path` yields the shared primitive, which also Displays the name.
    let shared = UserFormPath::username().into_path();
    assert_eq!(shared.to_string(), "username");
}

/// `Deref` and `AsRef` both reach the shared `gpui_form::core::FieldPath`.
/// Confirms the typed newtype is interoperable with any code that takes the
/// headless primitive.
#[test]
fn user_path_deref_and_as_ref_reach_shared_primitive() {
    let path = UserFormPath::age();

    fn takes_as_ref(p: &FieldPath) -> &[&'static str] {
        p.segments()
    }
    assert_eq!(takes_as_ref(path.as_ref()), &["age"]);

    // Deref<Target = FieldPath> exposes `segments()` directly on the newtype.
    assert_eq!(path.segments(), &["age"]);
    assert!(!path.is_empty());
}

/// The skipped field `skip_me` is ABSENT from the ctor set. There is no
/// `UserFormPath::skip_me()` — calling it would be a compile error, so this
/// test instead asserts the OTHER ctors are unaffected by the skip.
#[test]
fn user_skipped_field_has_no_ctor_but_others_remain() {
    // If `skip_me` had a ctor, the line below would shadow this binding with
    // a different type from a different call. We instead just confirm several
    // non-skipped ctors are reachable and distinct.
    let a = UserFormPath::username();
    let b = UserFormPath::email();
    assert_ne!(a.to_string(), b.to_string());
    assert_ne!(a.path(), b.path());
}

/// Hand-built multi-segment paths work today via `new(&["a", "b"])`, ahead of
/// the typed nested/list ctors promised by backlog #2/#3.
#[test]
fn user_hand_built_multi_segment_path_works_now() {
    let nested = UserFormPath::new(&["address", "city"]);
    assert_eq!(nested.to_string(), "address.city");
    assert_eq!(nested.path().segments(), &["address", "city"]);
    assert!(!nested.is_empty());
}

/// The empty-form branch of the derive must still emit a path type with
/// `new()`/`path()`/`into_path()` even when there are zero per-field ctors.
#[derive(GpuiForm)]
#[gpui_form(empty)]
struct EmptyForm {}

#[test]
fn empty_form_still_emits_path_type_without_ctors() {
    // No per-field ctors exist, but the type, `new`, `path`, and `into_path` do.
    let p = EmptyFormFormPath::new(&["anything"]);
    assert_eq!(p.to_string(), "anything");
    assert_eq!(p.into_path().segments(), &["anything"]);
}
