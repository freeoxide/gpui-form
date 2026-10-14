use crate::components::ComponentsBehaviour;
use crate::layout::FieldLayout;
use heck::{ToKebabCase as _, ToPascalCase as _};

inventory::collect!(GpuiFormShape);

#[derive(Debug)]
pub struct GpuiFormShape {
    pub struct_name: &'static str,
    pub components: &'static [FieldVariant],
    /// Source file path where the struct is declared (`file!()`), used to
    /// generate imports.
    pub source_path: &'static str,
    /// Whether the struct has koruma validation enabled at the struct level.
    pub koruma_enabled: bool,
    /// Whether the struct contains any `#[gpui_form(skip)]` fields; when true,
    /// `FormValueHolder` cannot convert back without skipped-field values.
    pub has_skipped_fields: bool,
}

impl GpuiFormShape {
    pub const fn new(
        struct_name: &'static str,
        components: &'static [FieldVariant],
        source_path: &'static str,
        koruma_enabled: bool,
    ) -> Self {
        Self {
            struct_name,
            components,
            source_path,
            koruma_enabled,
            has_skipped_fields: false,
        }
    }

    /// Marks whether the struct has any `#[gpui_form(skip)]` fields.
    pub const fn with_skipped_fields(mut self, has_skipped_fields: bool) -> Self {
        self.has_skipped_fields = has_skipped_fields;
        self
    }

    pub fn has_validations(&self) -> bool {
        self.koruma_enabled
            && self
                .components
                .iter()
                .any(|field| !field.validations.is_empty())
    }

    /// Returns true when koruma validation is enabled.
    pub const fn has_koruma(&self) -> bool {
        self.koruma_enabled
    }

    /// Returns true when at least one source field is marked `#[gpui_form(skip)]`.
    pub const fn has_skipped_fields(&self) -> bool {
        self.has_skipped_fields
    }
}

#[derive(Debug)]
pub struct FieldVariant {
    pub field_name: &'static str,
    /// Full Rust type string for the field's form-side base value type (e.g.
    /// `some_lib::country::Country`), excluding any generated `Option<...>`
    /// wrapper.
    pub value_type: &'static str,
    /// The source model's base value type before any `#[gpui_form(type = ...)]`
    /// override is applied.
    pub source_value_type: &'static str,
    pub optional: bool,
    /// Whether the generated value holder wraps this field in `Option<T>`
    /// because of component behavior. Source `Option<T>` fields are tracked by
    /// [`FieldVariant::optional`].
    pub wraps_in_option: bool,
    pub behaviour: ComponentsBehaviour,
    /// List of validation rule identifiers applied to this field (for diagnostics/rendering).
    pub validations: &'static [&'static str],
    /// Default value expression as a string, if one was specified.
    pub default_expr: Option<&'static str>,
    /// Source-to-form conversion expression, if one was specified.
    pub from_expr: Option<&'static str>,
    /// Form-to-source conversion expression, if one was specified.
    pub into_expr: Option<&'static str>,
    /// For custom components: the shape type path implementing
    /// `gpui_form::custom::CustomComponentShape`.
    pub custom_shape: Option<&'static str>,
    /// For custom components: the UI component type path (e.g. "TagsInput").
    /// Used by the prototyping code generator to emit `Component::new(&entity)`.
    pub custom_component: Option<&'static str>,
    /// Whether the custom component opted into
    /// `gpui_form::custom::CustomComponentValueAdapter` generation.
    pub custom_value_binding: bool,
    /// Non-rendering layout hints; consumers decide how to render. Defaults to
    /// an empty [`FieldLayout`] (all hints absent, width full).
    pub layout: FieldLayout,
    /// Async validator path from `#[gpui_form(validate(async_fn = ...))]`.
    /// Metadata-only: the path is recorded but nothing executes it yet.
    pub async_validator: Option<&'static str>,
    /// The sibling field this field depends on
    /// (`#[gpui_form(depends_on = <field>)]`), stored as the sibling's field
    /// name; wiring stays consumer-authored.
    pub depends_on: Option<&'static str>,
    /// The controlling sibling for `#[gpui_form(required_if = <field>)]`.
    /// Unlike `depends_on`, the derive also lowers it to a koruma validator,
    /// so the rule is enforced, not just recorded for tooling.
    pub required_if: Option<&'static str>,
}

