//! Schema export (M5, backlog #14): every form emits `<Name>FormSchema`
//! whose `schema()` accessor returns an owned, serializable projection of the
//! form's metadata — built from the same `FieldVariant`/`FieldLayout` chain
//! the inventory path builds (single metadata source).
//!
//! Coverage: the exported document includes the nested/list composition
//! metadata from M3 and the dependency metadata from M4 (`depends_on` /
//! `required_if`), layout hints, validation rules, and skipped-field
//! semantics; and the document round-trips through serde_json.

use gpui_form::schema::registry::{FormSchema, FormSchemaComponent};
use gpui_form_derive::GpuiForm;

/// The nested form (M3 backlog #2): derives GpuiForm itself.
#[derive(Clone, Debug, Default, GpuiForm, Eq, PartialEq, koruma::Koruma)]
pub struct Address {
    #[gpui_form(component(input))]
    #[koruma(koruma_collection::collection::NonEmptyValidation::<_>::builder())]
    pub city: String,

    #[gpui_form(component(input))]
    pub zip: Option<String>,
}

/// The exported form: flat + dependency (M4) + nested (M3) + list (M3)
/// fields, layout hints, and a skipped field.
#[derive(GpuiForm, koruma::Koruma)]
struct Profile {
    #[gpui_form(component(switch))]
    subscribe: bool,

    #[gpui_form(
        component(input),
        section = "Contact",
        label = "Email address",
        width = half,
        depends_on = subscribe,
        required_if = subscribe
    )]
    #[koruma(koruma_collection::collection::NonEmptyValidation::<_>::builder())]
    email: Option<String>,

    #[gpui_form(component(nested))]
    #[koruma(nested)]
    address: Address,

    #[gpui_form(component(list(item = input)))]
    #[koruma(each(koruma_collection::collection::LenValidation::<_>::builder().min(1).max(5)))]
    tags: Vec<String>,

    #[gpui_form(skip)]
    #[allow(dead_code)] // skipped fields are intentionally absent from the holder
    internal_id: u32,
}

/// Empty forms export an empty field list (both expansion return sites emit
/// the schema surface — pipeline hazard 1).
#[derive(GpuiForm)]
#[gpui_form(empty)]
struct Blank {}

/// A plain form (no koruma derive, no koruma attrs) whose `name` field is
/// "required-ish" — `component(input)` wraps it in `Option`, so the
/// required-shaped runtime slot exists but nothing ever enforces required
/// (there is no Koruma derive). Its export must not claim `RequiredValidation`.
#[derive(GpuiForm)]
struct Plain {
    #[gpui_form(component(input))]
    name: String,

    #[gpui_form(component(input))]
    nickname: Option<String>,
}

#[test]
fn schema_exports_the_form_shape_and_skipped_field_semantics() {
    let schema = ProfileFormSchema::schema();

    assert_eq!(schema.form, "Profile");
    assert!(schema.koruma_enabled, "koruma forms record it");
    assert!(schema.has_skipped_fields, "the skipped field is flagged");
    assert!(schema.has_validations());
    assert!(
        schema.source_path.ends_with("schema_export.rs"),
        "source_path comes from file!(): {}",
        schema.source_path
    );

    // Skipped fields are absent; declaration order is preserved.
    assert_eq!(
        schema.field_names().collect::<Vec<_>>(),
        vec!["subscribe", "email", "address", "tags"]
    );
    assert!(schema.field("internal_id").is_none());
}

#[test]
fn schema_includes_m4_dependency_metadata() {
    let schema = ProfileFormSchema::schema();
    let email = schema.field("email").expect("email entry");

    assert_eq!(email.depends_on.as_deref(), Some("subscribe"));
    assert_eq!(email.required_if.as_deref(), Some("subscribe"));
    // The conditional rule is recorded as a validation kind alongside the
    // mirrored koruma validator.
    assert!(
        email
            .validations
            .contains(&"RequiredIfValidation".to_string())
    );
    assert!(
        email
            .validations
            .contains(&"NonEmptyValidation".to_string())
    );
}

