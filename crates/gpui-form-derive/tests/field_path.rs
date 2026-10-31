use gpui_form::core::PathSegment;
use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct Profile {
    #[gpui_form(component(input))]
    name: String,

    #[gpui_form(component(input))]
    email: String,
}

#[derive(GpuiForm)]
struct Comment {
    #[gpui_form(component(input))]
    body: String,

    #[gpui_form(skip)]
    #[allow(dead_code)]
    audit_id: u64,
}

#[derive(GpuiForm)]
#[gpui_form(empty)]
struct Blank {}

#[test]
fn typed_constructor_matches_explicit_new_via_path() {
    let typed = ProfileFormPath::name();
    let explicit = ProfileFormPath::new(&["name"]);

    assert_eq!(typed.path(), explicit.path());
    assert_eq!(typed.path().segments(), &[PathSegment::Name("name")]);
}

#[test]
fn each_non_skipped_field_gets_a_constructor() {
    assert_eq!(
        ProfileFormPath::name().path().segments(),
        &[PathSegment::Name("name")]
    );
    assert_eq!(
        ProfileFormPath::email().path().segments(),
        &[PathSegment::Name("email")]
    );
    assert_ne!(
        ProfileFormPath::name().path(),
        ProfileFormPath::email().path()
    );
}

#[test]
fn display_formats_to_field_name() {
    assert_eq!(ProfileFormPath::name().to_string(), "name");
    assert_eq!(ProfileFormPath::email().to_string(), "email");
    let nested = ProfileFormPath::new(&["name", "first"]);
    assert_eq!(nested.to_string(), "name.first");
}

#[test]
fn into_path_yields_shared_field_path() {
    let path: ::gpui_form::core::FieldPath = ProfileFormPath::name().into_path();
    assert_eq!(path.segments(), &[PathSegment::Name("name")]);
    assert_eq!(path.to_string(), "name");
}

#[test]
fn deref_reaches_field_path() {
    let typed = ProfileFormPath::name();
    assert_eq!(typed.segments(), &[PathSegment::Name("name")]);
    assert!(!typed.is_empty());
}

#[test]
fn as_ref_reaches_field_path() {
    let typed = ProfileFormPath::email();
    let as_ref: &::gpui_form::core::FieldPath = typed.as_ref();
    assert_eq!(as_ref.segments(), &[PathSegment::Name("email")]);
}

#[test]
fn skipped_field_has_no_constructor_but_others_do() {
    assert_eq!(
        CommentFormPath::body().path().segments(),
        &[PathSegment::Name("body")]
    );
    let _ = CommentFormPath::body();
}

#[test]
fn empty_form_still_emits_path_type() {
    let typed = BlankFormPath::new(&["anything"]);
    assert_eq!(typed.path().segments(), &[PathSegment::Name("anything")]);
    let owned: ::gpui_form::core::FieldPath = typed.into_path();
    assert_eq!(owned.segments(), &[PathSegment::Name("anything")]);
}

#[test]
fn two_forms_have_non_mixable_path_types() {
    let profile = ProfileFormPath::name();
    let comment = CommentFormPath::body();

    assert_eq!(profile.path().segments(), &[PathSegment::Name("name")]);
    assert_eq!(comment.path().segments(), &[PathSegment::Name("body")]);

    assert_ne!(profile.path(), comment.path());
}