impl FieldVariant {
    pub const fn new(
        field_name: &'static str,
        value_type: &'static str,
        optional: bool,
        behaviour: ComponentsBehaviour,
    ) -> Self {
        Self {
            field_name,
            value_type,
            source_value_type: value_type,
            optional,
            wraps_in_option: behaviour.kind().default_wraps_in_option(),
            behaviour,
            validations: &[],
            default_expr: None,
            from_expr: None,
            into_expr: None,
            custom_shape: None,
            custom_component: None,
            custom_value_binding: false,
            layout: FieldLayout::new(),
            async_validator: None,
            depends_on: None,
            required_if: None,
        }
    }

    /// Attach the source model value type when it differs from the form-side value type.
    pub const fn with_source_value_type(mut self, source_value_type: &'static str) -> Self {
        self.source_value_type = source_value_type;
        self
    }

    /// Attach the component-driven generated value-holder wrapping policy.
    pub const fn with_wraps_in_option(mut self, wraps_in_option: bool) -> Self {
        self.wraps_in_option = wraps_in_option;
        self
    }

    /// Attach optional source/form conversion expressions.
    pub const fn with_conversions(
        mut self,
        from_expr: Option<&'static str>,
        into_expr: Option<&'static str>,
    ) -> Self {
        self.from_expr = from_expr;
        self.into_expr = into_expr;
        self
    }

    /// Attach a default value expression to this field metadata.
    pub const fn with_default(mut self, default_expr: &'static str) -> Self {
        self.default_expr = Some(default_expr);
        self
    }

    /// Attach a custom UI component path to this field metadata.
    pub const fn with_custom_component(mut self, component: &'static str) -> Self {
        self.custom_component = Some(component);
        self
    }

    /// Attach an optional custom component path, e.g. from the shape's
    /// `COMPONENT_PATH` constant rather than an explicit attribute value.
    pub const fn with_custom_component_opt(mut self, component: Option<&'static str>) -> Self {
        self.custom_component = component;
        self
    }

    /// Attach the custom component shape type path.
    pub const fn with_custom_shape(mut self, shape: &'static str) -> Self {
        self.custom_shape = Some(shape);
        self
    }

    /// Marks this custom component as value-bound for generated prototyping code.
    pub const fn with_custom_value_binding(mut self, enabled: bool) -> Self {
        self.custom_value_binding = enabled;
        self
    }

    /// Attach non-rendering layout hints; see [`crate::layout::FieldLayout`].
    pub const fn with_layout(mut self, layout: FieldLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Record the async validator path from `validate(async_fn = ...)`. Stays
    /// `const`-callable with a `&'static str` payload (the inventory contract);
    /// `None` clears it.
    pub const fn with_async_validator(mut self, path: Option<&'static str>) -> Self {
        self.async_validator = path;
        self
    }

    /// The async validator path recorded for this field, if any.
    pub const fn async_validator(&self) -> Option<&'static str> {
        self.async_validator
    }

    /// Record the sibling field from `#[gpui_form(depends_on = <field>)]`.
    /// Stays `const`-callable with a `&'static str` payload (the inventory
    /// contract); `None` clears it.
    pub const fn with_depends_on(mut self, sibling: Option<&'static str>) -> Self {
        self.depends_on = sibling;
        self
    }

