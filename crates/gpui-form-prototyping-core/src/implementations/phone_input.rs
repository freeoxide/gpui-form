use gpui_form_schema::registry::{FieldVariant, GpuiFormShape};
use proc_macro2::TokenStream;
use quote::quote;

use crate::imports::ImportItem;

use super::{
    FieldCodeGenerator, GeneratedSubscription, ResolvedField, input::InputCodeGenerator,
    render_standard_field,
};

/// Code generator for `component(phone_input)` fields.
///
/// The rendered control is a plain `gpui_component::input::Input`, so the value
/// wiring (entity creation, subscription, text prefill, focus) is identical to
/// [`InputCodeGenerator`] and is delegated to it. Only the rendered child
/// differs: it emits `Input::new(...)` directly instead of resolving the
/// behaviour's PascalCase component name (`PhoneInput`), which is not a real
/// control type.
pub struct PhoneInputCodeGenerator;

impl FieldCodeGenerator for PhoneInputCodeGenerator {
    fn generate_imports(&self, field: &FieldVariant) -> Vec<ImportItem> {
        InputCodeGenerator.generate_imports(field)
    }

    fn generate_cx_new_call(
        &self,
        field: &ResolvedField<'_>,
        component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        InputCodeGenerator.generate_cx_new_call(field, component)
    }

    fn generate_post_subscription_initialization(
        &self,
        field: &ResolvedField<'_>,
        component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        InputCodeGenerator.generate_post_subscription_initialization(field, component)
    }

    fn generate_field_initializers(
        &self,
        field: &ResolvedField<'_>,
        component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        InputCodeGenerator.generate_field_initializers(field, component)
    }

    fn generate_render_child(
        &self,
        field: &ResolvedField<'_>,
        component: &GpuiFormShape,
    ) -> TokenStream {
        let field_in_struct_name_ident = field.field_ident_with_behaviour();
        render_standard_field(
            field,
            component,
            quote! { Input::new(&self.fields.#field_in_struct_name_ident) },
        )
    }

    fn generate_focusable_cycle(
        &self,
        field: &ResolvedField<'_>,
        component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        InputCodeGenerator.generate_focusable_cycle(field, component)
    }

    fn generate_subscription(
        &self,
        field: &ResolvedField<'_>,
        component: &GpuiFormShape,
    ) -> Option<GeneratedSubscription> {
        InputCodeGenerator.generate_subscription(field, component)
    }
}

#[cfg(test)]
mod tests {
    use super::PhoneInputCodeGenerator;
    use crate::implementations::{FieldCodeGenerator as _, ResolvedField};
    use gpui_form_schema::{
        components::{ComponentsBehaviour, PhoneInputBehaviour},
        registry::{FieldVariant, GpuiFormShape},
    };

    fn compact(input: &str) -> String {
        input.chars().filter(|c| !c.is_whitespace()).collect()
    }

    #[test]
    fn phone_input_renders_a_plain_input_control() {
        const FIELDS: [FieldVariant; 1] = [FieldVariant::new(
            "phone_number",
            "String",
            true,
            ComponentsBehaviour::PhoneInput(PhoneInputBehaviour { country_field: None }),
        )];
        const SHAPE: GpuiFormShape = GpuiFormShape::new("Demo", &FIELDS, "src/demo.rs", false);

        let field = ResolvedField::new(&FIELDS[0]).unwrap();
        let rendered = compact(
            &PhoneInputCodeGenerator
                .generate_render_child(&field, &SHAPE)
                .to_string(),
        );

        assert!(
            rendered.contains("Input::new(&self.fields.phone_number_phone_input)"),
            "phone_input should render a plain Input control: {rendered}"
        );
        assert!(
            !rendered.contains("PhoneInput::new"),
            "phone_input must not render a nonexistent PhoneInput control: {rendered}"
        );
    }
}
