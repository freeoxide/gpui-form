pub mod checkbox;
pub mod custom;
pub mod date_picker;
pub mod file_picker;
pub mod infinite_select;
pub mod input;
pub mod number_input;
pub mod select;
pub mod switch;

use gpui_form_schema::{
    components::ComponentsBehaviour,
    layout::FieldLayout,
    registry::{FieldVariant, GpuiFormShape},
};
use heck::{ToPascalCase as _, ToSnakeCase as _};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, Path, Type};

use crate::{
    error::{PrototypingError, PrototypingResult},
    imports::ImportItem,
};

static INPUT_GENERATOR: input::InputCodeGenerator = input::InputCodeGenerator;
static NUMBER_INPUT_GENERATOR: number_input::NumberInputCodeGenerator =
    number_input::NumberInputCodeGenerator;
static CHECKBOX_GENERATOR: checkbox::CheckboxCodeGenerator = checkbox::CheckboxCodeGenerator;
static SWITCH_GENERATOR: switch::SwitchCodeGenerator = switch::SwitchCodeGenerator;
static SELECT_GENERATOR: select::SelectCodeGenerator = select::SelectCodeGenerator;
static INFINITE_SELECT_GENERATOR: infinite_select::InfiniteSelectCodeGenerator =
    infinite_select::InfiniteSelectCodeGenerator;
static CUSTOM_GENERATOR: custom::CustomCodeGenerator = custom::CustomCodeGenerator;
static DATE_PICKER_GENERATOR: date_picker::DatePickerCodeGenerator =
    date_picker::DatePickerCodeGenerator;
static FILE_PICKER_GENERATOR: file_picker::FilePickerCodeGenerator =
    file_picker::FilePickerCodeGenerator;

pub fn field_generator(behaviour: &ComponentsBehaviour) -> &'static dyn FieldCodeGenerator {
    match behaviour {
        ComponentsBehaviour::Input => &INPUT_GENERATOR,
        ComponentsBehaviour::NumberInput(_) => &NUMBER_INPUT_GENERATOR,
        ComponentsBehaviour::Checkbox => &CHECKBOX_GENERATOR,
        ComponentsBehaviour::Switch => &SWITCH_GENERATOR,
        ComponentsBehaviour::Select(_) => &SELECT_GENERATOR,
        ComponentsBehaviour::InfiniteSelect(_) => &INFINITE_SELECT_GENERATOR,
        ComponentsBehaviour::Custom => &CUSTOM_GENERATOR,
        ComponentsBehaviour::DatePicker => &DATE_PICKER_GENERATOR,
        ComponentsBehaviour::FilePicker => &FILE_PICKER_GENERATOR,
    }
}

pub struct ResolvedField<'a> {
    field: &'a FieldVariant,
    field_ident: Ident,
    field_ident_pascal: Ident,
    field_ident_with_behaviour: Ident,
    value_type: Type,
    component_ident: Ident,
    custom_shape_path: Option<Path>,
    custom_component_path: Option<Path>,
}

impl<'a> ResolvedField<'a> {
    pub fn new(field: &'a FieldVariant) -> PrototypingResult<Self> {
        let value_type = syn::parse_str::<Type>(field.value_type).map_err(|error| {
            PrototypingError::InvalidType {
                field_name: field.field_name.to_string(),
                value: field.value_type.to_string(),
                error: error.to_string(),
            }
        })?;

        let custom_component_path = match field.custom_component {
            Some(component_path) => {
                Some(syn::parse_str::<Path>(component_path).map_err(|error| {
                    PrototypingError::InvalidPath {
                        kind: "custom component path",
                        value: component_path.to_string(),
                        error: error.to_string(),
                    }
                })?)
            },
            None => None,
        };

        let custom_shape_path = match field.custom_shape {
            Some(shape_path) => Some(syn::parse_str::<Path>(shape_path).map_err(|error| {
                PrototypingError::InvalidPath {
                    kind: "custom component shape path",
                    value: shape_path.to_string(),
                    error: error.to_string(),
                }
            })?),
            None => None,
        };

        Ok(Self {
            field,
            field_ident: format_ident!("{}", field.field_name),
            field_ident_pascal: format_ident!("{}", field.field_name_pascal()),
            field_ident_with_behaviour: format_ident!("{}", field.field_name_with_behaviour()),
            value_type,
            component_ident: format_ident!("{}", field.behaviour.component_name().to_pascal_case()),
            custom_shape_path,
            custom_component_path,
        })
    }