    /// The controlling sibling field for `depends_on`, if any: the sibling's
    /// field name, not a free-form string.
    pub const fn depends_on(&self) -> Option<&'static str> {
        self.depends_on
    }

    /// Record the sibling field from `#[gpui_form(required_if = <field>)]`;
    /// the derive also lowers it to a koruma validator. Stays `const`-callable
    /// with a `&'static str` payload (the inventory contract); `None` clears it.
    pub const fn with_required_if(mut self, sibling: Option<&'static str>) -> Self {
        self.required_if = sibling;
        self
    }

    /// The controlling sibling field for `required_if`, if any: the sibling's
    /// field name, not a free-form string.
    pub const fn required_if(&self) -> Option<&'static str> {
        self.required_if
    }

    /// Returns true when the generated value holder stores this field as `Option<T>`.
    pub const fn value_holder_wraps_in_option(&self) -> bool {
        self.optional || self.wraps_in_option
    }

    /// This field is a `component(nested)` field whose holder stores the
    /// nested form's own generated value holder.
    pub const fn is_nested(&self) -> bool {
        matches!(self.behaviour, ComponentsBehaviour::Nested(_))
    }

    /// This field is a `component(list(...))` field whose holder stores the
    /// plain item `Vec` plus generated add/remove/items operations.
    pub const fn is_list(&self) -> bool {
        matches!(self.behaviour, ComponentsBehaviour::List(_))
    }

    /// Returns true when generated code should subscribe to this field.
    pub const fn subscribable(&self) -> bool {
        self.behaviour.kind().subscribable() || self.custom_value_binding
    }

    pub fn behaviour_suffix(&self) -> &'static str {
        self.behaviour.component_name()
    }

    pub fn field_name_pascal(&self) -> String {
        self.field_name.to_pascal_case()
    }

    pub fn field_name_with_behaviour(&self) -> String {
        format!("{}_{}", self.field_name, self.behaviour_suffix())
    }

    pub fn kebab_id(&self) -> String {
        self.field_name_with_behaviour().to_kebab_case()
    }

    /// Returns the validation rule identifiers attached to this field.
    pub fn validation_rules(&self) -> &'static [&'static str] {
        self.validations
    }

    /// Attach validation rule identifiers to this field metadata.
    pub const fn with_validations(mut self, validations: &'static [&'static str]) -> Self {
        self.validations = validations;
        self
    }
}

pub use inventory;

/// Owned, serializable mirror of a [`ComponentsBehaviour`] payload for the
/// schema export: one variant per kind with owned `String`s, serialized as an
/// internally-tagged enum on `"component"` with snake_case variant names.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case", tag = "component")]
pub enum FormSchemaComponent {
    Input,
    NumberInput {
        /// Optional `as = ...` override recorded for numeric validation.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        validation_type: Option<String>,
        kind: crate::components::NumberInputKind,
    },
    PhoneInput {
        /// Sibling field whose selected country constrains the phone number
        /// (`phone_input(country = <field>)`), when recorded.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        country_field: Option<String>,
    },
    Checkbox,
    Switch,
    Select {
        searchable: bool,
        partial: bool,
    },
    InfiniteSelect {
        searchable: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max_depth: Option<usize>,
    },
    Custom,
    DatePicker,
    FilePicker,
    /// The field composes another `#[derive(GpuiForm)]` struct.
    Nested {
        /// The nested form struct name (e.g. `"Address"`).
        form_struct: String,
    },
    /// An editable `Vec<T>` field.
    List {
        /// The scalar component kind rendered per item.
        item: crate::components::ComponentKind,
        /// The nested item form struct name; `None` for scalar items (the
        /// only shipped shape in v1 — nested item forms are reserved).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        item_struct: Option<String>,
        /// Optional add-button label override from `list(add_label = "...")`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        add_label: Option<String>,
    },
}

impl FormSchemaComponent {
    /// The component kind of this behaviour, mirroring
    /// [`crate::components::ComponentsBehaviour::kind`].
    pub fn kind(&self) -> crate::components::ComponentKind {
        match self {
            Self::Input => crate::components::ComponentKind::Input,
            Self::NumberInput { .. } => crate::components::ComponentKind::NumberInput,
            Self::PhoneInput { .. } => crate::components::ComponentKind::PhoneInput,
            Self::Checkbox => crate::components::ComponentKind::Checkbox,
            Self::Switch => crate::components::ComponentKind::Switch,
            Self::Select { .. } => crate::components::ComponentKind::Select,
            Self::InfiniteSelect { .. } => crate::components::ComponentKind::InfiniteSelect,
            Self::Custom => crate::components::ComponentKind::Custom,
            Self::DatePicker => crate::components::ComponentKind::DatePicker,
            Self::FilePicker => crate::components::ComponentKind::FilePicker,
            Self::Nested { .. } => crate::components::ComponentKind::Nested,
            Self::List { .. } => crate::components::ComponentKind::List,
        }
    }
}

