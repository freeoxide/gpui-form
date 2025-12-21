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

        use __crate_paths::gpui::{AppContext, Context, Entity, Window};
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

        let child_selects_field_name = quote::format_ident!("{}_child_selects", name);

        // Generate field structure definitions
        let field_structure_definition = quote! {
            /// The master select for choosing the top-level variant
            pub #master_field_name: #Entity<#master_state_type>,
            /// The dynamic list of child selects for nested variants
            pub #child_selects_field_name: Vec<#Entity<#master_state_type>>,
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

            /// Rebuilds the child selects for a given parent value.
            /// This iterates through the depth of the structure and creates selects for each level
            /// where children exist.
            pub fn #child_selects_field_name<V>(
                parent: &#r#type,
                start_level: usize,
                window: &mut #Window,
                cx: &mut #Context<V>
            ) -> Vec<#Entity<#master_state_type>>
            where V: 'static
            {
                use gpui_form_component::{TupleEnumInner, TupleSelectItem};
                use #SelectState;
                use #IndexPath;
                use #AppContext;

                let max_depth = <#r#type as TupleEnumInner>::depth();
                let mut current_value = parent.clone();
                let mut selects = Vec::new();

                // Skip to the start level
                // Note: This logic assumes we are rebuilding from start_level onwards,
                // but we need the 'current_value' to be correct at that level.
                // For a proper implementation, we'd need the path to descend correctly.
                // However, without the full path, we can only easily rebuild from 0 or
                // if we assume 'parent' IS the value at 'start_level'?
                //
                // In the manual implementation, 'parent' is the value selected at the previous level.
                // So if we are rebuilding level 1 (children of master), parent is the Master value.
                // If we are rebuilding level 2, parent is the L1 value?
                //
                // The manual impl: 'current_value = parent.clone()' and 'for level in start_level..'
                // This implies 'parent' is the root object if start_level is 0.
                // If start_level > 0, 'parent' should probably be the object at that level?
                //
                // Actually, the manual impl in location.rs:294 iterates from start_level.
                // But it does `if level == 0 { ... } else { ... }` using `current_value`.
                // `current_value` starts as `parent`.
                // So `parent` MUST be the root object (Country).

                for level in start_level..(max_depth - 1) {
                    let (child_names, has_more) = if level == 0 {
                        (current_value.child_variant_names(), current_value.has_inner())
                    } else {
                        (current_value.inner_child_variant_names(), current_value.inner_has_inner())
                    };

                    if !has_more || child_names.is_empty() {
                        break;
                    }

                    // Create items for this level
                    let items: Vec<TupleSelectItem<#r#type>> = child_names
                        .iter()
                        .enumerate()
                        .filter_map(|(idx, name)| {
                            let variant = if level == 0 {
                                current_value.set_child_by_index(idx)
                            } else {
                                current_value.inner_set_child_by_index(idx)
                            };
                            variant.map(|v| TupleSelectItem::new(v, name.to_string()))
                        })
                        .collect();

                    if items.is_empty() {
                        break;
                    }

                    // Default to first item selected
                    let selected_index = Some(IndexPath {
                        section: 0,
                        row: 0,
                        column: 0,
                    });

                    let child_select = cx.new(|cx| SelectState::new(items.clone(), selected_index, window, cx));
                    selects.push(child_select);

                    // Move to the first child for the next iteration
                    if let Some(first_item) = items.first() {
                        current_value = first_item.get_value().clone();
                    } else {
                        break;
                    }
                }

                selects
            }
        };

        field_structure_tokens.extend(field_structure_definition);
        field_base_declarations_tokens.extend(field_base_declaration);
    }
}
