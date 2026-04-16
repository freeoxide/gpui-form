use gpui_form_schema::registry::{FieldVariant, GpuiFormShape};
use proc_macro2::TokenStream;
use quote::quote;

use crate::imports::ImportItem;

use super::{
    FieldCodeGenerator, FieldVariantExt as _, GeneratedSubscription, generate_entity_creation,
    generate_entity_field_initializer, render_standard_field,
};

/// Custom component support in prototyping initializes generated form fields
/// but does not infer subscriptions or actual widget rendering.
pub struct CustomCodeGenerator;

impl FieldCodeGenerator for CustomCodeGenerator {
    fn generate_imports(&self, field: &FieldVariant) -> Vec<ImportItem> {
        // Only emit an import when the path is qualified (contains `::`).
        // A bare name like `TagsInput` is already brought in scope by the
        // `use source_module::*;` glob and needs no separate import.
        match field.custom_component {
            Some(path) if path.contains("::") => vec![ImportItem::path(path)],
            _ => vec![],
        }
    }

    fn generate_cx_new_call(
        &self,
        field: &FieldVariant,
        component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        Some(generate_entity_creation(field, component))
    }

    fn generate_field_initializers(
        &self,
        field: &FieldVariant,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        Some(generate_entity_field_initializer(field))
    }

    fn generate_render_child(
        &self,
        field: &FieldVariant,
        component: &GpuiFormShape,
    ) -> TokenStream {
        let field_in_struct_name_ident = field.field_ident_with_behaviour();

        // When the component type is known, emit Component::new(&entity) like other components.
        // The component type is in scope via `use {module}::*;` in the generated file.
        let child_tokens = if let Some(component_str) = field.custom_component {
            let component_path: syn::Path =
                syn::parse_str(component_str).expect("custom_component should be a valid path");
            quote! {
                #component_path::new(&self.fields.#field_in_struct_name_ident)
            }
        } else {
            let field_name = field.field_name;
            quote! {
                div().child(format!(
                    "Custom component `{}` – wire rendering via self.fields.{}",
                    #field_name,
                    stringify!(#field_in_struct_name_ident)
                ))
            }
        };

        render_standard_field(field, component, child_tokens)
    }

    fn generate_focusable_cycle(
        &self,
        _field: &FieldVariant,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        None
    }

    fn generate_subscription(
        &self,
        _field: &FieldVariant,
        _component: &GpuiFormShape,
    ) -> Option<GeneratedSubscription> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::CustomCodeGenerator;
    use crate::implementations::FieldCodeGenerator as _;
    use gpui_form_schema::{
        components::ComponentsBehaviour,
        registry::{FieldVariant, GpuiFormShape},
    };

    const CUSTOM_FIELDS: [FieldVariant; 1] = [FieldVariant::new(
        "tags",
        "Vec<String>",
        false,
        ComponentsBehaviour::Custom,
    )];
    const CUSTOM_SHAPE: GpuiFormShape =
        GpuiFormShape::new("Demo", &CUSTOM_FIELDS, "src/demo.rs", false);

    fn compact(input: &str) -> String {
        input.chars().filter(|c| !c.is_whitespace()).collect()
    }

    #[test]
    fn custom_generator_initializes_state_entity() {
        let generator = CustomCodeGenerator;
        let tokens = generator
            .generate_cx_new_call(&CUSTOM_FIELDS[0], &CUSTOM_SHAPE)
            .expect("custom fields should generate cx.new initialization");
        let compact = compact(&tokens.to_string());

        assert!(
            compact
                .contains("lettags_custom=cx.new(|cx|DemoFormComponents::tags_custom(window,cx));"),
            "cx initialization should call generated FormComponents constructor for custom field"
        );
    }

    #[test]
    fn custom_generator_initializes_form_fields_struct() {
        let generator = CustomCodeGenerator;
        let tokens = generator
            .generate_field_initializers(&CUSTOM_FIELDS[0], &CUSTOM_SHAPE)
            .expect("custom fields should be included in FormFields initializer");
        let compact = compact(&tokens.to_string());

        assert!(
            compact.contains("tags_custom,"),
            "field initializer should include custom state entity"
        );
    }

    #[test]
    fn custom_generator_emits_component_call_when_component_known() {
        const FIELDS_WITH_COMPONENT: [FieldVariant; 1] =
            [
                FieldVariant::new("tags", "Vec<String>", false, ComponentsBehaviour::Custom)
                    .with_custom_component("TagsInput"),
            ];
        const SHAPE: GpuiFormShape =
            GpuiFormShape::new("Demo", &FIELDS_WITH_COMPONENT, "src/demo.rs", false);

        let generator = CustomCodeGenerator;
        let tokens = generator.generate_render_child(&FIELDS_WITH_COMPONENT[0], &SHAPE);
        let compact = compact(&tokens.to_string());

        assert!(
            compact.contains("TagsInput::new(&self.fields.tags_custom)"),
            "render should emit Component::new(&entity): got {compact}"
        );
    }
}