impl From<&crate::components::ComponentsBehaviour> for FormSchemaComponent {
    fn from(behaviour: &crate::components::ComponentsBehaviour) -> Self {
        use crate::components::ComponentsBehaviour;
        match behaviour {
            ComponentsBehaviour::Input => Self::Input,
            ComponentsBehaviour::NumberInput(options) => Self::NumberInput {
                validation_type: options.validation_type.map(str::to_string),
                kind: options.kind,
            },
            ComponentsBehaviour::PhoneInput(options) => Self::PhoneInput {
                country_field: options.country_field.map(str::to_string),
            },
            ComponentsBehaviour::Checkbox => Self::Checkbox,
            ComponentsBehaviour::Switch => Self::Switch,
            ComponentsBehaviour::Select(options) => Self::Select {
                searchable: options.searchable,
                partial: options.partial,
            },
            ComponentsBehaviour::InfiniteSelect(options) => Self::InfiniteSelect {
                searchable: options.searchable,
                max_depth: options.max_depth,
            },
            ComponentsBehaviour::Custom => Self::Custom,
            ComponentsBehaviour::DatePicker => Self::DatePicker,
            ComponentsBehaviour::FilePicker => Self::FilePicker,
            ComponentsBehaviour::Nested(options) => Self::Nested {
                form_struct: options.form_struct.to_string(),
            },
            ComponentsBehaviour::List(options) => Self::List {
                item: options.item,
                item_struct: options.item_struct.map(str::to_string),
                add_label: options.add_label.map(str::to_string),
            },
        }
    }
}

/// Owned, serializable mirror of a [`FieldLayout`] (schema export).
#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct FormSchemaLayout {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    pub width: crate::layout::LayoutWidth,
}

impl From<&FieldLayout> for FormSchemaLayout {
    fn from(layout: &FieldLayout) -> Self {
        Self {
            section: layout.section.map(str::to_string),
            label: layout.label.map(str::to_string),
            description: layout.description.map(str::to_string),
            placeholder: layout.placeholder.map(str::to_string),
            width: layout.width,
        }
    }
}

/// Owned, serializable export of one [`FieldVariant`] for the schema export: a
/// stable consumer shape for docs, fixtures, prototyping, and non-GPUI tooling
/// — every `&'static str` payload projected to `String`.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct FormSchemaField {
    pub name: String,
    /// The component behaviour, including kind-specific payload details.
    pub component: FormSchemaComponent,
    /// Rust type path for the field's form-side base value type.
    pub value_type: String,
    /// Rust type path for the source model's base value type.
    pub source_value_type: String,
    /// Whether the source field was `Option<T>`.
    pub optional: bool,
    /// Whether the generated value holder wraps this field in `Option<T>`
    /// because of component behavior.
    pub wraps_in_option: bool,
    /// Validation rule identifiers applied to this field.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub validations: Vec<String>,
    /// Default value expression, if specified.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_expr: Option<String>,
    /// Source-to-form conversion expression, if specified.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_expr: Option<String>,
    /// Form-to-source conversion expression, if specified.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub into_expr: Option<String>,
    /// Custom component shape type path, if specified.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_shape: Option<String>,
    /// Custom UI component type path, if specified.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_component: Option<String>,
    /// Whether the custom component opted into value binding generation.
    #[serde(default)]
    pub custom_value_binding: bool,
    /// Non-rendering layout hints.
    #[serde(default)]
    pub layout: FormSchemaLayout,
    /// Async validator path from `validate(async_fn = ...)`; metadata-only,
    /// nothing executes it yet.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub async_validator: Option<String>,
    /// The sibling field this field depends on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depends_on: Option<String>,
    /// The controlling sibling field for the conditional-required rule (also
    /// lowered to a koruma validator).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_if: Option<String>,
}

impl FormSchemaField {
    /// The component kind of this field.
    pub fn kind(&self) -> crate::components::ComponentKind {
        self.component.kind()
    }

    /// This field composes another generated form.
    pub fn is_nested(&self) -> bool {
        matches!(self.component, FormSchemaComponent::Nested { .. })
    }

    /// This field is an editable `Vec<T>` field.
    pub fn is_list(&self) -> bool {
        matches!(self.component, FormSchemaComponent::List { .. })
    }
}

