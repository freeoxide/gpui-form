use super::__crate_paths;
use crate::components::*;
use proc_macro2::TokenStream;
use quote::quote;

impl super::ComponentLayout for TupleSelectComponent {
    fn field_tokens(
        &self,
        field_structure_tokens: &mut TokenStream,
        field_base_declarations_tokens: &mut TokenStream,
    ) {
        let FieldInformation::<TupleSelectOptions> {
            options,
            name,
            r#type,
            item_type: _,
        } = &self.0;

        // Generate field names for master select and selection path
        let master_field_name = quote::format_ident!("{}_master_select", name);
        let path_field_name = quote::format_ident!("{}_path", name);

        use __crate_paths::gpui::{Context, Entity, Window};
        use __crate_paths::gpui_component::IndexPath;
        use __crate_paths::gpui_component::select::{SearchableVec, SelectState};

        // For TupleSelect, we generate:
        // 1. A master select for the outer enum variants
        // 2. A TupleSelectPath to track the full selection through the hierarchy
        //
        // Child selects are created dynamically based on the current master selection.

        let searchable = options.behaviour.searchable;

        let vec_type = if searchable {
            quote! { #SearchableVec }
        } else {
            quote! { Vec }
        };

        // Master select state type - uses TupleSelectItem wrapper
        let master_state_type = quote! {
            #SelectState<#vec_type<gpui_form_component::TupleSelectItem<#r#type>>>
        };

        // Generate field structure definitions
        let field_structure_definition = quote! {
            /// The master select for choosing the top-level variant
            pub #master_field_name: #Entity<#master_state_type>,
            /// The selection path tracking all levels of the hierarchy
            pub #path_field_name: gpui_form_component::TupleSelectPath,
        };

        // Generate initialization methods
        let index = if let Some(named_index) = options.named_index() {
            let path = named_index.clone();
            quote! {
                Some(
                    #IndexPath::new(
                        <#r#type as gpui_form_component::TupleEnumInner>::variants()
                            .iter()
                            .position(|x| x.variant_name() == #path.variant_name())
                            .unwrap()
                    )
                )
            }
        } else if options.index_default() {
            quote! {
                Some(#IndexPath::new(0))
            }
        } else {
            quote! { None }
        };

        let field_base_declaration = quote! {
            /// Initialize the master select for the tuple enum outer variants
            pub fn #master_field_name(window: &mut #Window, cx: &mut #Context<'_, #master_state_type>) -> #master_state_type {
                let items: Vec<gpui_form_component::TupleSelectItem<#r#type>> =
                    gpui_form_component::tuple_enum_to_select_items::<#r#type>();

                #SelectState::new(items.into(), #index, window, cx)
            }

            /// Get the child variant names for a given parent value.
            /// Returns the names of variants available at the next level.
            pub fn #path_field_name(parent: &#r#type) -> Vec<&'static str> {
                use gpui_form_component::TupleEnumInner as _;
                parent.child_variant_names()
            }
        };

        field_structure_tokens.extend(field_structure_definition);
        field_base_declarations_tokens.extend(field_base_declaration);
    }
}
