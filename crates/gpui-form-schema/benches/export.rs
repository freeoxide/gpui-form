//! Criterion benches for the schema export projection (`FormSchema`).
//!
//! Round 0 of the performance campaign: measurement only. The projection
//! (`From<&GpuiFormShape> for FormSchema`) is what `<Name>FormSchema::schema()`
//! returns and what prototyping/docs tooling consumes; benches cover the
//! projection itself, the JSON wire document, and path-style lookups.

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use gpui_form_schema::components::{
    ComponentKind, ComponentsBehaviour, InfiniteSelectBehaviour, ListBehaviour, NestedBehaviour,
    NumberInputBehaviour, NumberInputKind, PhoneInputBehaviour, SelectBehaviour,
};
use gpui_form_schema::layout::{FieldLayout, LayoutWidth};
use gpui_form_schema::registry::{FieldVariant, FormSchema, GpuiFormShape};

/// A representative 12-field shape: every shipped component kind, layout
/// hints, validation-rule ids, and M4 dependency metadata — the metadata
/// spread a real `#[derive(GpuiForm)]` inventory entry carries.
const FIELDS: [FieldVariant; 12] = [
    FieldVariant::new("username", "String", false, ComponentsBehaviour::Input).with_layout(
        FieldLayout::new()
            .with_section(Some("Account"))
            .with_label(Some("Username"))
            .with_description(Some("Your unique handle"))
            .with_placeholder(Some("ada"))
            .with_width(LayoutWidth::Half),
    ),
    FieldVariant::new("email", "String", true, ComponentsBehaviour::Input)
        .with_validations(&["EmailValidation"])
        .with_layout(
            FieldLayout::new()
                .with_section(Some("Account"))
                .with_label(Some("Email"))
                .with_width(LayoutWidth::Half),
        ),
    FieldVariant::new(
        "age",
        "u32",
        true,
        ComponentsBehaviour::NumberInput(NumberInputBehaviour {
            validation_type: None,
            kind: NumberInputKind::UnsignedInteger,
        }),
    )
    .with_validations(&["RangeValidation"]),
    FieldVariant::new(
        "balance",
        "f64",
        true,
        ComponentsBehaviour::NumberInput(NumberInputBehaviour {
            validation_type: Some("f64"),
            kind: NumberInputKind::Float,
        }),
    ),
    FieldVariant::new(
        "phone",
        "String",
        true,
        ComponentsBehaviour::PhoneInput(PhoneInputBehaviour {
            country_field: Some("country"),
        }),
    ),
    FieldVariant::new(
        "country",
        "Country",
        true,
        ComponentsBehaviour::Select(SelectBehaviour {
            searchable: true,
            partial: false,
        }),
    ),
    FieldVariant::new(
        "region",
        "Region",
        true,
        ComponentsBehaviour::InfiniteSelect(InfiniteSelectBehaviour {
            searchable: false,
            max_depth: Some(3),
        }),
    )
    .with_depends_on(Some("country")),
    FieldVariant::new("newsletter", "bool", false, ComponentsBehaviour::Checkbox),
    FieldVariant::new("notifications", "bool", false, ComponentsBehaviour::Switch),
    FieldVariant::new(
        "address",
        "Address",
        false,
        ComponentsBehaviour::Nested(NestedBehaviour {
            form_struct: "Address",
        }),
    )
    .with_validations(&["NestedValidation"]),
    FieldVariant::new(
        "tags",
        "String",
        false,
        ComponentsBehaviour::List(ListBehaviour {
            item: ComponentKind::Input,
            item_struct: None,
            add_label: Some("Add tag"),
        }),
    )
    .with_validations(&["LenValidation"]),
    FieldVariant::new(
        "attachment",
        "PathBuf",
        true,
        ComponentsBehaviour::FilePicker,
    ),
];

fn bench_export_projection(c: &mut Criterion) {
    const SHAPE: GpuiFormShape =
        GpuiFormShape::new("ExportBenchProfile", &FIELDS, "src/profile.rs", true)
            .with_skipped_fields(true);

    c.bench_function("schema/export_projection/12_field_shape", |b| {
        b.iter(|| FormSchema::from(black_box(&SHAPE)))
    });

    c.bench_function("schema/export_json_wire/12_field_shape", |b| {
        b.iter(|| serde_json::to_string(&FormSchema::from(black_box(&SHAPE))))
    });

    // Docs/prototyping consumers look fields up by name in a loop; measure
    // one hit and one miss per iteration.
    let schema = FormSchema::from(&SHAPE);
    c.bench_function("schema/export_field_lookup/hit_and_miss", |b| {
        b.iter(|| {
            let hit = black_box(&schema).field("region").is_some();
            let miss = black_box(&schema).field("missing").is_none();
            hit && miss
        })
    });
}

criterion_group!(benches, bench_export_projection);
criterion_main!(benches);