    pub fn raw(&self) -> &FieldVariant {
        self.field
    }

    pub fn behaviour(&self) -> &ComponentsBehaviour {
        &self.field.behaviour
    }

    /// Non-rendering layout hints attached to this field (METADATA-FIRST v1).
    ///
    /// Mirrors [`ResolvedField::raw`] / [`ResolvedField::behaviour`]: a thin
    /// accessor over the underlying [`FieldVariant::layout`]. Consumers use this
    /// to read `section` / `label` / `description` / `placeholder` / `width`
    /// hints when emitting scaffolds. See [`FieldLayout`] for the v1 contract.
    pub fn layout(&self) -> &FieldLayout {
        &self.field.layout
    }

    pub fn field_name(&self) -> &'a str {
        self.field.field_name
    }

    pub fn field_ident(&self) -> &Ident {
        &self.field_ident
    }

    pub fn field_ident_pascal(&self) -> &Ident {
        &self.field_ident_pascal
    }

    pub fn field_ident_with_behaviour(&self) -> &Ident {
        &self.field_ident_with_behaviour
    }

    pub fn value_type(&self) -> &Type {
        &self.value_type
    }

    pub fn component_ident(&self) -> &Ident {
        &self.component_ident
    }

    pub fn optional(&self) -> bool {
        self.field.optional
    }

    pub fn value_holder_wraps_in_option(&self) -> bool {
        self.field.value_holder_wraps_in_option()
    }

    pub fn custom_value_binding(&self) -> bool {
        self.field.custom_value_binding
    }

    pub fn custom_component(&self) -> Option<&'a str> {
        self.field.custom_component
    }

    pub fn custom_shape_path(&self) -> Option<&Path> {
        self.custom_shape_path.as_ref()
    }

    pub fn custom_component_path(&self) -> Option<&Path> {
        self.custom_component_path.as_ref()
    }

    pub fn kebab_id(&self) -> String {
        self.field.kebab_id()
    }

    pub fn validation_rules(&self) -> &'static [&'static str] {
        self.field.validation_rules()
    }

    pub fn has_validation_rule(&self, rule: &str) -> bool {
        self.validation_rules().contains(&rule)
    }

    pub fn uses_optional_inner_validation_errors(&self) -> bool {
        self.optional()
            && (self.has_validation_rule("NewtypeValidation")
                || self.has_validation_rule("NestedValidation"))
    }

    pub fn suffixed_ident(&self, suffix: &str) -> Ident {
        format_ident!("{}_{}", self.field.field_name, suffix)
    }

    pub fn prefixed_ident(&self, prefix: &str) -> Ident {
        format_ident!("{}_{}", prefix, self.field.field_name)
    }

    pub fn event_handler_ident(&self, suffix: &str) -> Ident {
        format_ident!("on_{}_{}", self.field.field_name, suffix)
    }
}

#[derive(Default)]
pub struct GeneratedSubscription {
    pub calls: Vec<TokenStream>,
    pub handlers: Vec<TokenStream>,
}

impl GeneratedSubscription {
    pub fn is_empty(&self) -> bool {
        self.calls.is_empty() && self.handlers.is_empty()
    }
}

pub trait FieldCodeGenerator {
    fn generate_imports(&self, _field: &FieldVariant) -> Vec<ImportItem> {
        vec![]
    }

    fn generate_cx_new_call(
        &self,
        field: &ResolvedField<'_>,
        component: &GpuiFormShape,
    ) -> Option<TokenStream>;

    fn generate_field_initializers(
        &self,
        field: &ResolvedField<'_>,
        component: &GpuiFormShape,
    ) -> Option<TokenStream>;

