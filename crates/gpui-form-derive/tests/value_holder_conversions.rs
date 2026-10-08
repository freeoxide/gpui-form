//! Runtime behavior of the generated `...FormValueHolder` conversion surface:
//! `type`/`from`/`into` overrides, skipped-field `into_original`, the
//! default-expression comparison, and `present_fields_json`.

use gpui_form_derive::GpuiForm;

#[derive(Debug, GpuiForm)]
struct OverrideForm {
    #[gpui_form(
        type = String,
        from = |raw: u32| format!("id-{raw}"),
        into = |text: String| text.trim_start_matches("id-").parse::<u32>().unwrap(),
        component(input)
    )]
    count: u32,

    #[gpui_form(skip)]
    #[allow(dead_code)] // skipped fields are intentionally absent from the holder
    audit: bool,
}

#[derive(GpuiForm)]
struct PresentJsonForm {
    #[gpui_form(
        type = String,
        into = |text: String| format!("len:{}", text.len()),
        component(input)
    )]
    note: String,

    #[gpui_form(skip)]
    #[allow(dead_code)] // skipped fields are intentionally absent from the holder
    internal: u32,
}

#[derive(GpuiForm)]
struct DefaultEmailForm {
    #[gpui_form(component(input), default = "test@example.com")]
    email: String,
}

#[derive(Debug, GpuiForm)]
struct SkippedDefaultForm {
    #[gpui_form(component(input), default = "test@example.com")]
    email: String,

    #[gpui_form(skip)]
    #[allow(dead_code)] // skipped fields are intentionally absent from the holder
    audit: bool,
}

#[test]
fn override_conversions_apply_in_both_directions() {
    let holder = OverrideFormFormValueHolder::from(OverrideForm {
        count: 7,
        audit: true,
    });
    assert_eq!(holder.count.as_deref(), Some("id-7"));

    let original = holder.into_original(false).expect("count is present");
    assert_eq!(original.count, 7);
    assert!(!original.audit, "skipped field comes from the parameter");
}

#[test]
fn into_original_reports_missing_wrapped_value() {
    let err = OverrideFormFormValueHolder::default()
        .into_original(true)
        .expect_err("absent count must fail conversion");
    assert_eq!(err.field_name, "count");
}

#[test]
fn present_fields_json_applies_into_conversion() {
    let mut holder = PresentJsonFormFormValueHolder::default();
    assert_eq!(holder.present_fields_json(), "{}");

    holder.note = Some("abcd".to_string());
    let json = holder.present_fields_json();
    assert!(
        json.contains("\"note\"") && json.contains("len:4"),
        "converted debug value must appear for a present field: {json}"
    );
}

#[test]
fn string_default_lands_in_holder_default() {
    let holder = DefaultEmailFormFormValueHolder::default();
    assert_eq!(holder.email.as_deref(), Some("test@example.com"));
}

#[test]
fn absent_wrapped_value_restores_the_default() {
    let original = DefaultEmailForm::from(DefaultEmailFormFormValueHolder::default());
    assert_eq!(original.email, "test@example.com");
}

#[test]
fn skipped_form_default_comparison_maps_default_to_none() {
    let at_default = SkippedDefaultFormFormValueHolder::from(SkippedDefaultForm {
        email: "test@example.com".to_string(),
        audit: true,
    });
    assert!(
        at_default.email.is_none(),
        "source value equal to the default must map to None"
    );

    let altered = SkippedDefaultFormFormValueHolder::from(SkippedDefaultForm {
        email: "other@example.com".to_string(),
        audit: true,
    });
    assert_eq!(altered.email.as_deref(), Some("other@example.com"));

    let err = at_default
        .into_original(true)
        .expect_err("into_original is strict even when a default exists");
    assert_eq!(err.field_name, "email");
}