#[test]
fn schema_includes_m3_nested_and_list_metadata() {
    let schema = ProfileFormSchema::schema();

    let address = schema.field("address").expect("address entry");
    assert!(address.is_nested());
    assert!(!address.is_list());
    assert_eq!(
        address.component,
        FormSchemaComponent::Nested {
            form_struct: "Address".to_string(),
        }
    );

    let tags = schema.field("tags").expect("tags entry");
    assert!(tags.is_list());
    assert_eq!(
        tags.component,
        FormSchemaComponent::List {
            item: gpui_form::schema::components::ComponentKind::Input,
            item_struct: None,
            add_label: None,
        }
    );
    // Element validators (`each(...)`) stay element-scoped: the inventory
    // FieldVariant records field-level rule names only, and the export is a
    // read-only projection of that chain — it must not reshape metadata.
    assert!(tags.validations.is_empty());
    // Type strings are canonical Rust spelling (the derive normalizes the
    // proc-macro printer's token spacing before storing metadata).
    assert_eq!(tags.value_type, "Vec<String>");
}

#[test]
fn schema_includes_layout_hints_and_value_shapes() {
    let schema = ProfileFormSchema::schema();
    let email = schema.field("email").expect("email entry");

    assert_eq!(email.layout.section.as_deref(), Some("Contact"));
    assert_eq!(email.layout.label.as_deref(), Some("Email address"));
    assert_eq!(
        email.layout.width,
        gpui_form::schema::layout::LayoutWidth::Half
    );
    assert!(email.layout.description.is_none());

    assert_eq!(email.value_type, "String");
    assert_eq!(email.source_value_type, "String");
    assert!(email.optional, "an Option<T> source field is optional");
    assert!(
        email.wraps_in_option,
        "input components wrap the holder value in Option"
    );
}

#[test]
fn schema_round_trips_through_serde_json() {
    let schema = ProfileFormSchema::schema();

    let json = serde_json::to_string_pretty(&schema).expect("serialize FormSchema");
    let restored: FormSchema = serde_json::from_str(&json).expect("deserialize FormSchema");
    assert_eq!(restored, schema, "exported schema must round-trip");

    // The M3/M4 metadata survives the wire.
    let email = restored.field("email").expect("email entry");
    assert_eq!(email.required_if.as_deref(), Some("subscribe"));
    assert_eq!(email.depends_on.as_deref(), Some("subscribe"));
    assert!(
        restored
            .field("address")
            .expect("address entry")
            .is_nested()
    );
    assert!(restored.field("tags").expect("tags entry").is_list());
    assert_eq!(restored.form, "Profile");
    assert!(restored.has_skipped_fields);
}

#[test]
fn empty_form_exports_an_empty_field_list() {
    let schema = BlankFormSchema::schema();

    assert_eq!(schema.form, "Blank");
    assert!(schema.fields.is_empty());
    assert_eq!(schema.field_names().collect::<Vec<_>>(), Vec::<&str>::new());
    assert!(!schema.has_validations());

    let json = serde_json::to_string(&schema).expect("serialize empty schema");
    let restored: FormSchema = serde_json::from_str(&json).expect("deserialize empty schema");
    assert_eq!(restored, schema);
}

#[test]
fn non_koruma_forms_do_not_export_required_validation_metadata() {
    // The RequiredValidation metadata rides the same koruma gate as the
    // mirrored runtime attr: a form whose holder never derives Koruma never
    // enforces required, so its exported validations must not record it
    // either (plain forms keep exporting, without the phantom rule).
    let schema = PlainFormSchema::schema();

    assert!(!schema.koruma_enabled, "the form has no koruma opt-in");
    assert_eq!(
        schema.field_names().collect::<Vec<_>>(),
        vec!["name", "nickname"]
    );
    for field in schema.fields.iter() {
        assert!(
            !field
                .validations
                .contains(&"RequiredValidation".to_string()),
            "field `{}` must not export RequiredValidation on a non-koruma form: {:?}",
            field.name,
            field.validations
        );
    }
    assert!(
        !schema.has_validations(),
        "no field of this form carries any (enforceable) validation rule"
    );

    // Koruma forms are unchanged: the same projection still records the
    // required-adjacent rules their runtime enforces.
    let profile = ProfileFormSchema::schema();
    let email = profile.field("email").expect("email entry");
    assert!(
        email
            .validations
            .contains(&"RequiredIfValidation".to_string())
    );
    assert!(
        email
            .validations
            .contains(&"NonEmptyValidation".to_string())
    );
}

#[test]
fn nested_form_exports_its_own_schema() {
    // The nested form's own export exists independently — the outer export
    // references it by struct name, not by embedding.
    let schema = AddressFormSchema::schema();
    assert_eq!(schema.form, "Address");
    assert_eq!(
        schema.field_names().collect::<Vec<_>>(),
        vec!["city", "zip"]
    );
}
