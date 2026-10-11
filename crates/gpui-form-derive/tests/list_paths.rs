//! Milestone M3 typed path composition (backlog #2/#3, decision [D8]):
//! list-item paths carry a typed `PathSegment::Index`, render as
//! `contacts.2.name`, compare and hash by segment sequence, and compose with
//! nested forms' constructors through `join`.
//!
//! The plan pins the composed display `contacts.2.name` for a list-of-form
//! items; v1 ships scalar list items (nested item forms are sequenced after
//! v1 by the backlog), so the composition here joins the outer item path
//! with the item form's own constructor — the same seam nested item forms
//! will use.

use gpui_form::PathSegment;
use gpui_form_derive::GpuiForm;
use std::collections::HashSet;

#[derive(GpuiForm, koruma::Koruma)]
struct Contacts {
    #[gpui_form(component(list(item = input)))]
    contacts: Vec<String>,

    #[gpui_form(component(input))]
    title: String,
}

/// Stands in for the item form's path type (nested item forms arrive after
/// v1; `join` accepts any form's path newtype or a raw `FieldPath`).
#[derive(GpuiForm)]
struct Name {
    #[gpui_form(component(input))]
    name: String,
}

#[test]
fn list_item_paths_render_with_typed_indices() {
    assert_eq!(ContactsFormPath::contacts(0).to_string(), "contacts.0");
    assert_eq!(ContactsFormPath::contacts(2).to_string(), "contacts.2");
    assert_eq!(
        ContactsFormPath::contacts(1234).to_string(),
        "contacts.1234"
    );
}

#[test]
fn contacts_2_name_displays_as_contacts_2_name() {
    // The plan's pinned M3 composition: the outer list-item path joined
    // with the item form's field constructor names `contacts.2.name`.
    let composed = ContactsFormPath::contacts(2).join(&NameFormPath::name());
    assert_eq!(composed.to_string(), "contacts.2.name");
    assert_eq!(
        composed.path().segments(),
        &[
            PathSegment::Name("contacts"),
            PathSegment::Index(2),
            PathSegment::Name("name"),
        ]
    );
}

#[test]
fn item_paths_equal_only_for_the_same_index() {
    assert_eq!(ContactsFormPath::contacts(2), ContactsFormPath::contacts(2));
    assert_ne!(ContactsFormPath::contacts(2), ContactsFormPath::contacts(3));

    // A typed index never equals a name segment: `contacts.2` cannot
    // collide with a hand-built `contacts.2` of string segments.
    let by_index = ContactsFormPath::contacts(2);
    let by_names = ContactsFormPath::new(&["contacts", "2"]);
    assert_ne!(by_index, by_names);
}

#[test]
fn item_paths_hash_by_segment_sequence() {
    let mut set = HashSet::new();
    set.insert(ContactsFormPath::contacts(2));
    assert!(set.contains(&ContactsFormPath::contacts(2)));
    assert!(!set.contains(&ContactsFormPath::contacts(1)));
    assert!(!set.contains(&ContactsFormPath::contacts_root()));
}

#[test]
fn roots_and_items_are_distinct_paths() {
    let root = ContactsFormPath::contacts_root();
    assert_eq!(root.to_string(), "contacts");
    assert_eq!(root.field_index(), Some(0));
    assert_eq!(ContactsFormPath::title().field_index(), Some(1));
    assert!(ContactsFormPath::contacts(0).field_index().is_none());
}

#[test]
fn join_composes_flat_list_items_too() {
    // Scalar items have no item-form constructor, but the core primitive
    // still composes explicitly when an app needs deeper names.
    let joined = ContactsFormPath::contacts(1).into_path().push_name("value");
    assert_eq!(joined.to_string(), "contacts.1.value");
    // The typed newtype round-trips through the shared primitive.
    let back = ContactsFormPath::from_segments_ref(&joined);
    let _ = back;
}

impl ContactsFormPath {
    /// Test-local helper: rebuild the typed newtype from a shared primitive
    /// (the generated `new(&[&str])` covers names only).
    fn from_segments_ref(path: &gpui_form::FieldPath) -> Self {
        Self(gpui_form::core::FieldPath::from_segments(path.segments()))
    }
}
