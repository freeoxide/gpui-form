use gpui_form::PathSegment;
use gpui_form::core::FormFieldErrors;
use gpui_form_derive::GpuiForm;
use koruma_collection::{
    collection::{LenValidation, NonEmptyValidation},
    numeric::RangeValidation,
};

#[derive(GpuiForm, koruma::Koruma)]
struct TagsForm {
    #[gpui_form(component(list(item = input)))]
    #[koruma(each(LenValidation::<_>::builder().min(2).max(20)))]
    tags: Vec<String>,
}

#[derive(GpuiForm, koruma::Koruma)]
struct ScoresForm {
    #[gpui_form(component(input))]
    #[koruma(NonEmptyValidation::<_>::builder())]
    label: String,

    #[gpui_form(component(list(item = number_input)))]
    #[koruma(each(RangeValidation::<_>::builder().min(0).max(10)))]
    scores: Vec<u32>,
}

#[derive(GpuiForm, koruma::Koruma)]
struct InventoryForm {
    #[gpui_form(component(list(item = input)))]
    #[koruma(
        koruma_collection::collection::LenValidation::<_>::builder().min(2).max(10),
        each(koruma_collection::collection::NonEmptyValidation::<_>::builder())
    )]
    skus: Vec<String>,
}

#[derive(GpuiForm, koruma::Koruma)]
#[gpui_form(koruma)]
struct OptionalListForm {
    #[gpui_form(component(list(item = input)))]
    aliases: Option<Vec<String>>,
}

#[test]
fn holder_stores_the_plain_item_vec() {
    let holder = TagsFormFormValueHolder::default();
    let tags: &Vec<String> = &holder.tags;
    assert!(tags.is_empty());
}

#[test]
fn from_and_into_original_round_trip_the_vec() {
    let original = TagsForm {
        tags: vec!["rust".to_string(), "gpui".to_string()],
    };
    let holder = TagsFormFormValueHolder::from(original);
    assert_eq!(holder.tags_items(), &["rust", "gpui"]);

    let round: TagsForm = holder.into();
    assert_eq!(round.tags, vec!["rust".to_string(), "gpui".to_string()]);
}

#[test]
fn add_remove_and_items_mutation_api() {
    let mut holder = TagsFormFormValueHolder::default();
    assert!(holder.tags_items().is_empty());

    holder.add_tags("alpha".to_string());
    holder.add_tags("beta".to_string());
    holder.add_tags("gamma".to_string());
    assert_eq!(holder.tags_items().len(), 3);

    holder.remove_tags(1);
    assert_eq!(holder.tags_items(), &["alpha", "gamma"]);

    holder.remove_tags(0);
    holder.remove_tags(0);
    assert!(holder.tags_items().is_empty());
}

#[test]
fn optional_list_mutation_tolerates_absent() {
    let mut holder = OptionalListFormFormValueHolder::default();
    assert!(holder.aliases.is_none());
    assert!(holder.aliases_items().is_empty());

    holder.remove_aliases(0);
    assert!(holder.aliases.is_none());

    holder.add_aliases("ada".to_string());
    assert_eq!(holder.aliases_items(), &["ada"]);

    holder.remove_aliases(0);
    assert!(holder.aliases_items().is_empty());

    let original = OptionalListForm {
        aliases: Some(vec!["x".to_string()]),
    };
    let holder = OptionalListFormFormValueHolder::from(original);
    assert_eq!(holder.aliases_items(), &["x"]);
}

#[test]
fn element_validator_fires_with_index_in_error() {
    let holder = TagsFormFormValueHolder {
        tags: vec![
            "ok".to_string(),
            "x".to_string(),
            "also-ok".to_string(),
            "y".to_string(),
        ],
    };
    assert!(holder.validate().is_err());

    let errors = holder.validate_fields();
    assert!(errors.has_errors());
    let wrapper = errors.tags().expect("the list slot holds the wrapper");

    let failing_indices: Vec<usize> = wrapper.element_errors().iter().map(|(i, _)| *i).collect();
    assert_eq!(failing_indices, vec![1, 3]);

    assert!(errors.tags_element_error(0).is_none());
    assert!(errors.tags_element_error(2).is_none());
    let element = errors
        .tags_element_error(1)
        .expect("item 1 failed its each(..) validators");
    assert!(element.len_validation().is_some());
}