    fn generate_render_child(
        &self,
        field: &ResolvedField<'_>,
        component: &GpuiFormShape,
    ) -> TokenStream;

    fn generate_focusable_cycle(
        &self,
        field: &ResolvedField<'_>,
        component: &GpuiFormShape,
    ) -> Option<TokenStream>;

    fn generate_subscription(
        &self,
        field: &ResolvedField<'_>,
        component: &GpuiFormShape,
    ) -> Option<GeneratedSubscription>;

    fn generate_post_subscription_initialization(
        &self,
        _field: &ResolvedField<'_>,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        None
    }
}

pub trait ShapeIdentities {
    fn struct_name(&self) -> &'static str;

    fn struct_name_ident(&self) -> syn::Ident {
        format_ident!("{}", self.struct_name())
    }

    fn struct_form_ident(&self) -> syn::Ident {
        format_ident!("{}Form", self.struct_name())
    }

    fn struct_form_components_ident(&self) -> syn::Ident {
        format_ident!("{}FormComponents", self.struct_name())
    }

    fn struct_form_fields_ident(&self) -> syn::Ident {
        format_ident!("{}FormFields", self.struct_name())
    }

    fn form_id_literal(&self) -> String {
        format!("{}-form", self.struct_name().to_snake_case())
    }

    fn ftl_label_ident(&self) -> syn::Ident {
        format_ident!("{}LabelVariants", self.struct_name())
    }

    fn ftl_description_ident(&self) -> syn::Ident {
        format_ident!("{}DescriptionVariants", self.struct_name())
    }
}

impl ShapeIdentities for GpuiFormShape {
    fn struct_name(&self) -> &'static str {
        self.struct_name
    }
}

pub fn generate_entity_creation(
    field: &ResolvedField<'_>,
    component: &GpuiFormShape,
) -> TokenStream {
    let form_components_struct_ident = component.struct_form_components_ident();
    let var_name_ident = field.field_ident_with_behaviour().clone();
    let fn_name_ident = var_name_ident.clone();

    quote! {
        let #var_name_ident =
            cx.new(|cx| #form_components_struct_ident::#fn_name_ident(window, cx));
    }
}

pub fn generate_entity_field_initializer(field: &ResolvedField<'_>) -> TokenStream {
    let field_var_name_ident = field.field_ident_with_behaviour();
    quote! { #field_var_name_ident, }
}

pub fn generate_entity_focus(field: &ResolvedField<'_>) -> TokenStream {
    let field_var_name_ident = field.field_ident_with_behaviour();
    quote! {
        self.fields.#field_var_name_ident.focus_handle(cx),
    }
}

pub fn generate_text_value_prefill(field: &ResolvedField<'_>) -> TokenStream {
    let field_var_name_ident = field.field_ident_with_behaviour();
    let field_name_ident = field.field_ident();

    if field.value_holder_wraps_in_option() {
        quote! {
            if let Some(value) = current_data.#field_name_ident.as_ref() {
                #field_var_name_ident.update(cx, |state, cx| {
                    state.set_value(value.to_string(), window, cx);
                });
            }
        }
    } else {
        quote! {
            #field_var_name_ident.update(cx, |state, cx| {
                state.set_value(current_data.#field_name_ident.to_string(), window, cx);
            });
        }
    }
}

