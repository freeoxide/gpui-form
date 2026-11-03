use gpui_form::{FieldPath, PathSegment};
use gpui_form_derive::GpuiForm;
use some_lib::structs::post::{AddressFormPath, PostEditorFormPath};
use some_lib::structs::user::UserFormPath;

#[test]
fn user_path_ctor_matches_explicit_new() {
    let ctor = UserFormPath::username();
    let explicit = UserFormPath::new(&["username"]);
    assert_eq!(ctor.path(), explicit.path());
}

#[test]
fn user_non_skipped_fields_each_have_ctor() {
    let username = UserFormPath::username();
    let age = UserFormPath::age();
    let birth_date = UserFormPath::birth_date();

    assert_eq!(username.to_string(), "username");
    assert_eq!(age.to_string(), "age");
    assert_eq!(birth_date.to_string(), "birth_date");
}

#[test]
fn user_path_display_renders_field_name() {
    let email = UserFormPath::email();
    assert_eq!(email.to_string(), "email");

    let shared = UserFormPath::username().into_path();
    assert_eq!(shared.to_string(), "username");
}

#[test]
fn user_path_deref_and_as_ref_reach_shared_primitive() {
    let path = UserFormPath::age();

    fn takes_as_ref(p: &FieldPath) -> &[PathSegment] {
        p.segments()
    }
    assert_eq!(takes_as_ref(path.as_ref()), &[PathSegment::Name("age")]);

    assert_eq!(path.segments(), &[PathSegment::Name("age")]);
    assert!(!path.is_empty());
}

#[test]
fn user_skipped_field_has_no_ctor_but_others_remain() {
    let a = UserFormPath::username();
    let b = UserFormPath::email();
    assert_ne!(a.to_string(), b.to_string());
    assert_ne!(a.path(), b.path());
}

#[test]
fn user_hand_built_multi_segment_path_works_now() {
    let nested = UserFormPath::new(&["address", "city"]);
    assert_eq!(nested.to_string(), "address.city");
    assert_eq!(
        nested.path().segments(),
        &[PathSegment::Name("address"), PathSegment::Name("city")]
    );
    assert!(!nested.is_empty());
}

#[test]
fn post_editor_composes_nested_and_list_paths() {
    let city = PostEditorFormPath::address_root().join(&AddressFormPath::city());
    assert_eq!(city.to_string(), "address.city");

    assert_eq!(PostEditorFormPath::tags(2).to_string(), "tags.2");
    assert_eq!(
        PostEditorFormPath::tags(2).path().segments(),
        &[PathSegment::Name("tags"), PathSegment::Index(2)]
    );
}

#[derive(GpuiForm)]
#[gpui_form(empty)]
struct EmptyForm {}

#[test]
fn empty_form_still_emits_path_type_without_ctors() {
    let p = EmptyFormFormPath::new(&["anything"]);
    assert_eq!(p.to_string(), "anything");
    assert_eq!(p.into_path().segments(), &[PathSegment::Name("anything")]);
}
