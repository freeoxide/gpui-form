//! Documentation generation over the schema export (backlog #14, M5).
//!
//! [`schema_markdown`] renders a [`FormSchema`] — the document
//! `<Name>FormSchema::schema()` returns — into a Markdown reference table.
//! This is the prototyping side of "optionally consume the export for
//! docs/fixture generation": the same owned document that serializes to JSON
//! for test fixtures also drives human-readable form references, without a
//! second metadata model (the export is a read-only projection of the
//! inventory `FieldVariant`/`FieldLayout` chain).

use gpui_form_schema::registry::{FormSchema, FormSchemaComponent};

/// Render an exported form schema as a Markdown reference document.
///
/// Deterministic: field rows follow declaration order, and absent hints are
/// omitted rather than rendered as empty placeholders.
pub fn schema_markdown(schema: &FormSchema) -> String {
    let mut out = String::new();

    out.push_str(&format!("# `{}` form\n\n", schema.form));

    out.push_str(&format!(
        "- Source: `{}`\n",
        if schema.source_path.is_empty() {
            "-"
        } else {
            &schema.source_path
        }
    ));
    out.push_str(&format!(
        "- Koruma validation: {}\n",
        if schema.koruma_enabled {
            "enabled"
        } else {
            "disabled"
        }
    ));
    out.push_str(&format!(
        "- Skipped fields: {}\n",
        if schema.has_skipped_fields {
            "yes"
        } else {
            "no"
        }
    ));
    out.push('\n');

    if schema.fields.is_empty() {
        out.push_str("This form has no fields.\n");
        return out;
    }

    out.push_str(
        "| Field | Component | Type | Optional | Validations | Depends on | Required if |\n",
    );
    out.push_str("|---|---|---|---|---|---|---|\n");
    for field in &schema.fields {
        out.push_str(&format!(
            "| `{}` | {} | `{}` | {} | {} | {} | {} |\n",
            field.name,
            component_label(&field.component),
            field.value_type,
            yes_no(field.optional),
            if field.validations.is_empty() {
                "-".to_string()
            } else {
                field.validations.join(", ")
            },
            field.depends_on.as_deref().unwrap_or("-"),
            field.required_if.as_deref().unwrap_or("-"),
        ));
    }
    out.push('\n');

    // Detail notes: layout hints and composed shapes that do not fit the
    // table columns.
    for field in &schema.fields {
        let mut notes: Vec<String> = Vec::new();
        if let Some(section) = field.layout.section.as_deref() {
            notes.push(format!("section: `{section}`"));
        }
        if let Some(label) = field.layout.label.as_deref() {
            notes.push(format!("label: `{label}`"));
        }
        if let Some(description) = field.layout.description.as_deref() {
            notes.push(format!("description: `{description}`"));
        }
        if let Some(placeholder) = field.layout.placeholder.as_deref() {
            notes.push(format!("placeholder: `{placeholder}`"));
        }
        if field.layout.width != gpui_form_schema::layout::LayoutWidth::Full {
            notes.push(format!("width: `{}`", field.layout.width));
        }
        if let Some(validator) = field.async_validator.as_deref() {
            notes.push(format!("async validator: `{validator}`"));
        }
        if let Some(default) = field.default_expr.as_deref() {
            notes.push(format!("default: `{default}`"));
        }
        if let Some(custom) = field.custom_component.as_deref() {
            notes.push(format!("custom component: `{custom}`"));
        }
        if !notes.is_empty() {
            out.push_str(&format!("- `{}` — {}\n", field.name, notes.join("; ")));
        }
    }

    out
}

