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
#[darling(attributes(tuple_enum), forward_attrs(fluent_kv), supports(enum_any))]
struct InfiniteSelectArgs {
    ident: Ident,

    data: darling::ast::Data<VariantArgs, ()>,

    attrs: Vec<syn::Attribute>,
}

/// Information about a single variant
struct VariantInfo {
    ident: Ident,

    inner_type: Option<Type>,

    /// For struct variants, the name of the field (e.g., `state` in `USA { state: USAState }`)
    field_name: Option<Ident>,

    is_unit: bool,
}

impl VariantInfo {
    /// Generate a match pattern that ignores the inner value (for patterns where we don't need the value)
    /// Returns `Variant(_)` for tuple or `Variant { field: _ }` for struct
    fn ignore_pattern(&self) -> proc_macro2::TokenStream {
        let vident = &self.ident;
        if self.is_unit {
            quote! { Self::#vident }
        } else if let Some(fname) = &self.field_name {
            quote! { Self::#vident { #fname: _ } }
        } else {
            quote! { Self::#vident(_) }
        }
    }

    /// Generate a match pattern that binds the inner value to `inner`
    /// Returns `Variant(inner)` for tuple or `Variant { field: inner }` for struct
    fn binding_pattern(&self) -> proc_macro2::TokenStream {
        let vident = &self.ident;
        if self.is_unit {
            quote! { Self::#vident }
        } else if let Some(fname) = &self.field_name {
            quote! { Self::#vident { #fname: inner } }
        } else {
            quote! { Self::#vident(inner) }
        }
    }

    /// Generate a constructor expression with a value
    /// Returns `Variant(value)` for tuple or `Variant { field: value }` for struct
    fn constructor(&self, value: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        let vident = &self.ident;
        if self.is_unit {
            quote! { Self::#vident }
        } else if let Some(fname) = &self.field_name {
            quote! { Self::#vident { #fname: #value } }
        } else {
            quote! { Self::#vident(#value) }
        }
    }
}

pub fn from(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    let args = match InfiniteSelectArgs::from_derive_input(&input) {
        Ok(args) => args,

        Err(err) => return err.write_errors().into(),
    };

    let enum_ident = &args.ident;

    // Parse fluent_kv attribute to find keys
    let mut has_label = false;

    let mut has_description = false;

    let mut keys_this = false;

    for attr in &args.attrs {
        if attr.path().is_ident("fluent_kv") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("keys") {
                    let value = meta.value()?;

                    let list: syn::ExprArray = value.parse()?;

                    for elem in list.elems {
                        if let syn::Expr::Lit(lit) = elem
                            && let syn::Lit::Str(s) = lit.lit
                        {
                            if s.value() == "label" {
                                has_label = true;
                            }

                            if s.value() == "description" {
                                has_description = true;
                            }
                        }
                    }
                } else if meta.path.is_ident("keys_this") {
                    keys_this = true;
                }

                Ok(())
            });
        }
    }

    let type_label_impl = if has_label && keys_this {
        let label_enum = quote::format_ident!("{}LabelVariants", enum_ident);

        quote! {

            use es_fluent::ThisFtl as _;

            #label_enum::this_ftl().into()

        }
    } else {
        quote! { stringify!(#enum_ident).into() }
    };

    let type_description_impl = if has_description && keys_this {
        let desc_enum = quote::format_ident!("{}DescriptionVariants", enum_ident);

        quote! {

            use es_fluent::ThisFtl as _;

            #desc_enum::this_ftl().into()

        }
    } else {
        quote! { stringify!(#enum_ident).into() }
    };

    let variants: Result<Vec<VariantInfo>, syn::Error> = match &args.data {
        darling::ast::Data::Enum(variants) => variants
            .iter()
            .filter(|v| !v.skip)
            .map(|v| {
                let (inner_type, field_name) = match &v.fields.style {
                    darling::ast::Style::Tuple => {
                        if v.fields.fields.len() == 1 {
                            (Some(v.fields.fields[0].ty.clone()), None)
                        } else if v.fields.fields.is_empty() {
                            (None, None)
                        } else {
                            return Err(syn::Error::new_spanned(
                                &v.ident,
                                format!(
                                    "InfiniteSelect only supports single-element tuple variants for tree construction, got {} elements in variant `{}`",
                                    v.fields.fields.len(),
                                    v.ident
                                ),
                            ));
                        }
                    }
                    darling::ast::Style::Unit => (None, None),
                    darling::ast::Style::Struct => {
                        if v.fields.fields.len() == 1 {
                            let field = &v.fields.fields[0];
                            let fname = field.ident.clone().expect("Struct field must have a name");
                            (Some(field.ty.clone()), Some(fname))
                        } else if v.fields.fields.is_empty() {
                            (None, None)
                        } else {
                            return Err(syn::Error::new_spanned(
                                &v.ident,
                                format!(
                                    "InfiniteSelect only supports single-field struct variants for tree construction, got {} fields in variant `{}`",
                                    v.fields.fields.len(),
                                    v.ident
                                ),
                            ));
                        }
                    }
                };

                let is_unit = inner_type.is_none();
                Ok(VariantInfo {
                    ident: v.ident.clone(),
                    inner_type,
                    field_name,
                    is_unit,
                })
            })
            .collect(),
        _ => unreachable!("InfiniteSelect only supports enums"),
    };

    let variants = match variants {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };

    // Check if all variants are unit variants
    let all_unit = variants.iter().all(|v| v.is_unit);

    // Generate variants() items - for tuple/struct variants, use Default::default() for inner
    let variant_items: Vec<_> = variants
        .iter()
        .map(|v| {
            let constructor = v.constructor(quote! { Default::default() });
            quote! { #constructor, }
        })
        .collect();

    // Generate variant_name() match arms
    let variant_name_arms: Vec<_> = variants
        .iter()
        .map(|v| {
            let pattern = v.ignore_pattern();
            let name = v.ident.to_string();
            quote! { #pattern => #name, }
        })
        .collect();

    // Generate has_inner() match arms
    let has_inner_arms: Vec<_> = variants
        .iter()
        .map(|v| {
            let pattern = v.ignore_pattern();
            let has = !v.is_unit;
            quote! { #pattern => #has, }
        })
        .collect();

    // Generate child_variant_names() match arms for heterogeneous enums
    // This returns the variant names of the INNER TYPE, not the inner value's children
    let child_variant_names_arms: Vec<_> = variants
        .iter()
        .map(|v| {
            let pattern = v.ignore_pattern();
            if v.is_unit {
                quote! { #pattern => vec![], }
            } else {
                let inner_ty = v.inner_type.as_ref().unwrap();
                quote! {
                    #pattern => {
                        <#inner_ty as gpui_form_component::infinite_select::InfiniteSelect>::variants()
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
            let pattern = v.binding_pattern();
            if v.is_unit {
                let ignore_pattern = v.ignore_pattern();
                quote! { #ignore_pattern => vec![], }
            } else {
                quote! {
                    #pattern => inner.child_variant_names(),
                }
            }
        })
        .collect();

    // Generate inner_set_child_by_index() match arms - sets child on the inner VALUE
    let inner_set_child_arms: Vec<_> = variants
        .iter()
        .map(|v| {
            if v.is_unit {
                let pattern = v.ignore_pattern();
                quote! { #pattern => None, }
            } else {
                let pattern = v.binding_pattern();
                let constructor = v.constructor(quote! { new_inner });
                quote! {
                    #pattern => {
                        inner.set_child_by_index(index).map(|new_inner| #constructor)
                    }
                }
            }
        })
        .collect();

    // Generate inner_has_inner() match arms - checks if the inner VALUE has children
    let inner_has_inner_arms: Vec<_> = variants
        .iter()
        .map(|v| {
            if v.is_unit {
                let pattern = v.ignore_pattern();
                quote! { #pattern => false, }
            } else {
                let pattern = v.binding_pattern();
                quote! {
                    #pattern => inner.has_inner(),
                }
            }
        })
        .collect();

    // Generate set_child_by_index() match arms
    let set_child_arms: Vec<_> = variants
        .iter()
        .map(|v| {
            if v.is_unit {
                let pattern = v.ignore_pattern();
                quote! { #pattern => None, }
            } else {
                let pattern = v.ignore_pattern();
                let inner_ty = v.inner_type.as_ref().unwrap();
                let constructor = v.constructor(quote! { child.clone() });
                quote! {
                    #pattern => {
                        let children = <#inner_ty as gpui_form_component::infinite_select::InfiniteSelect>::variants();
                        children.get(index).map(|child| #constructor)
                    }
                }
            }
        })
        .collect();

    // Generate set_child_by_path() match arms - recursive path setting
    let set_child_by_path_arms: Vec<_> = variants
        .iter()
        .map(|v| {
            if v.is_unit {
                let pattern = v.ignore_pattern();
                quote! { #pattern => None, }
            } else {
                let pattern = v.ignore_pattern();
                let inner_ty = v.inner_type.as_ref().unwrap();
                let constructor_child = v.constructor(quote! { child });
                let constructor_updated = v.constructor(quote! { updated_child });
                quote! {
                    #pattern => {
                        if path.is_empty() {
                            return None;
                        }
                        let children = <#inner_ty as gpui_form_component::infinite_select::InfiniteSelect>::variants();
                        let child = children.get(path[0])?.clone();
                        if path.len() == 1 {
                            // Last element in path - just set the child
                            Some(#constructor_child)
                        } else {
                            // More path elements - recursively set on the child
                            let updated_child = child.set_child_by_path(&path[1..])?;
                            Some(#constructor_updated)
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
            let pattern = v.ignore_pattern();
            if v.is_unit {
                quote! { #pattern => 0, }
            } else {
                let inner_ty = v.inner_type.as_ref().unwrap();
                quote! {
                    #pattern => <#inner_ty as gpui_form_component::infinite_select::InfiniteSelect>::depth(),
                }
            }
        })
        .collect();

    let inner_child_label_arms: Vec<_> = variants
        .iter()
        .map(|v| {
            if v.is_unit {
                let pattern = v.ignore_pattern();
                quote! { #pattern => None, }
            } else {
                let pattern = v.binding_pattern();
                quote! { #pattern => inner.child_label_at_depth(depth), }
            }
        })
        .collect();

    let inner_child_description_arms: Vec<_> = variants
        .iter()
        .map(|v| {
            if v.is_unit {
                let pattern = v.ignore_pattern();
                quote! { #pattern => None, }
            } else {
                let pattern = v.binding_pattern();
                quote! { #pattern => inner.child_description_at_depth(depth), }
            }
        })
        .collect();

    let child_label_immediate_arms: Vec<_> = variants
        .iter()
        .map(|v| {
            if v.is_unit {
                let pattern = v.ignore_pattern();
                quote! { #pattern => None, }
            } else {
                let pattern = v.binding_pattern();
                quote! { #pattern => Some(inner.type_label()), }
            }
        })
        .collect();

    let child_description_immediate_arms: Vec<_> = variants
        .iter()
        .map(|v| {
            if v.is_unit {
                let pattern = v.ignore_pattern();
                quote! { #pattern => None, }
            } else {
                let pattern = v.binding_pattern();
                quote! { #pattern => Some(inner.type_description()), }
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
                quote! { <#inner_ty as gpui_form_component::infinite_select::InfiniteSelect>::depth() }
            })
            .collect();
        quote! {
            1 + [#(#depth_checks),*].into_iter().max().unwrap_or(0)
        }
    };

    let expanded = quote! {
        impl gpui_form_component::infinite_select::InfiniteSelect for #enum_ident {
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

            fn type_label(&self) -> gpui::SharedString {
                #type_label_impl
            }

            fn type_description(&self) -> gpui::SharedString {
                #type_description_impl
            }

            fn inner_child_label_at_depth(&self, depth: usize) -> Option<gpui::SharedString> {
                match self {
                    #(#inner_child_label_arms)*
                }
            }

            fn inner_child_description_at_depth(&self, depth: usize) -> Option<gpui::SharedString> {
                match self {
                    #(#inner_child_description_arms)*
                }
            }

            fn child_label_at_depth(&self, depth: usize) -> Option<gpui::SharedString> {
                if depth == 0 {
                    match self {
                        #(#child_label_immediate_arms)*
                    }
                } else {
                    self.inner_child_label_at_depth(depth - 1)
                }
            }

            fn child_description_at_depth(&self, depth: usize) -> Option<gpui::SharedString> {
                if depth == 0 {
                    match self {
                        #(#child_description_immediate_arms)*
                    }
                } else {
                    self.inner_child_description_at_depth(depth - 1)
                }
            }
        }
    };

    expanded.into()
}
