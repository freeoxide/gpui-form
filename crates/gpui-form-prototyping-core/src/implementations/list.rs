//! Prototyping generator for `component(list(item = ...))` fields: repeatable
//! rows over the holder's generated `<field>_items()` / `add_<field>` /
//! `remove_<field>` operations; non-`Vec<T>` value types error.

use gpui_form_schema::components::ComponentsBehaviour;
use gpui_form_schema::registry::{FieldVariant, GpuiFormShape};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Type;

use crate::error::{PrototypingError, PrototypingResult};
use crate::imports::ImportItem;

use super::{FieldCodeGenerator, ResolvedField};

pub struct ListCodeGenerator;

const IMPORTS: &[ImportItem] = &[
    ImportItem::path("::gpui_kit::component::button::Button"),
    ImportItem::path("::gpui_kit::component::h_flex"),
];

impl FieldCodeGenerator for ListCodeGenerator {
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

    fn generate_impl_members(
        &self,
        field: &ResolvedField<'_>,
        component: &GpuiFormShape,
    ) -> PrototypingResult<Vec<TokenStream>> {
        let control_ident = format_ident!("{}_list_control", field.field_name());
        let field_name_str = field.field_name();
        let items_fn = format_ident!("{}_items", field_name_str);
        let add_fn = format_ident!("add_{}", field_name_str);
        let remove_fn = format_ident!("remove_{}", field_name_str);
        let row_button_prefix = format!("{field_name_str}-remove");
        let add_button_id = format!("{field_name_str}-add");
        let item_type = list_item_type(field)?;
        let group_label = super::generate_label_tokens(field, component);
        let item_label = super::localized_text_tokens(field, component, "item_label");
        let add_label = match add_label_override(field) {
            Some(label) => quote! { #label.to_string() },
            None => super::localized_text_tokens_with_fallback(field, component, "add", "Add"),
        };
        let remove_label =
            super::localized_text_tokens_with_fallback(field, component, "remove", "Remove");

        Ok(vec![quote! {
            /// Repeatable-row control for this list field: one row per current
            /// item (Remove button each) plus an Add button appending a default
            /// item. Generated scaffold; edit freely.
            fn #control_ident(&self, cx: &mut Context<Self>) -> impl IntoElement {
                let mut rows = ::std::vec::Vec::new();
                for (index, item) in self.current_data.#items_fn().iter().enumerate() {
                    let remove_id = format!("{}-{}", #row_button_prefix, index);
                    rows.push(
                        h_flex()
                            .justify_between()
                            .child(
                                field()
                                    .label(format!("{} {}", #item_label, index + 1))
                                    .child(format!("{:?}", item)),
                            )
                            .child(
                                Button::new(remove_id)
                                    .label(#remove_label.clone())
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.current_data.#remove_fn(index);
                                        cx.notify();
                                    })),
                            ),
                    );
                }
                v_flex()
                    .gap_2()
                    .child(field().label(#group_label.clone()))
                    .children(rows)
                    .child(
                        Button::new(#add_button_id)
                            .label(#add_label.clone())
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.current_data.#add_fn(
                                    <#item_type as ::core::default::Default>::default(),
                                );
                                cx.notify();
                            })),
                    )
            }
        }])
    }

    fn generate_render_child(
        &self,
        field: &ResolvedField<'_>,
        _component: &GpuiFormShape,
    ) -> TokenStream {
        let control_ident = format_ident!("{}_list_control", field.field_name());
        quote! {
            .child(self.#control_ident(cx))
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

fn list_item_type(field: &ResolvedField<'_>) -> PrototypingResult<Type> {
    let value_type = field.value_type();
    let Type::Path(type_path) = value_type else {
        return Err(invalid_list_value_type(field));
    };
    let last_segment = type_path.path.segments.last();
    if let Some(segment) = last_segment
        && segment.ident == "Vec"
        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
    {
        return Ok(inner.clone());
    }
    Err(invalid_list_value_type(field))
}

fn invalid_list_value_type(field: &ResolvedField<'_>) -> PrototypingError {
    PrototypingError::InvalidListValueType {
        field_name: field.field_name().to_string(),
        value: field.raw().value_type.to_string(),
    }
}

fn add_label_override(field: &ResolvedField<'_>) -> Option<&'static str> {
    match field.behaviour() {
        ComponentsBehaviour::List(behaviour) => behaviour.add_label,
        _ => None,
    }
}