fn component_label(component: &FormSchemaComponent) -> String {
    match component {
        FormSchemaComponent::Nested { form_struct } => format!("nested (`{form_struct}`)"),
        FormSchemaComponent::List {
            item,
            item_struct,
            add_label,
        } => {
            let mut label = format!("list of `{item}`");
            if let Some(item_struct) = item_struct.as_deref() {
                label.push_str(&format!(" (`{item_struct}`)"));
            }
            if let Some(add_label) = add_label.as_deref() {
                label.push_str(&format!(", add label: `{add_label}`"));
            }
            label
        },
        other => other.kind().component_name().to_string(),
    }
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_form_schema::components::{ComponentKind, NumberInputKind};
    use gpui_form_schema::layout::LayoutWidth;
    use gpui_form_schema::registry::{FormSchemaComponent, FormSchemaField, FormSchemaLayout};

    fn fixture() -> FormSchema {
        FormSchema {
            form: "Profile".to_string(),
            source_path: "src/profile.rs".to_string(),
            koruma_enabled: true,
            has_skipped_fields: false,
            fields: vec![
                FormSchemaField {
                    name: "email".to_string(),
                    component: FormSchemaComponent::Input,
                    value_type: "String".to_string(),
                    source_value_type: "String".to_string(),
                    optional: true,
                    wraps_in_option: true,
                    validations: vec!["NonEmptyValidation".to_string()],
                    default_expr: None,
                    from_expr: None,
                    into_expr: None,
                    custom_shape: None,
                    custom_component: None,
                    custom_value_binding: false,
                    layout: FormSchemaLayout {
                        section: Some("Contact".to_string()),
                        label: Some("Email address".to_string()),
                        description: None,
                        placeholder: Some("you@example.com".to_string()),
                        width: LayoutWidth::Half,
                    },
                    async_validator: Some("checks::email_taken".to_string()),
                    depends_on: Some("subscribe".to_string()),
                    required_if: Some("subscribe".to_string()),
                },
                FormSchemaField {
                    name: "tags".to_string(),
                    component: FormSchemaComponent::List {
                        item: ComponentKind::Input,
                        item_struct: None,
                        add_label: Some("Add tag".to_string()),
                    },
                    value_type: "Vec<String>".to_string(),
                    source_value_type: "Vec<String>".to_string(),
                    optional: false,
                    wraps_in_option: false,
                    validations: Vec::new(),
                    default_expr: None,
                    from_expr: None,
                    into_expr: None,
                    custom_shape: None,
                    custom_component: None,
                    custom_value_binding: false,
                    layout: FormSchemaLayout::default(),
                    async_validator: None,
                    depends_on: None,
                    required_if: None,
                },
                FormSchemaField {
                    name: "age".to_string(),
                    component: FormSchemaComponent::NumberInput {
                        validation_type: Some("f64".to_string()),
                        kind: NumberInputKind::Float,
                    },
                    value_type: "u32".to_string(),
                    source_value_type: "u32".to_string(),
                    optional: false,
                    wraps_in_option: true,
                    validations: Vec::new(),
                    default_expr: None,
                    from_expr: None,
                    into_expr: None,
                    custom_shape: None,
                    custom_component: None,
                    custom_value_binding: false,
                    layout: FormSchemaLayout::default(),
                    async_validator: None,
                    depends_on: None,
                    required_if: None,
                },
            ],
        }
    }

    #[test]
    fn markdown_documents_fields_layout_and_dependencies() {
        let markdown = schema_markdown(&fixture());

        assert!(markdown.contains("# `Profile` form"), "{markdown}");
        assert!(
            markdown.contains("- Source: `src/profile.rs`"),
            "{markdown}"
        );
        assert!(
            markdown.contains("- Koruma validation: enabled"),
            "{markdown}"
        );

        // Field rows follow declaration order with dependency columns.
        assert!(
            markdown.contains(
                "| `email` | input | `String` | yes | NonEmptyValidation | subscribe | subscribe |"
            ),
            "{markdown}"
        );
        assert!(
            markdown.contains("| `tags` | list of `input`, add label: `Add tag` |"),
            "{markdown}"
        );

        // Layout + async validator notes render per field.
        assert!(
            markdown.contains(
                "- `email` — section: `Contact`; label: `Email address`; placeholder: `you@example.com`; width: `half`; async validator: `checks::email_taken`"
            ),
            "{markdown}"
        );
    }

    #[test]
    fn markdown_for_an_empty_schema_states_it() {
        let empty = FormSchema {
            form: "Blank".to_string(),
            source_path: "src/blank.rs".to_string(),
            koruma_enabled: false,
            has_skipped_fields: false,
            fields: Vec::new(),
        };
        let markdown = schema_markdown(&empty);
        assert!(markdown.contains("This form has no fields."), "{markdown}");
        assert!(
            markdown.contains("- Koruma validation: disabled"),
            "{markdown}"
        );
    }
}
