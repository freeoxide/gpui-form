//! Prototyping generator for `component(nested)` fields: a bordered group
//! hosting the nested form's own generated scaffold (`<Nested>Form`), resolved
//! by the `<Name>Form` / snake-case-module naming convention.

use gpui_form_schema::{
    components::{ComponentsBehaviour, NestedBehaviour},
    registry::{FieldVariant, GpuiFormShape},
};
use heck::ToSnakeCase as _;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::imports::ImportItem;

use super::{FieldCodeGenerator, ResolvedField};

pub struct NestedCodeGenerator;

impl FieldCodeGenerator for NestedCodeGenerator {
    fn generate_imports(&self, field: &FieldVariant) -> Vec<ImportItem> {
        let Some(NestedBehaviour { form_struct }) = nested_behaviour(field) else {
            return vec![];
        };
        let module = form_struct.to_snake_case();
        vec![ImportItem::owned_path(format!(
            "crate::{module}::{form_struct}Form"
        ))]
    }

    fn generate_cx_new_call(
        &self,
        field: &ResolvedField<'_>,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        let var_ident = nested_form_var(field);
        let form_ident = nested_form_ident(field);
        Some(quote! {
            let #var_ident = cx.new(|cx| #form_ident::new(window, cx));
        })
    }

    fn generate_field_initializers(
        &self,
        _field: &ResolvedField<'_>,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        None
    }

    fn generate_scaffold_struct_field(
        &self,
        field: &ResolvedField<'_>,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        let var_ident = nested_form_var(field);
        let form_ident = nested_form_ident(field);
        Some(quote! { #var_ident: Entity<#form_ident>, })
    }

    fn generate_scaffold_struct_init(
        &self,
        field: &ResolvedField<'_>,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        let var_ident = nested_form_var(field);
        Some(quote! { #var_ident, })
    }

    fn generate_render_child(
        &self,
        field: &ResolvedField<'_>,
        component: &GpuiFormShape,
    ) -> TokenStream {
        let var_ident = nested_form_var(field);
        let label_tokens = super::generate_label_tokens(field, component);

        quote! {
            .child(
                div()
                    .border_1()
                    .rounded_md()
                    .p_2()
                    .child(
                        v_flex()
                            .gap_2()
                            .child(field().label(#label_tokens))
                            .child(#var_ident.clone()),
                    ),
            )
        }
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
    ) -> Option<super::GeneratedSubscription> {
        None
    }
}

fn nested_behaviour(field: &FieldVariant) -> Option<NestedBehaviour> {
    match &field.behaviour {
        ComponentsBehaviour::Nested(behaviour) => Some(*behaviour),
        _ => None,
    }
}

fn nested_form_var(field: &ResolvedField<'_>) -> proc_macro2::Ident {
    format_ident!("{}_nested_form", field.field_name())
}

fn nested_form_ident(field: &ResolvedField<'_>) -> proc_macro2::Ident {
    let form_struct = nested_behaviour(field.raw())
        .map(|behaviour| behaviour.form_struct.to_string())
        .unwrap_or_else(|| field.raw().value_type.to_string());
    format_ident!("{form_struct}Form")
}
