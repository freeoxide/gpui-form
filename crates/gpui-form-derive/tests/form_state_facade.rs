use gpui_form::TouchedFields;
use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct Profile {
    #[gpui_form(component(input))]
    name: String,

    #[gpui_form(component(input))]
    email: String,

    #[gpui_form(skip)]
    #[allow(dead_code)]
    audit_id: u64,
}

#[derive(GpuiForm)]
#[gpui_form(empty)]
struct Blank {}

#[test]
fn field_count_and_names_match_non_skipped_fields() {
    assert_eq!(ProfileFormPath::FIELD_COUNT, 2);
    assert_eq!(ProfileFormPath::FIELD_NAMES, &["name", "email"]);

    assert_eq!(BlankFormPath::FIELD_COUNT, 0);
    assert_eq!(BlankFormPath::FIELD_NAMES, &[] as &[&str]);
}

#[test]
fn skipped_field_has_no_index_or_path() {
    assert!(!ProfileFormPath::FIELD_NAMES.contains(&"audit_id"));
    assert_eq!(ProfileFormPath::new(&["audit_id"]).field_index(), None);
    assert_eq!(ProfileFormPath::path_for_index(2), None);
}

#[test]
fn field_index_and_path_for_index_round_trip() {
    let name = ProfileFormPath::name();
    let email = ProfileFormPath::email();
    assert_eq!(name.field_index(), Some(0));
    assert_eq!(email.field_index(), Some(1));
    assert_eq!(
        ProfileFormPath::path_for_index(0).as_ref(),
        Some(&name),
        "path_for_index inverts field_index"
    );
    assert_eq!(
        ProfileFormPath::path_for_index(1).as_ref(),
        Some(&email),
        "path_for_index inverts field_index"
    );
    assert_eq!(ProfileFormPath::path_for_index(9), None);

    assert_eq!(ProfileFormPath::new(&["name", "deep"]).field_index(), None);
    assert_eq!(ProfileFormPath::new(&["nope"]).field_index(), None);
}

#[test]
fn all_paths_iterates_in_declaration_order() {
    let paths: Vec<String> = ProfileFormPath::all_paths()
        .map(|p| p.to_string())
        .collect();
    assert_eq!(paths, vec!["name", "email"]);
    assert_eq!(BlankFormPath::all_paths().count(), 0);
}

#[test]
fn facade_starts_clean_and_untouched() {
    let state = ProfileFormState::new(ProfileFormValueHolder::default());
    assert!(!state.is_dirty());
    assert!(!state.is_any_touched());
    assert!(state.touched_fields().is_empty());
    assert!(state.changed_fields().is_empty());
    assert!(state.patch().is_empty());
}

#[test]
fn touch_and_query_touched_fields() {
    let mut state = ProfileFormState::new(ProfileFormValueHolder::default());
    state.touch(&ProfileFormPath::name());
    assert!(state.is_touched(&ProfileFormPath::name()));
    assert!(!state.is_touched(&ProfileFormPath::email()));
    assert!(state.is_any_touched());

    let touched = state.touched_fields();
    assert_eq!(touched.len(), 1);
    assert_eq!(touched[0].to_string(), "name");

    state.touch(&ProfileFormPath::new(&["nope"]));
    assert!(!state.is_touched(&ProfileFormPath::new(&["nope"])));

    state.clear_touched(&ProfileFormPath::name());
    assert!(!state.is_any_touched());

    state.touch(&ProfileFormPath::email());
    state.clear_all_touched();
    assert!(!state.is_any_touched());
}

#[test]
fn touched_survives_value_resets_when_asked_and_clears_by_default() {
    let mut state = ProfileFormState::new(ProfileFormValueHolder::default());
    state.current_mut().name = Some("ada".to_string());
    state.touch(&ProfileFormPath::name());
    assert!(state.is_dirty());

    state.reset_to_baseline();
    assert!(!state.is_dirty());
    assert!(!state.is_any_touched());
    assert_eq!(state.current().name, None);

    state.current_mut().name = Some("grace".to_string());
    state.touch(&ProfileFormPath::name());
    state.reset_to_baseline_keeping_touched();
    assert!(!state.is_dirty());
    assert!(state.is_touched(&ProfileFormPath::name()));
}

#[test]
fn sync_baseline_and_replace_current_keep_touched() {
    let mut state = ProfileFormState::new(ProfileFormValueHolder::default());
    state.touch(&ProfileFormPath::email());

    state.current_mut().email = Some("a@b.c".to_string());
    state.sync_baseline();
    assert!(!state.is_dirty());
    assert!(state.is_touched(&ProfileFormPath::email()));

    let saved = state.baseline().clone();
    state.replace_current(saved);
    assert!(!state.is_dirty());
    assert!(state.is_touched(&ProfileFormPath::email()));
}

#[test]
fn state_escape_hatches_reach_the_core_state() {
    let mut state = ProfileFormState::new(ProfileFormValueHolder::default());
    assert!(!state.state().is_dirty());
    state.state_mut().current_mut().name = Some("z".to_string());
    assert!(state.state().is_dirty());
    assert_eq!(state.current().name.as_deref(), Some("z"));
    assert_eq!(state.baseline().name, None);

    let live = state.into_current();
    assert_eq!(live.name.as_deref(), Some("z"));
}

#[test]
fn empty_form_emits_bare_facade_and_patch() {
    let mut state = BlankFormState::new(BlankFormValueHolder::default());
    assert!(!state.is_dirty());
    assert!(!state.is_any_touched());
    assert!(state.touched_fields().is_empty());
    assert!(state.changed_fields().is_empty());
    assert!(state.patch().is_empty());

    state.touch(&BlankFormPath::new(&["anything"]));
    assert!(!state.is_any_touched());
    assert!(!state.is_field_dirty(&BlankFormPath::new(&["anything"])));
}

#[test]
fn touched_bitset_sizes_from_field_count() {
    let core = TouchedFields::new(ProfileFormPath::FIELD_COUNT);
    assert_eq!(core.field_count(), 2);
}
