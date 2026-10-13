//! Facade re-export compile proof for the M2 headless surface and the M5
//! schema-export document: the new types must be reachable through
//! `gpui_form::` paths (the facade is the public boundary — generated code
//! and apps reference these names).

use gpui_form::{
    AsyncFieldValidation, BoxError, FieldPath, FormSchema, SubmitAction, SubmitTracker,
    ValidationCache,
};

#[derive(Clone, Debug, PartialEq, Default)]
struct Holder {
    name: String,
}

struct NoopValidation;

impl AsyncFieldValidation<Holder> for NoopValidation {
    type Error = &'static str;

    async fn validate_field(
        &self,
        _holder: &Holder,
        _field: &FieldPath,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

struct NoopAction;

impl SubmitAction<Holder> for NoopAction {
    type Output = ();
    type Error = &'static str;

    async fn submit(&mut self, _holder: &Holder) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[test]
fn m2_headless_types_re_export_through_the_facade() {
    let mut tracker: SubmitTracker<&'static str> = SubmitTracker::new();
    assert!(tracker.begin());
    tracker.finish(Ok(()));
    assert!(tracker.is_submit_successful());
    assert_eq!(tracker.submit_count(), 1);

    let boxed: SubmitTracker<BoxError> = SubmitTracker::default();
    assert!(boxed.can_submit());

    let mut cache: ValidationCache<Holder, bool> = ValidationCache::new();
    cache.store(Holder::default(), true);
    assert_eq!(cache.get(&Holder::default()), Some(&true));
}

#[test]
fn contract_traits_are_implementable_through_facade_imports() {
    // Compiling this at all is the proof: the trait objects are nameable and
    // implementable using only facade imports. Driving the futures is core's
    // test surface (gpui-form-core/src/submit.rs, async_validation.rs).
    fn asserts<V: AsyncFieldValidation<Holder>, A: SubmitAction<Holder>>() {
        let _ = std::mem::size_of::<V>();
        let _ = std::mem::size_of::<A>();
    }
    asserts::<NoopValidation, NoopAction>();

    // The futures are constructible (polling them is exercised in core).
    let holder = Holder::default();
    let path = FieldPath::new(&["name"]);
    let future = NoopValidation.validate_field(&holder, &path);
    drop(future);
    let mut action = NoopAction;
    let future = action.submit(&holder);
    drop(future);
}

#[test]
fn schema_export_document_round_trips_through_the_facade() {
    // The wire format the derive emits for `<Name>FormSchema::schema()`:
    // snake_case component tags, absent hints omitted, defaults filled back
    // on deserialization. Round-tripping through the facade re-export with
    // serde_json proves the document is nameable and serializable without
    // touching the schema crate directly.
    let json = r#"{
        "form": "Demo",
        "source_path": "src/demo.rs",
        "fields": [
            {
                "name": "email",
                "component": {"component": "input"},
                "value_type": "String",
                "source_value_type": "String",
                "optional": false,
                "wraps_in_option": true
            },
            {
                "name": "tags",
                "component": {"component": "list", "item": "input"},
                "value_type": "Vec<String>",
                "source_value_type": "Vec<String>",
                "optional": false,
                "wraps_in_option": false,
                "depends_on": "email"
            }
        ]
    }"#;

    let schema: FormSchema = serde_json::from_str(json).expect("deserialize through facade");
    assert_eq!(schema.form, "Demo");
    assert!(!schema.koruma_enabled, "absent flags default to false");
    assert_eq!(
        schema.field_names().collect::<Vec<_>>(),
        vec!["email", "tags"]
    );
    let tags = schema.field("tags").expect("tags entry");
    assert!(tags.is_list(), "list component tag deserializes");
    assert_eq!(tags.depends_on.as_deref(), Some("email"));

    let re = serde_json::to_string(&schema).expect("serialize through facade");
    let round: FormSchema = serde_json::from_str(&re).expect("round-trip");
    assert_eq!(round, schema);
}
