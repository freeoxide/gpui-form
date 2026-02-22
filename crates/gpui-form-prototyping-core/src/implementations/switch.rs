use gpui_form_core::registry::{FieldVariant, GpuiFormShape};
use proc_macro2::TokenStream;
use quote::quote;

use crate::imports::ImportItem;

use super::{FieldCodeGenerator, GeneratedSubscription};

pub struct SwitchCodeGenerator;

const IMPORTS: &[ImportItem] = &[ImportItem::path("gpui_component::switch::Switch")];

impl FieldCodeGenerator for SwitchCodeGenerator {
    fn generate_imports(&self, _field: &FieldVariant) -> Vec<ImportItem> {
        IMPORTS.to_vec()
    }

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
        field: &FieldVariant,
        component: &GpuiFormShape,
    ) -> TokenStream {
        let field_name_ident = field.field_ident();

        let component_gpui_type = field.behaviour.as_component_ident();

        let checkbox_id_str = field.kebab_id();

        let description_fn_tokens = super::generate_description_fn_tokens(field, component);
        let label_tokens = super::generate_label_tokens(field, component);

        // Show description always, and error below it when present (hidden when empty)
        quote! {
            .child(
                field()
                    .label(#label_tokens)
                    #description_fn_tokens
                    .child(#component_gpui_type::new(#checkbox_id_str)
                    .checked(self.current_data.#field_name_ident)
                    .on_click(cx.listener(move |v, checked, _, cx| {
                        v.current_data.#field_name_ident = *checked;
                        cx.notify();
                    })),
                )
            )
        }
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