impl From<&FieldVariant> for FormSchemaField {
    fn from(variant: &FieldVariant) -> Self {
        Self {
            name: variant.field_name.to_string(),
            component: FormSchemaComponent::from(&variant.behaviour),
            value_type: variant.value_type.to_string(),
            source_value_type: variant.source_value_type.to_string(),
            optional: variant.optional,
            wraps_in_option: variant.wraps_in_option,
            validations: variant
                .validations
                .iter()
                .map(|rule| rule.to_string())
                .collect::<Vec<_>>(),
            default_expr: variant.default_expr.map(str::to_string),
            from_expr: variant.from_expr.map(str::to_string),
            into_expr: variant.into_expr.map(str::to_string),
            custom_shape: variant.custom_shape.map(str::to_string),
            custom_component: variant.custom_component.map(str::to_string),
            custom_value_binding: variant.custom_value_binding,
            layout: FormSchemaLayout::from(&variant.layout),
            async_validator: variant.async_validator.map(str::to_string),
            depends_on: variant.depends_on.map(str::to_string),
            required_if: variant.required_if.map(str::to_string),
        }
    }
}

/// Owned, serializable export of a [`GpuiFormShape`], generated per form as
/// `<Name>FormSchema::schema()` — a read-only projection of the inventory
/// metadata for docs, fixtures, prototyping, and non-GPUI consumers.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct FormSchema {
    /// The source struct name (e.g. `"UserProfile"`).
    pub form: String,
    /// The source file path where the struct is declared (`file!()`).
    pub source_path: String,
    /// Whether the struct has koruma validation enabled.
    #[serde(default)]
    pub koruma_enabled: bool,
    /// Whether the source struct contains `#[gpui_form(skip)]` fields.
    #[serde(default)]
    pub has_skipped_fields: bool,
    /// One entry per non-skipped component field, in declaration order.
    pub fields: Vec<FormSchemaField>,
}

impl FormSchema {
    /// Look up an exported field entry by field name.
    pub fn field(&self, name: &str) -> Option<&FormSchemaField> {
        self.fields.iter().find(|field| field.name == name)
    }

    /// Field names in declaration order (skipped fields are absent).
    pub fn field_names(&self) -> impl Iterator<Item = &str> {
        self.fields.iter().map(|field| field.name.as_str())
    }

    /// True when koruma validation is enabled and at least one field carries
    /// validation rules — mirrors [`GpuiFormShape::has_validations`].
    pub fn has_validations(&self) -> bool {
        self.koruma_enabled
            && self
                .fields
                .iter()
                .any(|field| !field.validations.is_empty())
    }
}

