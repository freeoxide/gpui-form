use gpui_form_core::registry::{FieldVariant, GpuiFormShape};
use proc_macro2::TokenStream;

use super::{FieldCodeGenerator, GeneratedSubscription};

/// Custom component support in prototyping is intentionally a no-op.
///
/// The prototyping generator cannot infer how to render or subscribe to
/// user-defined component state without additional user-provided hooks.
pub struct CustomCodeGenerator;

impl FieldCodeGenerator for CustomCodeGenerator {
    fn generate_cx_new_call(
        &self,
        _field: &FieldVariant,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        None
    }

    fn generate_field_initializers(
        &self,
        _field: &FieldVariant,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        None
    }

    fn generate_render_child(
        &self,
        _field: &FieldVariant,
        _component: &GpuiFormShape,
    ) -> TokenStream {
        TokenStream::new()
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
