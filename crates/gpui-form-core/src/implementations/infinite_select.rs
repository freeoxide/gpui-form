use super::__crate_paths;
use crate::components::*;
use proc_macro2::TokenStream;
use quote::quote;

impl super::ComponentLayout for InfiniteSelectComponent {
    fn field_tokens(
        &self,
        field_structure_tokens: &mut TokenStream,
        field_base_declarations_tokens: &mut TokenStream,
    ) {
        let FieldInformation::<InfiniteSelectOptions> {
            options,
            name,
            r#type,
        } = &self.0;

        let master_field_name = quote::format_ident!("{}_master_select", name);
        let path_field_name = quote::format_ident!("{}_path", name);

        use __crate_paths::gpui::{AppContext, Context, Entity, Window};
        use __crate_paths::gpui_component::IndexPath;
        use __crate_paths::gpui_component::select::{SearchableVec, SelectState};

        let searchable = options.behaviour.searchable;

        let vec_type = if searchable {
            quote! { #SearchableVec }
        } else {
            quote! { Vec }
        };

        let master_state_type = quote! {
            #SelectState<#vec_type<::gpui_form::infinite_select::InfiniteSelectItem<#r#type>>>
        };

        let child_selects_field_name = quote::format_ident!("{}_child_selects", name);

        // Generate field structure definitions
        let field_structure_definition = quote! {
            /// The master select for choosing the top-level variant
            pub #master_field_name: #Entity<#master_state_type>,
            /// The dynamic list of child selects for nested variants
            pub #child_selects_field_name: Vec<#Entity<#master_state_type>>,
            /// The selection path tracking all levels of the hierarchy
            pub #path_field_name: ::gpui_form::infinite_select::InfiniteSelectPath,
        };

        // Generate initialization methods
        let index = if let Some(default_expr) = options.field_default() {
            let default_expr = default_expr.clone();
            quote! {
                {
                    let __gpui_form_default = #default_expr;
                    Some(
                        #IndexPath::new(
                            <#r#type as ::gpui_form::infinite_select::InfiniteSelect>::variants()
                                .iter()
                                .position(|x| x.variant_name() == __gpui_form_default.variant_name())
                                .unwrap()
                        )
                    )
                }
            }
        } else if options.use_enum_default() {
            quote! {
                Some(#IndexPath::new(0))
            }
        } else {
            quote! { None }
        };

        let max_depth_expr = if let Some(max_depth) = options.behaviour.max_depth {
            quote! {
                ::core::cmp::max(
                    1usize,
                    ::core::cmp::min(
                        <#r#type as InfiniteSelect>::depth(),
                        #max_depth
                    )
                )
            }
        } else {
            quote! { <#r#type as InfiniteSelect>::depth() }
        };

        let field_base_declaration = quote! {
            /// Initialize the master select for the infinite select enum outer variants
            pub fn #master_field_name(window: &mut #Window, cx: &mut #Context<'_, #master_state_type>) -> #master_state_type {
                let items: Vec<::gpui_form::infinite_select::InfiniteSelectItem<#r#type>> =
                    ::gpui_form::infinite_select::to_select_items::<#r#type>();

                #SelectState::new(items.into(), #index, window, cx)
            }

            /// Get the child variant names for a given parent value.
            /// Returns the names of variants available at the next level.
            pub fn #path_field_name(parent: &#r#type) -> Vec<&'static str> {
                use ::gpui_form::infinite_select::InfiniteSelect as _;
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
                use ::gpui_form::infinite_select::{InfiniteSelect, InfiniteSelectItem};
                use #SelectState;
                use #IndexPath;
                use #AppContext;

                let max_depth = #max_depth_expr;
                let mut current_value = parent.clone();
                let mut selects = Vec::new();

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
                    let items: Vec<InfiniteSelectItem<#r#type>> = child_names
                        .iter()
                        .enumerate()
                        .filter_map(|(idx, name)| {
                            let variant = if level == 0 {
                                current_value.set_child_by_index(idx)
                            } else {
                                current_value.inner_set_child_by_index(idx)
                            };
                            variant.map(|v| InfiniteSelectItem::new(v, name.to_string()))
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