pub fn render_standard_field(
    field: &ResolvedField<'_>,
    component: &GpuiFormShape,
    child_tokens: TokenStream,
) -> TokenStream {
    let description_fn_tokens = generate_description_fn_tokens(field, component);
    let label_tokens = generate_label_tokens(field, component);

    quote! {
        .child(
            field()
                .label(#label_tokens)
                #description_fn_tokens
                .child(#child_tokens)
        )
    }
}

pub fn render_component_entity_field(
    field: &ResolvedField<'_>,
    component: &GpuiFormShape,
) -> TokenStream {
    let component_gpui_type = field.component_ident();
    let field_in_struct_name_ident = field.field_ident_with_behaviour();

    render_standard_field(
        field,
        component,
        quote! { #component_gpui_type::new(&self.fields.#field_in_struct_name_ident) },
    )
}

pub fn generate_label_tokens(
    field: &ResolvedField<'_>,
    component: &GpuiFormShape,
) -> proc_macro2::TokenStream {
    #[cfg(not(feature = "fluent"))]
    let _ = component;

    #[cfg(feature = "fluent")]
    {
        let ftl_label_ident = component.ftl_label_ident();
        let field_name_pascal_case_ident = field.field_ident_pascal();
        quote! {{
            let message = #ftl_label_ident::#field_name_pascal_case_ident;
            localize(cx, &message)
        }}
    }
    #[cfg(not(feature = "fluent"))]
    {
        // METADATA-FIRST v1: prefer an explicit `layout.label` hint when the
        // field declared one. Fall back to a title-cased field name otherwise.
        // (label defaults to the field name at consumption time per the v1
        // contract — see `gpui_form_schema::layout`.)
        if let Some(label) = field.layout().label {
            quote! { #label }
        } else {
            use heck::ToTitleCase as _;
            let title = field.field_name().to_title_case();
            quote! { #title }
        }
    }
}

pub fn generate_description_fn_tokens(
    field: &ResolvedField<'_>,
    component: &GpuiFormShape,
) -> proc_macro2::TokenStream {
    let field_name_ident = field.field_ident();

    #[cfg(feature = "fluent")]
    let description_tokens = {
        let ftl_description_ident = component.ftl_description_ident();
        let field_name_pascal_case_ident = field.field_ident_pascal();
        quote! {{
            let message = #ftl_description_ident::#field_name_pascal_case_ident;
            localize(cx, &message)
        }}
    };
    #[cfg(not(feature = "fluent"))]
    let description_tokens = {
        // METADATA-FIRST v1: prefer an explicit `layout.description` hint when
        // present, falling back to the title-cased field name.
        if let Some(description) = field.layout().description {
            quote! { #description }
        } else {
            use heck::ToTitleCase as _;
            let title = field.field_name().to_title_case();
            quote! { #title }
        }
    };

    let field_has_validations = !field.validation_rules().is_empty() && component.has_koruma();
    let uses_optional_inner_validation_errors = field.uses_optional_inner_validation_errors();
    let error_tokens = if field_has_validations {
        #[cfg(feature = "fluent")]
        let conversion_tokens = quote! {
            localize(cx, v)
        };
        #[cfg(not(feature = "fluent"))]
        let conversion_tokens = quote! { v.to_string() };

        if uses_optional_inner_validation_errors {
            quote! {
                validation_errors
                    .as_ref()
                    .and_then(|e| e.#field_name_ident())
                    .map(|inner_error| inner_error.all())
                    .filter(|errs| !errs.is_empty())
                    .map(|errs| {
                        errs.iter()
                            .map(|v| #conversion_tokens)
                            .collect::<Vec<_>>()
                            .join("\n")
                    })
            }
        } else {
            quote! {{
                validation_errors.as_ref().and_then(|e| {
                    let errs = e.#field_name_ident().all();
                    if errs.is_empty() {
                        None
                    } else {
                        Some(
                            errs.iter()
                                .map(|v| #conversion_tokens)
                                .collect::<Vec<_>>()
                            .join("\n"),
                        )
                    }
                })
            }}
        }
    } else {
        quote! {{ None }}
    };
    let error_color_tokens = quote! { cx.theme().danger };

    if !field_has_validations {
        quote! {
            .description_fn({
                let description = #description_tokens;
                move |_, _| {
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(div().child(description.clone()))
                }
            })
        }
    } else {
        quote! {
            .description_fn({
                let description = #description_tokens;
                let error = #error_tokens;
                let error_color = #error_color_tokens;
                move |_, _| {
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(div().child(description.clone()))
                        .when(error.is_some(), |this| {
                            this.child(
                                div()
                                    .text_color(error_color)
                                    .child(error.clone().unwrap_or_default()),
                            )
                        })
                }
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ResolvedField, generate_description_fn_tokens};
    // `generate_label_tokens` is only exercised by the non-fluent label tests;
    // the fluent label branch localizes through `es-fluent` keys instead.
    #[cfg(not(feature = "fluent"))]
    use super::generate_label_tokens;
    use gpui_form_schema::{
        components::ComponentsBehaviour,
        layout::{FieldLayout, LayoutWidth},
        registry::{FieldVariant, GpuiFormShape},
    };

    fn compact(input: &str) -> String {
        input.chars().filter(|c| !c.is_whitespace()).collect()
    }

    // ── METADATA-FIRST v1: layout.label / layout.description consumption ──────
    // The non-fluent label/description branches consume `layout.label` /
    // `layout.description`. The fluent branches localize through
    // `es-fluent` keys instead (v1 minimal scope — fluent policy is deferred),
    // so these assertions are only meaningful without the `fluent` feature.

    #[test]
    #[cfg(not(feature = "fluent"))]
    fn label_uses_layout_label_when_present() {
        const LAYOUT: FieldLayout = FieldLayout::new().with_label(Some("Enable experiments"));
        const FIELDS: [FieldVariant; 1] =
            [FieldVariant::new("enable_experimental", "bool", false, ComponentsBehaviour::Switch)
                .with_layout(LAYOUT)];
        const SHAPE: GpuiFormShape = GpuiFormShape::new("Demo", &FIELDS, "src/demo.rs", false);

        let field = ResolvedField::new(&FIELDS[0]).expect("field metadata should parse");
        let tokens = generate_label_tokens(&field, &SHAPE).to_string();

        assert!(
            tokens.contains("Enable experiments"),
            "explicit layout.label should be used as the label: {tokens}"
        );
        assert!(
            !tokens.contains("Enable Experimental"),
            "title-cased fallback must not be used when layout.label is set: {tokens}"
        );
    }

    #[test]
    #[cfg(not(feature = "fluent"))]
    fn label_falls_back_to_field_name_title_case_when_absent() {
        // Empty layout (no label) — defaults via FieldLayout::new().
        const FIELDS: [FieldVariant; 1] = [
            FieldVariant::new("enable_experimental", "bool", false, ComponentsBehaviour::Switch),
        ];
        const SHAPE: GpuiFormShape = GpuiFormShape::new("Demo", &FIELDS, "src/demo.rs", false);

        let field = ResolvedField::new(&FIELDS[0]).expect("field metadata should parse");
        let tokens = generate_label_tokens(&field, &SHAPE).to_string();

        assert!(
            tokens.contains("Enable Experimental"),
            "field name should be title-cased as the fallback label: {tokens}"
        );
    }

    #[test]
    #[cfg(not(feature = "fluent"))]
    fn description_uses_layout_description_when_present() {
        const LAYOUT: FieldLayout =
            FieldLayout::new().with_description(Some("Toggles unreleased features"));
        const FIELDS: [FieldVariant; 1] =
            [FieldVariant::new("enable_experimental", "bool", false, ComponentsBehaviour::Switch)
                .with_layout(LAYOUT)];
        const SHAPE: GpuiFormShape = GpuiFormShape::new("Demo", &FIELDS, "src/demo.rs", false);

        let field = ResolvedField::new(&FIELDS[0]).expect("field metadata should parse");
        let tokens = generate_description_fn_tokens(&field, &SHAPE).to_string();

        assert!(
            tokens.contains("Toggles unreleased features"),
            "explicit layout.description should be used as the description hint: {tokens}"
        );
        assert!(
            !tokens.contains("Enable Experimental"),
            "title-cased fallback must not be used when layout.description is set: {tokens}"
        );
    }

    #[test]
    #[cfg(not(feature = "fluent"))]
    fn description_falls_back_to_field_name_title_case_when_absent() {
        const FIELDS: [FieldVariant; 1] = [
            FieldVariant::new("enable_experimental", "bool", false, ComponentsBehaviour::Switch),
        ];
        const SHAPE: GpuiFormShape = GpuiFormShape::new("Demo", &FIELDS, "src/demo.rs", false);

        let field = ResolvedField::new(&FIELDS[0]).expect("field metadata should parse");
        let tokens = generate_description_fn_tokens(&field, &SHAPE).to_string();

        assert!(
            tokens.contains("Enable Experimental"),
            "field name should be title-cased as the fallback description: {tokens}"
        );
    }

    #[test]
    fn resolved_field_layout_accessor_returns_field_layout() {
        // Guards the public accessor the code_gen section-grouping loop reads.
        const LAYOUT: FieldLayout = FieldLayout::new()
            .with_section(Some("Account"))
            .with_label(Some("Username"))
            .with_width(LayoutWidth::Half);
        const FIELDS: [FieldVariant; 1] =
            [FieldVariant::new("username", "String", false, ComponentsBehaviour::Input)
                .with_layout(LAYOUT)];
        const SHAPE: GpuiFormShape = GpuiFormShape::new("Demo", &FIELDS, "src/demo.rs", false);

        let _ = SHAPE;
        let field = ResolvedField::new(&FIELDS[0]).expect("field metadata should parse");
        let layout = field.layout();

        assert_eq!(layout.section, Some("Account"));
        assert_eq!(layout.label, Some("Username"));
        assert_eq!(layout.width, LayoutWidth::Half);
        assert!(!layout.is_empty());
    }

    #[test]
    fn description_uses_direct_all_for_non_optional_newtype_errors() {
        const VALIDATIONS: &[&str] = &["NewtypeValidation"];
        const FIELDS: [FieldVariant; 1] =
            [
                FieldVariant::new("index", "Age", false, ComponentsBehaviour::Input)
                    .with_validations(VALIDATIONS),
            ];
        const SHAPE: GpuiFormShape = GpuiFormShape::new("Demo", &FIELDS, "src/demo.rs", true);

        let field = ResolvedField::new(&FIELDS[0]).expect("field metadata should parse");
        let compact = compact(&generate_description_fn_tokens(&field, &SHAPE).to_string());

        assert!(
            compact.contains("leterrs=e.index().all();"),
            "non-optional newtype errors should keep direct .all() access: {compact}"
        );
        assert!(
            !compact.contains("e.index().and_then(|inner_error|"),
            "non-optional newtype errors should not be treated as optional: {compact}"
        );
    }

    #[test]
    fn description_unwraps_optional_newtype_inner_errors_before_all() {
        const VALIDATIONS: &[&str] = &["NewtypeValidation"];
        const FIELDS: [FieldVariant; 1] =
            [
                FieldVariant::new("age", "Age", true, ComponentsBehaviour::Input)
                    .with_validations(VALIDATIONS),
            ];
        const SHAPE: GpuiFormShape = GpuiFormShape::new("Demo", &FIELDS, "src/demo.rs", true);

        let field = ResolvedField::new(&FIELDS[0]).expect("field metadata should parse");
        let compact = compact(&generate_description_fn_tokens(&field, &SHAPE).to_string());

        assert!(
            compact.contains(
                "validation_errors.as_ref().and_then(|e|e.age()).map(|inner_error|inner_error.all()).filter(|errs|!errs.is_empty()).map(|errs|{"
            ),
            "optional newtype errors should map/filter the inner errors before rendering: {compact}"
        );
    }

    #[test]
    fn description_unwraps_optional_nested_inner_errors_before_all() {
        const VALIDATIONS: &[&str] = &["NestedValidation"];
        const FIELDS: [FieldVariant; 1] =
            [
                FieldVariant::new("address", "Address", true, ComponentsBehaviour::Input)
                    .with_validations(VALIDATIONS),
            ];
        const SHAPE: GpuiFormShape = GpuiFormShape::new("Demo", &FIELDS, "src/demo.rs", true);

        let field = ResolvedField::new(&FIELDS[0]).expect("field metadata should parse");
        let compact = compact(&generate_description_fn_tokens(&field, &SHAPE).to_string());

        assert!(
            compact.contains(
                "validation_errors.as_ref().and_then(|e|e.address()).map(|inner_error|inner_error.all()).filter(|errs|!errs.is_empty()).map(|errs|{"
            ),
            "optional nested errors should map/filter the inner errors before rendering: {compact}"
        );
    }
}
