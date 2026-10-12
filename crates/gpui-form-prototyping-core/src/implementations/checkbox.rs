use gpui_form_schema::registry::{FieldVariant, GpuiFormShape};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::imports::ImportItem;

use super::{
    FieldCodeGenerator, GeneratedSubscription, ResolvedField, dependent_reset_statements,
    render_standard_field,
};

pub struct CheckboxCodeGenerator;

const IMPORTS: &[ImportItem] = &[ImportItem::path(
    "::gpui_kit::component::checkbox::Checkbox",
)];

impl FieldCodeGenerator for CheckboxCodeGenerator {
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
        // M4 (backlog #12): reset dependent fields when the toggle flips.
        let dependent_resets = dependent_reset_statements(field, component, &format_ident!("v"));
        let field_name_ident = field.field_ident();
        let component_gpui_type = field.component_ident();

        let checkbox_id_str = field.kebab_id();

        render_standard_field(
            field,
            component,
            quote! {
                #component_gpui_type::new(#checkbox_id_str)
                    .checked(self.current_data.#field_name_ident)
                    .on_click(cx.listener(|v, _, _, _| {
                        v.current_data.#field_name_ident = !v.current_data.#field_name_ident;
                        #dependent_resets
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