impl From<&GpuiFormShape> for FormSchema {
    fn from(shape: &GpuiFormShape) -> Self {
        Self {
            form: shape.struct_name.to_string(),
            source_path: shape.source_path.to_string(),
            koruma_enabled: shape.koruma_enabled,
            has_skipped_fields: shape.has_skipped_fields,
            fields: shape
                .components
                .iter()
                .map(FormSchemaField::from)
                .collect::<Vec<_>>(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FieldVariant, FormSchema, FormSchemaComponent, GpuiFormShape};
    use crate::components::ComponentsBehaviour;

    #[test]
    fn async_validator_defaults_to_none_and_builder_round_trips() {
        let plain = FieldVariant::new("email", "String", false, ComponentsBehaviour::Input);
        assert_eq!(plain.async_validator, None);
        assert_eq!(plain.async_validator(), None);

        const FIELDS: [FieldVariant; 1] =
            [
                FieldVariant::new("email", "String", false, ComponentsBehaviour::Input)
                    .with_async_validator(Some("my_check")),
            ];
        const SHAPE: GpuiFormShape = GpuiFormShape::new("Demo", &FIELDS, "src/demo.rs", false);

        assert_eq!(FIELDS[0].async_validator, Some("my_check"));
        assert_eq!(SHAPE.components[0].async_validator(), Some("my_check"));

        let cleared = FieldVariant::new("email", "String", false, ComponentsBehaviour::Input)
            .with_async_validator(Some("my_check"))
            .with_async_validator(None);
        assert_eq!(cleared.async_validator, None);
    }

    #[test]
    fn dependency_payloads_default_to_none_and_round_trip() {
        let plain = FieldVariant::new(
            "state",
            "State",
            true,
            ComponentsBehaviour::Select(crate::components::SelectBehaviour {
                searchable: false,
                partial: false,
            }),
        );
        assert_eq!(plain.depends_on, None);
        assert_eq!(plain.required_if, None);
        assert_eq!(plain.depends_on(), None);
        assert_eq!(plain.required_if(), None);

        const FIELDS: [FieldVariant; 1] = [FieldVariant::new(
            "state",
            "State",
            true,
            ComponentsBehaviour::Select(crate::components::SelectBehaviour {
                searchable: false,
                partial: false,
            }),
        )
        .with_depends_on(Some("country"))
        .with_required_if(Some("subscribe"))];
        const SHAPE: GpuiFormShape = GpuiFormShape::new("Demo", &FIELDS, "src/demo.rs", true);

        assert_eq!(FIELDS[0].depends_on(), Some("country"));
        assert_eq!(FIELDS[0].required_if(), Some("subscribe"));
        assert_eq!(SHAPE.components[0].depends_on(), Some("country"));
        assert_eq!(SHAPE.components[0].required_if(), Some("subscribe"));

        let cleared = FieldVariant::new(
            "state",
            "State",
            true,
            ComponentsBehaviour::Select(crate::components::SelectBehaviour {
                searchable: false,
                partial: false,
            }),
        )
        .with_depends_on(Some("country"))
        .with_required_if(Some("subscribe"))
        .with_depends_on(None)
        .with_required_if(None);
        assert_eq!(cleared.depends_on, None);
        assert_eq!(cleared.required_if, None);
    }

    fn export_fixture() -> GpuiFormShape {
        const FIELDS: [FieldVariant; 4] = [
            FieldVariant::new("country", "u8", true, ComponentsBehaviour::Input).with_layout(
                crate::layout::FieldLayout::new()
                    .with_section(Some("Location"))
                    .with_label(Some("Country"))
                    .with_width(crate::layout::LayoutWidth::Half),
            ),
            FieldVariant::new(
                "state",
                "u8",
                true,
                ComponentsBehaviour::Select(crate::components::SelectBehaviour {
                    searchable: true,
                    partial: false,
                }),
            )
            .with_validations(&["RequiredIfValidation"])
            .with_depends_on(Some("country"))
            .with_required_if(Some("country")),
            FieldVariant::new(
                "address",
                "Address",
                false,
                ComponentsBehaviour::Nested(crate::components::NestedBehaviour {
                    form_struct: "Address",
                }),
            )
            .with_validations(&["NestedValidation"]),
            FieldVariant::new(
                "tags",
                "String",
                false,
                ComponentsBehaviour::List(crate::components::ListBehaviour {
                    item: crate::components::ComponentKind::Input,
                    item_struct: None,
                    add_label: Some("Add tag"),
                }),
            )
            .with_validations(&["LenValidation"]),
        ];
        GpuiFormShape::new("Profile", &FIELDS, "src/profile.rs", true).with_skipped_fields(true)
    }

    #[test]
    fn form_schema_projects_shape_metadata_faithfully() {
        let shape = export_fixture();
        let schema = FormSchema::from(&shape);

        assert_eq!(schema.form, "Profile");
        assert_eq!(schema.source_path, "src/profile.rs");
        assert!(schema.koruma_enabled);
        assert!(schema.has_skipped_fields);
        assert!(schema.has_validations());
        assert_eq!(
            schema.field_names().collect::<Vec<_>>(),
            vec!["country", "state", "address", "tags"]
        );

        let country = schema.field("country").expect("country entry");
        assert_eq!(country.layout.section.as_deref(), Some("Location"));
        assert_eq!(country.layout.label.as_deref(), Some("Country"));
        assert_eq!(country.layout.width, crate::layout::LayoutWidth::Half);
        assert!(country.layout.description.is_none());

        let state = schema.field("state").expect("state entry");
        assert_eq!(state.depends_on.as_deref(), Some("country"));
        assert_eq!(state.required_if.as_deref(), Some("country"));
        assert_eq!(state.validations, vec!["RequiredIfValidation".to_string()]);
        assert_eq!(
            state.component,
            FormSchemaComponent::Select {
                searchable: true,
                partial: false
            }
        );

        let address = schema.field("address").expect("address entry");
        assert!(address.is_nested());
        assert_eq!(address.kind(), crate::components::ComponentKind::Nested);
        assert_eq!(
            address.component,
            FormSchemaComponent::Nested {
                form_struct: "Address".to_string()
            }
        );

        let tags = schema.field("tags").expect("tags entry");
        assert!(tags.is_list());
        assert!(!tags.is_nested());
        assert_eq!(
            tags.component,
            FormSchemaComponent::List {
                item: crate::components::ComponentKind::Input,
                item_struct: None,
                add_label: Some("Add tag".to_string()),
            }
        );

        assert!(schema.field("missing").is_none());
    }

    #[test]
    fn behaviour_payloads_project_to_their_mirror_variants() {
        use crate::components::{
            ComponentsBehaviour, NumberInputBehaviour, NumberInputKind, PhoneInputBehaviour,
        };

        let number =
            FormSchemaComponent::from(&ComponentsBehaviour::NumberInput(NumberInputBehaviour {
                validation_type: Some("f64"),
                kind: NumberInputKind::SignedInteger,
            }));
        assert_eq!(
            number,
            FormSchemaComponent::NumberInput {
                validation_type: Some("f64".to_string()),
                kind: NumberInputKind::SignedInteger,
            }
        );

        let phone =
            FormSchemaComponent::from(&ComponentsBehaviour::PhoneInput(PhoneInputBehaviour {
                country_field: Some("country"),
            }));
        assert_eq!(
            phone,
            FormSchemaComponent::PhoneInput {
                country_field: Some("country".to_string()),
            }
        );
        assert_eq!(phone.kind(), crate::components::ComponentKind::PhoneInput);
    }

    #[test]
    fn form_schema_round_trips_through_serde_json() {
        let schema = FormSchema::from(&export_fixture());

        let json = serde_json::to_string(&schema).expect("serialize FormSchema");
        let restored: FormSchema = serde_json::from_str(&json).expect("deserialize FormSchema");
        assert_eq!(restored, schema, "exported schema must round-trip");

        assert_eq!(
            restored.field("state").expect("state entry").required_if,
            Some("country".to_string())
        );
        assert!(
            restored
                .field("address")
                .expect("address entry")
                .is_nested()
        );
        assert!(restored.field("tags").expect("tags entry").is_list());
    }

    #[test]
    fn form_schema_json_uses_snake_case_component_tags() {
        const FIELDS: [FieldVariant; 1] = [FieldVariant::new(
            "age",
            "u32",
            false,
            ComponentsBehaviour::NumberInput(crate::components::NumberInputBehaviour {
                validation_type: None,
                kind: crate::components::NumberInputKind::Float,
            }),
        )];
        let shape = GpuiFormShape::new("Demo", &FIELDS, "src/demo.rs", false);
        let json = serde_json::to_string(&FormSchema::from(&shape)).unwrap();
        assert!(
            json.contains(r#""component":"number_input""#),
            "component tag must be snake_case in {json}"
        );
        assert!(
            json.contains(r#""kind":"float""#),
            "number kind must be snake_case in {json}"
        );
    }

    #[test]
    fn form_schema_wire_format_omits_absent_hints() {
        const FIELDS: [FieldVariant; 1] = [FieldVariant::new(
            "name",
            "String",
            false,
            ComponentsBehaviour::Input,
        )];
        let shape = GpuiFormShape::new("Demo", &FIELDS, "src/demo.rs", false);
        let json = serde_json::to_string(&FormSchema::from(&shape)).unwrap();
        assert!(
            !json.contains("section"),
            "absent hint must be omitted: {json}"
        );
        assert!(
            !json.contains("depends_on"),
            "absent dependency must be omitted: {json}"
        );

        let restored: FormSchema = serde_json::from_str(&json).unwrap();
        let field = restored.field("name").unwrap();
        assert_eq!(field.layout.section, None);
        assert_eq!(field.depends_on, None);
        assert_eq!(field.layout.width, crate::layout::LayoutWidth::Full);
    }
}