#[test]
fn error_for_routes_root_and_typed_item_index() {
    let holder = TagsFormFormValueHolder {
        tags: vec!["ok".to_string(), "x".to_string()],
    };
    let errors = holder.validate_fields();

    let root = errors
        .error_for(&TagsFormFormPath::tags_root())
        .expect("the root path routes to the wrapper slot");
    assert_eq!(root.path().to_string(), "tags");
    assert_eq!(root.issue_keys(), vec!["validation.len"]);

    assert!(errors.error_for(&TagsFormFormPath::tags(0)).is_none());
    let item = errors
        .error_for(&TagsFormFormPath::tags(1))
        .expect("item 1 failed");
    assert_eq!(item.path().to_string(), "tags");
    assert!(errors.error_for(&TagsFormFormPath::tags(99)).is_none());

    let valid = TagsFormFormValueHolder {
        tags: vec!["ok".to_string()],
    };
    let errors = valid.validate_fields();
    assert!(errors.error_for(&TagsFormFormPath::tags_root()).is_none());
    assert!(errors.error_for(&TagsFormFormPath::tags(0)).is_none());
}

#[test]
fn failing_paths_expand_per_failing_item_index() {
    let holder = ScoresFormFormValueHolder {
        label: Some("demo".to_string()),
        scores: vec![5, 11, 7],
    };
    let errors = holder.validate_fields();
    fn generic_paths<E: FormFieldErrors>(errors: &E) -> Vec<String> {
        errors
            .failing_paths()
            .iter()
            .map(|path| path.to_string())
            .collect()
    }
    assert_eq!(generic_paths(&errors), vec!["scores.1", "scores"]);
}

#[test]
fn combined_element_and_whole_vec_failure_reports_root_and_items() {
    let holder = InventoryFormFormValueHolder {
        skus: vec![String::new()],
    };
    let errors = holder.validate_fields();

    fn generic_paths<E: FormFieldErrors>(errors: &E) -> Vec<String> {
        errors
            .failing_paths()
            .iter()
            .map(|path| path.to_string())
            .collect()
    }
    assert_eq!(
        generic_paths(&errors),
        vec!["skus.0", "skus"],
        "root AND item paths in push order (items first, then root)"
    );

    let failing = errors.failing_fields().collect::<Vec<_>>();
    assert_eq!(failing.len(), 2, "item + root");
    assert_eq!(failing[0].0.to_string(), "skus.0");
    assert_eq!(failing[1].0.to_string(), "skus");
    assert_eq!(
        failing[0].1.issue_keys(),
        vec!["validation.len", "validation.non_empty"]
    );
    assert_eq!(
        failing[1].1.issue_keys(),
        vec!["validation.len", "validation.non_empty"],
        "the root failure carries the whole-Vec key plus the element key"
    );

    let root = errors
        .error_for(&InventoryFormFormPath::skus_root())
        .expect("the root path routes to the populated whole-field slot");
    assert_eq!(root.path().to_string(), "skus");
    assert!(errors.error_for(&InventoryFormFormPath::skus(0)).is_some());
    assert!(errors.error_for(&InventoryFormFormPath::skus(5)).is_none());

    let elements_only = InventoryFormFormValueHolder {
        skus: vec!["ok".to_string(), String::new()],
    };
    let errors = elements_only.validate_fields();
    assert_eq!(generic_paths(&errors), vec!["skus.1", "skus"]);
}

#[test]
fn validate_fields_agrees_with_raw_validate_for_lists() {
    let failing = ScoresFormFormValueHolder {
        label: Some("demo".to_string()),
        scores: vec![42],
    };
    assert_eq!(
        failing.validate_fields().is_empty(),
        failing.validate().is_ok()
    );

    let valid = ScoresFormFormValueHolder {
        label: Some("demo".to_string()),
        scores: vec![0, 10],
    };
    assert_eq!(valid.validate_fields().is_empty(), valid.validate().is_ok());
}

#[test]
fn conversion_error_reports_index_when_present() {
    let error = TagsFormFormValueHolderConversionError {
        field_name: "tags",
        index: Some(2),
    };
    assert!(error.to_string().contains("item 2"));
    assert!(error.to_string().contains("tags"));

    let whole_field = TagsFormFormValueHolderConversionError {
        field_name: "tags",
        index: None,
    };
    assert!(whole_field.to_string().contains("Failed to unwrap"));
}

#[test]
fn path_segments_carry_typed_indices() {
    let path = TagsFormFormPath::tags(2);
    assert_eq!(
        path.path().segments(),
        &[PathSegment::Name("tags"), PathSegment::Index(2)]
    );
    assert_eq!(path.to_string(), "tags.2");

    assert!(path.field_index().is_none());
    assert_eq!(
        TagsFormFormPath::tags_root().field_index(),
        Some(0),
        "roots keep the dense touched/dirty index"
    );
}
