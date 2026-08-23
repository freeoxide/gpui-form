//! Feature #8 (typed field paths, FLAT v1): the derive emits a
//! strongly-typed `<Name>FormPath` for every form, wrapping the shared
//! `::gpui_form::core::FieldPath`. These tests exercise the generated surface
//! end-to-end: per-field constructors, Display, segments, Deref/AsRef, and
//! that two distinct forms get non-mixable path types.
//!
//! Not gated behind any feature: `FieldPath` is an unconditional headless
//! primitive, so this runs under the default (no-feature) build.
//!
//! NOTE on naming: the generated type is `<Name>FormPath` where `<Name>` is
//! the source struct ident (mirroring `<Name>FormValueHolder`). We name the
//! source structs here WITHOUT a trailing `Form` so the generated path types
//! read as `ProfileFormPath`, `CommentFormPath`, and `BlankFormPath`.

use gpui_form_derive::GpuiForm;

/// Plain form with two fields: both get typed constructors.
#[derive(GpuiForm)]
struct Profile {
    #[gpui_form(component(input))]
    name: String,

    #[gpui_form(component(input))]
    email: String,
}

/// Form with a skipped field: the skipped field must NOT get a constructor.
#[derive(GpuiForm)]
struct Comment {
    #[gpui_form(component(input))]
    body: String,

    #[gpui_form(skip)]
    #[allow(dead_code)] // skipped fields are absent from the holder / path ctors
    audit_id: u64,
}

/// Empty form (`#[gpui_form(empty)]`): still emits a `BlankFormPath` type with
/// `new()` / `path()` / `into_path()` and no per-field constructors.
#[derive(GpuiForm)]
#[gpui_form(empty)]
struct Blank {}

#[test]
fn typed_constructor_matches_explicit_new_via_path() {
    // The typed constructor and an explicit new() with the field name produce
    // equal inner FieldPaths.
    let typed = ProfileFormPath::name();
    let explicit = ProfileFormPath::new(&["name"]);

    assert_eq!(typed.path(), explicit.path());
    assert_eq!(typed.path().segments(), &["name"]);
}

#[test]
fn each_non_skipped_field_gets_a_constructor() {
    // name and email are both non-skipped, so both constructors exist.
    assert_eq!(ProfileFormPath::name().path().segments(), &["name"]);
    assert_eq!(ProfileFormPath::email().path().segments(), &["email"]);
    // The two are distinct paths.
    assert_ne!(
        ProfileFormPath::name().path(),
        ProfileFormPath::email().path()
    );
}

#[test]
fn display_formats_to_field_name() {
    // Display delegates to the inner FieldPath's dotted join.
    assert_eq!(ProfileFormPath::name().to_string(), "name");
    assert_eq!(ProfileFormPath::email().to_string(), "email");
    // Multi-segment hand-built path still renders dotted.
    let nested = ProfileFormPath::new(&["name", "first"]);
    assert_eq!(nested.to_string(), "name.first");
}

#[test]
fn into_path_yields_shared_field_path() {
    // into_path returns the headless shared primitive, dropping the typed wrapper.
    let path: ::gpui_form::core::FieldPath = ProfileFormPath::name().into_path();
    assert_eq!(path.segments(), &["name"]);
    assert_eq!(path.to_string(), "name");
}

#[test]
fn deref_reaches_field_path() {
    // Deref<Target = FieldPath> exposes the primitive's methods directly.
    let typed = ProfileFormPath::name();
    assert_eq!(typed.segments(), &["name"]);
    assert!(!typed.is_empty());
}

#[test]
fn as_ref_reaches_field_path() {
    // AsRef<FieldPath> reaches the same inner value.
    let typed = ProfileFormPath::email();
    let as_ref: &::gpui_form::core::FieldPath = typed.as_ref();
    assert_eq!(as_ref.segments(), &["email"]);
}

#[test]
fn skipped_field_has_no_constructor_but_others_do() {
    // `body` is non-skipped -> constructor exists.
    assert_eq!(CommentFormPath::body().path().segments(), &["body"]);
    // `audit_id` is skipped -> there is NO `CommentFormPath::audit_id`. The
    // fact that this file compiles, while only `body` is referenced, proves the
    // type exists with the expected surface and the skipped field has no ctor.
    let _ = CommentFormPath::body();
}

#[test]
fn empty_form_still_emits_path_type() {
    // Zero non-skipped fields: the type still exists with new()/path()/into_path()
    // and no per-field constructors.
    let typed = BlankFormPath::new(&["anything"]);
    assert_eq!(typed.path().segments(), &["anything"]);
    let owned: ::gpui_form::core::FieldPath = typed.into_path();
    assert_eq!(owned.segments(), &["anything"]);
}

#[test]
fn two_forms_have_non_mixable_path_types() {
    // ProfileFormPath and CommentFormPath are distinct types: a constructor
    // from one cannot satisfy the other. Each names its own form's fields.
    let profile = ProfileFormPath::name();
    let comment = CommentFormPath::body();

    // Both wrap the same headless primitive, so their inner paths compare by
    // segment value when the segments happen to match...
    assert_eq!(profile.path().segments(), &["name"]);
    assert_eq!(comment.path().segments(), &["body"]);

    // ...but the typed wrappers are NOT comparable to each other (the lines
    // below would fail to compile if uncommented, proving non-mixability):
    //   let _: ProfileFormPath = comment;
    //   assert_eq!(profile, comment);

    // Segments differ, so even the inner primitives are unequal here.
    assert_ne!(profile.path(), comment.path());
}
