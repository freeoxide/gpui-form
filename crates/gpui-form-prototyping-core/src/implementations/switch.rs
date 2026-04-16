use gpui_form_schema::registry::{FieldVariant, GpuiFormShape};
use proc_macro2::TokenStream;
use quote::quote;

use crate::imports::ImportItem;

use super::{
    FieldCodeGenerator, GeneratedSubscription, ResolvedField, render_standard_field,
};

pub struct SwitchCodeGenerator;

const IMPORTS: &[ImportItem] = &[ImportItem::path("gpui_component::switch::Switch")];

impl FieldCodeGenerator for SwitchCodeGenerator {
    fn generate_imports(&self, _field: &FieldVariant) -> Vec<ImportItem> {
        IMPORTS.to_vec()
    }

    fn generate_cx_new_call(
        &self,
        _field: &ResolvedField<'_>,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        None
    }

    fn generate_field_initializers(
        &self,
        _field: &ResolvedField<'_>,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        None
    }

    fn generate_render_child(
        &self,
        field: &ResolvedField<'_>,
        component: &GpuiFormShape,
    ) -> TokenStream {
        let field_name_ident = field.field_ident();
        let component_gpui_type = field.component_ident();

        let checkbox_id_str = field.kebab_id();

        render_standard_field(
            field,
            component,
            quote! {
                #component_gpui_type::new(#checkbox_id_str)
                    .checked(self.current_data.#field_name_ident)
                    .on_click(cx.listener(move |v, checked, _, cx| {
                        v.current_data.#field_name_ident = *checked;
                        cx.notify();
                    }))
            },
        )
    }

    fn generate_focusable_cycle(
        &self,
        _field: &ResolvedField<'_>,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        None
    }

    fn generate_subscription(
        &self,
        _field: &ResolvedField<'_>,
        _component: &GpuiFormShape,
    ) -> Option<GeneratedSubscription> {
        None
    }
}
