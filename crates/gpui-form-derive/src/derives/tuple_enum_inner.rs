use darling::{FromDeriveInput, FromVariant};
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Ident, Type};

#[derive(FromVariant)]
#[darling(attributes(tuple_enum))]
struct VariantArgs {
    ident: Ident,
    fields: darling::ast::Fields<syn::Field>,
    #[darling(default)]
    skip: bool,
}

#[derive(FromDeriveInput)]
#[darling(attributes(tuple_enum), supports(enum_any))]
struct TupleEnumInnerArgs {
    ident: Ident,
    data: darling::ast::Data<VariantArgs, ()>,
}

/// Information about a single variant
struct VariantInfo {
    ident: Ident,
    inner_type: Option<Type>,
    is_unit: bool,
}

pub fn from(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    let args = match TupleEnumInnerArgs::from_derive_input(&input) {
        Ok(args) => args,
        Err(err) => return err.write_errors().into(),
    };

    let enum_ident = &args.ident;

    let variants: Vec<VariantInfo> = match &args.data {
        darling::ast::Data::Enum(variants) => variants
            .iter()
            .filter(|v| !v.skip)
            .map(|v| {
                let inner_type = match &v.fields.style {
                    darling::ast::Style::Tuple => {
                        if v.fields.fields.len() == 1 {
                            Some(v.fields.fields[0].ty.clone())
                        } else if v.fields.fields.is_empty() {
                            None
                        } else {
                            panic!(
                                "TupleEnumInner only supports single-element tuple variants, got {} elements in {}",
                                v.fields.fields.len(),
                                v.ident
                            );
                        }
                    }
                    darling::ast::Style::Unit => None,
                    darling::ast::Style::Struct => {
                        panic!(
                            "TupleEnumInner does not support struct variants: {}",
                            v.ident
                        );
                    }
                };

                let is_unit = inner_type.is_none();
                VariantInfo {
                    ident: v.ident.clone(),
                    inner_type,
                    is_unit,
                }
            })
            .collect(),
        _ => unreachable!("TupleEnumInner only supports enums"),
    };

    // Check if all variants are unit variants
    let all_unit = variants.iter().all(|v| v.is_unit);

    // Generate variants() items - for tuple variants, use Default::default() for inner
    let variant_items: Vec<_> = variants
        .iter()
        .map(|v| {
            let vident = &v.ident;
            if v.is_unit {
                quote! { Self::#vident, }
            } else {
                quote! { Self::#vident(Default::default()), }
            }
        })
        .collect();

    // Generate variant_name() match arms
    let variant_name_arms: Vec<_> = variants
        .iter()
        .map(|v| {
            let vident = &v.ident;
            let name = vident.to_string();
            if v.is_unit {
                quote! { Self::#vident => #name, }
            } else {
                quote! { Self::#vident(_) => #name, }
            }
        })
        .collect();

    // Generate has_inner() match arms
    let has_inner_arms: Vec<_> = variants
        .iter()
        .map(|v| {
            let vident = &v.ident;
            if v.is_unit {
                quote! { Self::#vident => false, }
            } else {
                quote! { Self::#vident(_) => true, }
            }
        })
        .collect();

    // Generate child_variant_names() match arms for heterogeneous enums
    // This returns the variant names of the INNER TYPE, not the inner value's children
    let child_variant_names_arms: Vec<_> = variants
        .iter()
        .map(|v| {
            let vident = &v.ident;
            if v.is_unit {
                quote! { Self::#vident => vec![], }
            } else {
                let inner_ty = v.inner_type.as_ref().unwrap();
                quote! {
                    Self::#vident(_) => {
                        <#inner_ty as gpui_form_component::TupleEnumInner>::variants()
                            .into_iter()
                            .map(|v| v.variant_name())
                            .collect()
                    }
                }
            }
        })
        .collect();

    // Generate inner_child_variant_names() match arms - gets the children of the inner VALUE
    let inner_child_variant_names_arms: Vec<_> = variants
        .iter()
        .map(|v| {
            let vident = &v.ident;
            if v.is_unit {
                quote! { Self::#vident => vec![], }
            } else {
                quote! {
                    Self::#vident(inner) => inner.child_variant_names(),
                }
            }
        })
        .collect();

    // Generate inner_set_child_by_index() match arms - sets child on the inner VALUE
    let inner_set_child_arms: Vec<_> = variants
        .iter()
        .map(|v| {
            let vident = &v.ident;
            if v.is_unit {
                quote! { Self::#vident => None, }
            } else {
                quote! {
                    Self::#vident(inner) => {
                        inner.set_child_by_index(index).map(|new_inner| Self::#vident(new_inner))
                    }
                }
            }
        })
        .collect();

    // Generate inner_has_inner() match arms - checks if the inner VALUE has children
    let inner_has_inner_arms: Vec<_> = variants
        .iter()
        .map(|v| {
            let vident = &v.ident;
            if v.is_unit {
                quote! { Self::#vident => false, }
            } else {
                quote! {
                    Self::#vident(inner) => inner.has_inner(),
                }
            }
        })
        .collect();

    // Generate set_child_by_index() match arms
    let set_child_arms: Vec<_> = variants
        .iter()
        .map(|v| {
            let vident = &v.ident;
            if v.is_unit {
                quote! { Self::#vident => None, }
            } else {
                let inner_ty = v.inner_type.as_ref().unwrap();
                quote! {
                    Self::#vident(_) => {
                        let children = <#inner_ty as gpui_form_component::TupleEnumInner>::variants();
                        children.get(index).map(|child| Self::#vident(child.clone()))
                    }
                }
            }
        })
        .collect();

    // Generate set_child_by_path() match arms - recursive path setting
    let set_child_by_path_arms: Vec<_> = variants
        .iter()
        .map(|v| {
            let vident = &v.ident;
            if v.is_unit {
                quote! { Self::#vident => None, }
            } else {
                let inner_ty = v.inner_type.as_ref().unwrap();
                quote! {
                    Self::#vident(_) => {
                        if path.is_empty() {
                            return None;
                        }
                        let children = <#inner_ty as gpui_form_component::TupleEnumInner>::variants();
                        let child = children.get(path[0])?.clone();
                        if path.len() == 1 {
                            // Last element in path - just set the child
                            Some(Self::#vident(child))
                        } else {
                            // More path elements - recursively set on the child
                            let updated_child = child.set_child_by_path(&path[1..])?;
                            Some(Self::#vident(updated_child))
                        }
                    }
                }
            }
        })
        .collect();

    // Generate child_depth() match arms
    let child_depth_arms: Vec<_> = variants
        .iter()
        .map(|v| {
            let vident = &v.ident;
            if v.is_unit {
                quote! { Self::#vident => 0, }
            } else {
                let inner_ty = v.inner_type.as_ref().unwrap();
                quote! {
                    Self::#vident(_) => <#inner_ty as gpui_form_component::TupleEnumInner>::depth(),
                }
            }
        })
        .collect();

    // For depth calculation, we need to find the max depth among all variants
    let depth_calculation = if all_unit {
        quote! { 1 }
    } else {
        let depth_checks: Vec<_> = variants
            .iter()
            .filter(|v| !v.is_unit)
            .map(|v| {
                let inner_ty = v.inner_type.as_ref().unwrap();
                quote! { <#inner_ty as gpui_form_component::TupleEnumInner>::depth() }
            })
            .collect();
        quote! {
            1 + [#(#depth_checks),*].into_iter().max().unwrap_or(0)
        }
    };

    let expanded = quote! {
        impl gpui_form_component::TupleEnumInner for #enum_ident {
            fn variants() -> Vec<Self> {
                vec![
                    #(#variant_items)*
                ]
            }

            fn variant_name(&self) -> &'static str {
                match self {
                    #(#variant_name_arms)*
                }
            }

            fn has_inner(&self) -> bool {
                match self {
                    #(#has_inner_arms)*
                }
            }

            fn child_variant_names(&self) -> Vec<&'static str> {
                match self {
                    #(#child_variant_names_arms)*
                }
            }

            fn set_child_by_index(&self, index: usize) -> Option<Self> {
                match self {
                    #(#set_child_arms)*
                }
            }

            fn set_child_by_path(&self, path: &[usize]) -> Option<Self> {
                match self {
                    #(#set_child_by_path_arms)*
                }
            }

            fn child_depth(&self) -> usize {
                match self {
                    #(#child_depth_arms)*
                }
            }

            fn depth() -> usize {
                #depth_calculation
            }

            fn inner_child_variant_names(&self) -> Vec<&'static str> {
                match self {
                    #(#inner_child_variant_names_arms)*
                }
            }

            fn inner_set_child_by_index(&self, index: usize) -> Option<Self> {
                match self {
                    #(#inner_set_child_arms)*
                }
            }

            fn inner_has_inner(&self) -> bool {
                match self {
                    #(#inner_has_inner_arms)*
                }
            }
        }
    };

    expanded.into()
}
