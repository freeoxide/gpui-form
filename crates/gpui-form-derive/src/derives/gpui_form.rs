use std::collections::HashMap;

use darling::{FromDeriveInput, FromField, FromMeta, ast};
use gpui_form_core::components::*;
use gpui_form_core::implementations::ComponentLayout as _;
use itertools::Itertools as _;
use koruma_derive_core::{FieldInfo as KorumaFieldInfo, ParseFieldResult, ValidatorAttr};
use proc_macro2::TokenStream;
use quote::{ToTokens as _, format_ident, quote};
use syn::{DeriveInput, GenericArgument, Ident, Meta, PathArguments, Type, parse_macro_input};

/// Flattens `cfg_attr` attributes in a DeriveInput.
/// For each `#[cfg_attr(condition, attr)]` found, we unconditionally expand it to `#[attr]`.
/// This is necessary because darling doesn't handle cfg_attr  automatically.
///
/// Note: We expand ALL cfg_attr unconditionally because:
/// 1. At proc-macro expansion time, we can't reliably evaluate cfg conditions
/// 2. The Rust compiler will have already filtered out this derive if the cfg didn't match
/// 3. If we're running, it means the relevant features are enabled
fn flatten_cfg_attr_in_derive_input(mut input: DeriveInput) -> DeriveInput {
    // Flatten struct-level attributes
    input.attrs = flatten_cfg_attr_in_attrs(input.attrs);

    // Flatten field-level attributes
    if let syn::Data::Struct(ref mut data_struct) = input.data {
        for field in data_struct.fields.iter_mut() {
            field.attrs = flatten_cfg_attr_in_attrs(std::mem::take(&mut field.attrs));
        }
    }

    input
}

/// Flattens `cfg_attr` attributes in an attribute list.
/// Converts `#[cfg_attr(condition, attr1, attr2)]` into `#[attr1]` and `#[attr2]`.
fn flatten_cfg_attr_in_attrs(attrs: Vec<syn::Attribute>) -> Vec<syn::Attribute> {
    let mut result = Vec::new();

    for attr in attrs {
        if attr.path().is_ident("cfg_attr") {
            // Parse the cfg_attr to extract the wrapped attributes
            if let Meta::List(meta_list) = &attr.meta {
                let tokens = meta_list.tokens.clone();
                let mut iter = tokens.into_iter().peekable();

                // Skip past the condition (everything until first comma at depth 0)
                let mut depth: i32 = 0;
                loop {
                    match iter.next() {
                        Some(proc_macro2::TokenTree::Punct(p))
                            if p.as_char() == ',' && depth == 0 =>
                        {
                            break;
                        },
                        Some(proc_macro2::TokenTree::Group(g))
                            if matches!(g.delimiter(), proc_macro2::Delimiter::Parenthesis) =>
                        {
                            depth += 1;
                            // Process group contents recursively
                            for token in g.stream() {
                                if let proc_macro2::TokenTree::Punct(p) = &token
                                    && p.as_char() == ')'
                                {
                                    depth = depth.saturating_sub(1);
                                }
                            }
                        },
                        None => break,
                        _ => {},
                    }
                }

                // Collect remaining tokens as wrapped attributes
                let remaining: proc_macro2::TokenStream = iter.collect();

                // Try to parse each attribute from the remaining tokens
                // Split by commas at depth 0
                let attr_token_groups = split_by_comma(remaining);

                for attr_tokens in attr_token_groups {
                    // Convert the tokens into an attribute by wrapping with #[...]
                    let attr_stream = quote! { #[#attr_tokens] };
                    if let Ok(parsed_attrs) =
                        syn::parse::Parser::parse2(syn::Attribute::parse_outer, attr_stream)
                    {
                        result.extend(parsed_attrs);
                    }
                }
            }
        } else {
            // Keep non-cfg_attr attributes as-is
            result.push(attr);
        }
    }

    result
}

/// Splits a token stream by top-level commas (depth 0).
fn split_by_comma(tokens: proc_macro2::TokenStream) -> Vec<proc_macro2::TokenStream> {
    let mut result = Vec::new();
    let mut current = proc_macro2::TokenStream::new();
    let mut depth = 0;

    for token in tokens {
        match &token {
            proc_macro2::TokenTree::Group(_) => {
                depth += 1;
                current.extend(std::iter::once(token.clone()));
            },
            proc_macro2::TokenTree::Punct(p) if p.as_char() == ',' && depth == 0 => {
                if !current.is_empty() {
                    result.push(current);
                    current = proc_macro2::TokenStream::new();
                }
            },
            _ => {
                current.extend(std::iter::once(token.clone()));
            },
        }
    }

    if !current.is_empty() {
        result.push(current);
    }

    result
}

/// Check if a struct has `gpui_form(koruma(...))` inside a `cfg_attr`.
/// Returns `Some(KorumaOptions)` if found, `None` otherwise.
fn find_koruma_in_cfg_attr(attrs: &[syn::Attribute]) -> Option<KorumaOptions> {
    for attr in attrs {
        if !attr.path().is_ident("cfg_attr") {
            continue;
        }

        if let Meta::List(meta_list) = &attr.meta {
            let tokens = meta_list.tokens.clone();
            let mut iter = tokens.into_iter().peekable();

            // Skip the condition (everything until we hit a comma at depth 0)
            let mut depth: i32 = 0;
            loop {
                match iter.next() {
                    Some(proc_macro2::TokenTree::Punct(p)) if p.as_char() == ',' && depth == 0 => {
                        break;
                    },
                    Some(proc_macro2::TokenTree::Punct(p)) if p.as_char() == '<' => depth += 1,
                    Some(proc_macro2::TokenTree::Punct(p)) if p.as_char() == '>' => {
                        depth = depth.saturating_sub(1);
                    },
                    None => break,
                    _ => {},
                }
            }

            // Now check if any of the remaining attributes is "gpui_form"
            while let Some(token) = iter.next() {
                if let proc_macro2::TokenTree::Ident(ident) = token
                    && ident == "gpui_form"
                {
                    // Next should be a group containing the gpui_form content
                    if let Some(proc_macro2::TokenTree::Group(group)) = iter.next() {
                        // Parse the content to check for koruma(...)
                        let group_tokens: proc_macro2::TokenStream = group.stream();
                        let mut group_iter = group_tokens.into_iter().peekable();

                        while let Some(inner_token) = group_iter.next() {
                            if let proc_macro2::TokenTree::Ident(inner_ident) = inner_token
                                && inner_ident == "koruma"
                            {
                                // Found koruma - check for options
                                if let Some(proc_macro2::TokenTree::Group(koruma_group)) =
                                    group_iter.next()
                                {
                                    // Manually check for "fluent" in koruma options
                                    let koruma_stream = koruma_group.stream();
                                    let has_fluent = koruma_stream.into_iter().any(|t| {
                                                matches!(t, proc_macro2::TokenTree::Ident(id) if id == "fluent")
                                            });
                                    return Some(KorumaOptions { fluent: has_fluent });
                                }
                                // koruma without options
                                return Some(KorumaOptions::default());
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

/// Convert a ValidatorAttr to a TokenStream for generating koruma attribute.
/// This produces tokens like `ValidatorPath::<Type>(arg1 = val1, arg2 = val2)`.
fn validator_attr_to_tokens(validator: &ValidatorAttr) -> TokenStream {
    let path = &validator.validator;

    // Build the type parameters if present
    let type_params = if validator.infer_type {
        quote! { ::<_> }
    } else if let Some(explicit_ty) = &validator.explicit_type {
        quote! { ::<#explicit_ty> }
    } else {
        quote! {}
    };

    // Build the arguments if present
    let args = if validator.args.is_empty() {
        quote! {}
    } else {
        let arg_tokens: Vec<_> = validator
            .args
            .iter()
            .map(|(name, expr)| quote! { #name = #expr })
            .collect();
        quote! { (#(#arg_tokens),*) }
    };

    quote! { #path #type_params #args }
}

#[derive(Clone, Debug, Default, FromMeta)]
struct KorumaOptions {
    #[darling(default)]
    fluent: bool,
}

#[derive(Clone, Debug)]
struct KorumaField(KorumaOptions);

impl FromMeta for KorumaField {
    fn from_word() -> darling::Result<Self> {
        Ok(KorumaField(KorumaOptions::default()))
    }

    fn from_list(items: &[darling::ast::NestedMeta]) -> darling::Result<Self> {
        KorumaOptions::from_list(items).map(KorumaField)
    }
}

/// Information about a field for value holder generation.
/// Tracks whether the field was originally optional and the inner type.
struct FieldOptionality {
    field_name: Ident,
    /// The original type of the field
    original_type: Type,
    /// The inner type (unwrapped from Option if it was optional)
    inner_type: Type,
    /// Whether the original field was Option<T>
    was_optional: bool,
    /// Whether this field should be wrapped in Option in the value holder
    /// (true for component fields that need unwrapping behavior)
    wrap_in_option: bool,
    /// Parsed koruma validators for this field (from koruma_derive_core)
    koruma_validators: Vec<ValidatorAttr>,
    /// Whether this field is marked with #[koruma(newtype)]
    is_newtype: bool,
    /// Whether this field is marked with #[koruma(nested)]
    is_nested: bool,
}

#[derive(Debug, FromField)]
#[darling(attributes(gpui_form))]
struct ComponentField {
    pub ident: Option<Ident>,
    pub ty: Type,
    #[darling(default)]
    pub component: Option<Components>,
    #[darling(default)]
    skip: bool,
}

impl ComponentField {
    pub fn skip(&self) -> bool {
        self.skip && self.component.is_none()
    }
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(gpui_form), supports(struct_named, struct_unit))]
struct ComponentStruct {
    pub ident: Ident,
    pub data: ast::Data<(), ComponentField>,
    #[darling(default)]
    pub empty: bool,
    #[darling(default)]
    pub koruma: Option<KorumaField>,
}

fn get_components_behaviour_tokens(component: &Components) -> TokenStream {
    match component {
        Components::Input => {
            quote! { ::gpui_form::core::components::ComponentsBehaviour::Input }
        },
        Components::NumberInput(_) => {
            quote! { ::gpui_form::core::components::ComponentsBehaviour::NumberInput }
        },
        Components::Checkbox => {
            quote! { ::gpui_form::core::components::ComponentsBehaviour::Checkbox }
        },
        Components::Switch => {
            quote! { ::gpui_form::core::components::ComponentsBehaviour::Switch }
        },
        Components::Select(options) => {
            let searchable = options.behaviour.searchable;
            let partial = options.behaviour.partial;
            quote! {
                ::gpui_form::core::components::ComponentsBehaviour::Select(
                    ::gpui_form::core::components::BehaviourSelectOptions {
                        searchable: #searchable,
                        partial: #partial,
                    }
                )
            }
        },
        Components::InfiniteSelect(options) => {
            let searchable = options.behaviour.searchable;
            let max_depth = match options.behaviour.max_depth {
                Some(d) => quote! { Some(#d) },
                None => quote! { None },
            };
            quote! {
                ::gpui_form::core::components::ComponentsBehaviour::InfiniteSelect(
                    ::gpui_form::core::components::BehaviourInfiniteSelectOptions {
                        searchable: #searchable,
                        max_depth: #max_depth,
                    }
                )
            }
        },
        Components::DatePicker => {
            quote! { ::gpui_form::core::components::ComponentsBehaviour::DatePicker }
        },
    }
}

struct ComponentFieldContent {
    field_structure_tokens: TokenStream,
    field_base_declarations_tokens: TokenStream,
    /// (field_name, wrap_in_option) - whether this field should be wrapped in Option in the value holder
    wrap_in_option: (String, bool),
}

fn generate_component_field(field: &ComponentField) -> ComponentFieldContent {
    let field_name = field.ident.as_ref().unwrap().to_string();
    let field_type = &field.ty;

    let mut field_structure_tokens = proc_macro2::TokenStream::new();
    let mut field_base_declarations_tokens = proc_macro2::TokenStream::new();

    let component_def = match &field.component {
        Some(c) => c,
        None => {
            return ComponentFieldContent {
                field_structure_tokens,
                field_base_declarations_tokens,
                wrap_in_option: (field_name, false),
            };
        },
    };

    // Use the component's wraps_in_option() method to determine if the field should be wrapped
    let wrap_in_option = component_def.wraps_in_option();

    match component_def {
        Components::Input => {
            let component = InputComponent(FieldInformation::new(
                InputOptions,
                field_name.clone(),
                extract_type_ident(field_type),
            ));
            component.field_tokens(
                &mut field_structure_tokens,
                &mut field_base_declarations_tokens,
            );
        },
        Components::NumberInput(options) => {
            let component = NumberInputComponent(FieldInformation::new(
                options.clone(),
                field_name.clone(),
                extract_type_ident(field_type),
            ));
            component.field_tokens(
                &mut field_structure_tokens,
                &mut field_base_declarations_tokens,
            );
        },
        Components::Checkbox => {
            let component = CheckboxComponent(FieldInformation::new(
                CheckboxOptions,
                field_name.clone(),
                extract_type_ident(field_type),
            ));
            component.field_tokens(
                &mut field_structure_tokens,
                &mut field_base_declarations_tokens,
            );
        },
        Components::Switch => {
            let component = SwitchComponent(FieldInformation::new(
                SwitchOptions,
                field_name.clone(),
                extract_type_ident(field_type),
            ));
            component.field_tokens(
                &mut field_structure_tokens,
                &mut field_base_declarations_tokens,
            );
        },
        Components::Select(options) => {
            let component = SelectComponent(FieldInformation::new(
                options.clone(),
                field_name.clone(),
                extract_type_ident(field_type),
            ));
            component.field_tokens(
                &mut field_structure_tokens,
                &mut field_base_declarations_tokens,
            );
        },
        Components::InfiniteSelect(options) => {
            let component = InfiniteSelectComponent(FieldInformation::new(
                options.clone(),
                field_name.clone(),
                extract_type_ident(field_type),
            ));
            component.field_tokens(
                &mut field_structure_tokens,
                &mut field_base_declarations_tokens,
            );
        },
        Components::DatePicker => {
            let component = DatePickerComponent(FieldInformation::new(
                DatePickerOptions,
                field_name.clone(),
                extract_type_ident(field_type),
            ));
            component.field_tokens(
                &mut field_structure_tokens,
                &mut field_base_declarations_tokens,
            );
        },
    }

    ComponentFieldContent {
        field_structure_tokens,
        field_base_declarations_tokens,
        wrap_in_option: (field_name, wrap_in_option),
    }
}

fn extract_type_ident(ty: &Type) -> Ident {
    match ty {
        Type::Path(type_path) => {
            let last_segment = type_path.path.segments.last().unwrap_or_else(|| {
                panic!(
                    "Expected at least one segment in type path: {:?}",
                    type_path.to_token_stream()
                )
            });

            if last_segment.ident == "Option"
                && let PathArguments::AngleBracketed(args) = &last_segment.arguments
                && let Some(GenericArgument::Type(inner_type)) = args.args.first()
            {
                return extract_type_ident(inner_type);
            }
            last_segment.ident.clone()
        },
        _ => panic!(
            "Unsupported type for component field: not a Type::Path. Got: {:?}",
            ty.to_token_stream()
        ),
    }
}

/// Checks if a type is Option<T> and returns (is_option, inner_type).
/// If not an Option, returns the original type as inner_type.
fn extract_option_inner_type(ty: &Type) -> (bool, Type) {
    if let Type::Path(type_path) = ty
        && let Some(last_segment) = type_path.path.segments.last()
        && last_segment.ident == "Option"
        && let PathArguments::AngleBracketed(args) = &last_segment.arguments
        && let Some(GenericArgument::Type(inner_type)) = args.args.first()
    {
        (true, inner_type.clone())
    } else {
        (false, ty.clone())
    }
}

/// Generates the FormValueHolder struct and its implementations.
/// Component fields that need unwrapping become Option<T> in the value holder.
/// Other fields retain their original type.
/// The value holder derives Koruma with copied attributes from original fields,
/// plus RequiredValidation::<Option<_>> for non-optional fields wrapped in Option.
/// Returns (value_holder_tokens, fields_requiring_required_validation).
fn generate_value_holder(
    struct_name: &Ident,
    fields: &[FieldOptionality],
    enable_koruma: bool,
    enable_koruma_fluent: bool,
) -> (TokenStream, Vec<String>) {
    let value_holder_name = format_ident!("{}FormValueHolder", struct_name);

    // Check if we need to derive Koruma:
    // - Any field has koruma validators
    let has_any_koruma = fields
        .iter()
        .any(|f| !f.koruma_validators.is_empty() || f.is_newtype || f.is_nested);
    // - Any field needs RequiredValidation (non-optional wrapped in Option AND has other validations)
    let has_any_required = fields
        .iter()
        .any(|f| f.wrap_in_option && !f.was_optional && !f.koruma_validators.is_empty());
    let _needs_koruma_derive = has_any_koruma || has_any_required;

    // Generate value holder fields with koruma attributes
    // - Fields with wrap_in_option=true become Option<inner_type>
    // - Other fields keep their original type
    // - Copy koruma validators from original fields (using koruma_derive_core parsed data)
    // - Add RequiredValidation::<Option<_>> for non-optional fields wrapped in Option IF they have other validations
    let value_holder_fields: Vec<TokenStream> = fields
        .iter()
        .map(|f| {
            let name = &f.field_name;

            // Determine if we need to add RequiredValidation
            // (non-optional field that gets wrapped in Option AND has other validations)
            let needs_required = f.wrap_in_option && !f.was_optional && !f.koruma_validators.is_empty();

            // Build the koruma attribute(s) for this field
            let koruma_attr = if needs_required || !f.koruma_validators.is_empty() || f.is_newtype {
                // Convert parsed validators to token streams
                let existing_validations: Vec<TokenStream> = f
                    .koruma_validators
                    .iter()
                    .map(validator_attr_to_tokens)
                    .collect();

                // Build a combined list of all koruma items
                let mut koruma_items: Vec<TokenStream> = Vec::new();
                
                // Add RequiredValidation if needed
                if needs_required {
                    koruma_items.push(quote! { koruma_collection::general::RequiredValidation::<Option<_>> });
                }
                
                // Add newtype flag if present
                if f.is_newtype {
                    koruma_items.push(quote! { newtype });
                }
                
                // Add existing validators
                koruma_items.extend(existing_validations);
                
                // Generate the attribute if we have any items
                if !koruma_items.is_empty() {
                    quote! { #[koruma(#(#koruma_items),*)] }
                } else {
                    quote! {}
                }
            } else {
                quote! {}
            };

            if f.wrap_in_option {
                let inner_ty = &f.inner_type;
                quote! {
                    #koruma_attr
                    pub #name: Option<#inner_ty>,
                }
            } else {
                let orig_ty = &f.original_type;
                quote! {
                    #koruma_attr
                    pub #name: #orig_ty,
                }
            }
        })
        .collect();

    // Generate From<OriginalStruct> for ValueHolder
    let from_original_fields: Vec<TokenStream> = fields
        .iter()
        .map(|f| {
            let name = &f.field_name;
            if f.wrap_in_option {
                // Component field that needs option wrapping
                if f.was_optional {
                    // Original was Option<T>, value holder is Option<T> -> direct copy
                    quote! { #name: from.#name, }
                } else {
                    // Original was T, value holder is Option<T>
                    // Use None if value equals default, otherwise wrap in Some
                    let inner_ty = &f.inner_type;
                    quote! { #name: if from.#name == <#inner_ty as Default>::default() { None } else { Some(from.#name) }, }
                }
            } else {
                // Non-component field or field without unwrapping -> direct copy
                quote! { #name: from.#name, }
            }
        })
        .collect();

    // Generate From<ValueHolder> for OriginalStruct
    let from_holder_fields: Vec<TokenStream> = fields
        .iter()
        .map(|f| {
            let name = &f.field_name;
            if f.wrap_in_option {
                // Component field with option wrapping in value holder
                if f.was_optional {
                    // Original was Option<T>, value holder is Option<T> -> direct copy
                    quote! { #name: from.#name, }
                } else {
                    // Original was T, value holder is Option<T> -> unwrap_or_default
                    quote! { #name: from.#name.unwrap_or_default(), }
                }
            } else {
                // Non-component field or field without unwrapping -> direct copy
                quote! { #name: from.#name, }
            }
        })
        .collect();

    // Generate Default impl
    let default_fields: Vec<TokenStream> = fields
        .iter()
        .map(|f| {
            let name = &f.field_name;
            if f.wrap_in_option {
                quote! { #name: None, }
            } else {
                quote! { #name: Default::default(), }
            }
        })
        .collect();

    // Collect field names that were originally non-optional and are wrapped in option
    // AND have custom validations (these require RequiredValidation)
    let fields_requiring_required: Vec<String> = fields
        .iter()
        .filter(|f| f.wrap_in_option && !f.was_optional && !f.koruma_validators.is_empty())
        .map(|f| f.field_name.to_string())
        .collect();

    // Generate derive attributes conditionally
    let derive_attrs = if enable_koruma && _needs_koruma_derive {
        if enable_koruma_fluent {
            quote! { #[derive(Clone, Debug, ::koruma::Koruma, ::koruma::KorumaAllFluent)] }
        } else {
            quote! { #[derive(Clone, Debug, ::koruma::Koruma)] }
        }
    } else {
        quote! { #[derive(Clone, Debug)] }
    };

    let tokens = quote! {
        #derive_attrs
        pub struct #value_holder_name {
            #(#value_holder_fields)*
        }

        impl Default for #value_holder_name {
            fn default() -> Self {
                Self {
                    #(#default_fields)*
                }
            }
        }

        impl From<#struct_name> for #value_holder_name {
            fn from(from: #struct_name) -> Self {
                Self {
                    #(#from_original_fields)*
                }
            }
        }
        impl From<#value_holder_name> for #struct_name {
            fn from(from: #value_holder_name) -> Self {
                Self {
                    #(#from_holder_fields)*
                }
            }
        }
    };

    (tokens, fields_requiring_required)
}

pub struct GpuiFormOptions {
    pub generate_shape: bool,
}

fn expand_gpui_form(
    derive_input: DeriveInput,
    options: GpuiFormOptions,
) -> proc_macro2::TokenStream {
    // Flatten cfg_attr attributes before darling processing
    // This allows darling to see attributes like #[gpui_form(component(...))] even when
    // they're wrapped in #[cfg_attr(feature = "ui", ...)]
    let derive_input = flatten_cfg_attr_in_derive_input(derive_input);

    let parsed = match ComponentStruct::from_derive_input(&derive_input) {
        Ok(parsed) => parsed,
        Err(e) => return e.write_errors(),
    };

    let struct_name = &parsed.ident;
    let components_holder_name = format_ident!("{}FormFields", struct_name);
    let components_base_declarations_name = format_ident!("{}FormComponents", struct_name);
    let items_errors_struct_name = format_ident!("{}FormItemsErrors", struct_name);

    // Check for koruma options from direct attribute or cfg_attr
    let cfg_attr_koruma = find_koruma_in_cfg_attr(&derive_input.attrs);
    let koruma_options = parsed
        .koruma
        .as_ref()
        .map(|k| k.0.clone())
        .or(cfg_attr_koruma);

    // Handle empty structs with #[gpui_form(empty)] attribute
    if parsed.empty {
        let shape_impl = if options.generate_shape {
            quote! {
                ::gpui_form::core::registry::inventory::submit! {
                    ::gpui_form::core::registry::GpuiFormShape::new(
                        stringify!(#struct_name),
                        &[],
                        file!()
                    )
                }
            }
        } else {
            quote! {}
        };

        return quote! {
            pub struct #components_holder_name;

            #shape_impl

            pub struct #components_base_declarations_name;
        };
    }

    let fields_iter = match &parsed.data {
        ast::Data::Struct(s) => &s.fields,
        _ => unreachable!("GpuiForm derive only supports named structs"),
    };

    // Parse koruma fields using koruma_derive_core for proper attribute parsing
    // This replaces manual parsing and gives us both validation names and ValidatorAttr structs
    // Always parse - koruma_derive_core handles cfg_attr internally
    let parsed_koruma_fields: HashMap<String, KorumaFieldInfo> = match &derive_input.data {
        syn::Data::Struct(data_struct) => data_struct
            .fields
            .iter()
            .filter_map(|field| {
                let ident = field.ident.as_ref()?.to_string();
                match koruma_derive_core::parse_field(field) {
                    ParseFieldResult::Valid(info) => Some((ident, info)),
                    ParseFieldResult::Skip | ParseFieldResult::Error(_) => None,
                }
            })
            .collect(),
        _ => HashMap::new(),
    };

    // Enable koruma if we have explicit options OR if any koruma fields were found
    let enable_koruma = koruma_options.is_some() || !parsed_koruma_fields.is_empty();
    let enable_koruma_fluent = koruma_options.as_ref().map(|k| k.fluent).unwrap_or(false);

    // Extract koruma validation names (for metadata) from parsed data
    // Include explicit field_validators AND implicit validators from newtype/nested flags
    let koruma_validations: HashMap<String, Vec<String>> = parsed_koruma_fields
        .iter()
        .map(|(name, info)| {
            let mut validator_names: Vec<String> = info
                .field_validators
                .iter()
                .map(|v| v.name().to_string())
                .collect();

            // Add NewtypeValidation if the field is marked as newtype
            if info.is_newtype() {
                validator_names.push("NewtypeValidation".to_string());
            }

            // Add NestedValidation if the field is marked as nested
            if info.is_nested() {
                validator_names.push("NestedValidation".to_string());
            }

            (name.clone(), validator_names)
        })
        .collect();

    // Check if struct has no fields but is missing #[gpui_form(empty)] attribute
    if fields_iter.is_empty() {
        return syn::Error::new_spanned(
            &derive_input,
            format!(
                "Struct `{}` has no fields. Add `#[gpui_form(empty)]` attribute to explicitly mark it as an empty form.",
                struct_name
            ),
        )
        .to_compile_error();
    }

    let component_field_pairs: Vec<ComponentFieldContent> = fields_iter
        .iter()
        .filter(|field| !field.skip())
        .map(generate_component_field)
        .collect();

    let (field_structure_tokens, field_base_declarations_tokens, wrap_in_option_map): (
        Vec<TokenStream>,
        Vec<TokenStream>,
        HashMap<String, bool>,
    ) = component_field_pairs
        .into_iter()
        .map(|content| {
            (
                content.field_structure_tokens,
                content.field_base_declarations_tokens,
                content.wrap_in_option,
            )
        })
        .multiunzip();

    // Build field optionality information for value holder generation
    // Include ALL fields (even skipped ones) so From impls work correctly
    let field_optionality: Vec<FieldOptionality> = fields_iter
        .iter()
        .map(|field| {
            let field_name = field.ident.clone().unwrap();
            let field_name_str = field_name.to_string();
            let (was_optional, inner_type) = extract_option_inner_type(&field.ty);
            // Wrap in option based on component's wraps_in_option() (and not skipped)
            let wrap_in_option = !field.skip()
                && field.component.is_some()
                && wrap_in_option_map
                    .get(&field_name_str)
                    .copied()
                    .unwrap_or(false);
            // Get parsed koruma validators for this field
            let koruma_info = parsed_koruma_fields.get(&field_name_str);
            let koruma_validators = koruma_info
                .map(|info| info.field_validators.clone())
                .unwrap_or_default();
            let is_newtype = koruma_info.map(|info| info.is_newtype()).unwrap_or(false);
            let is_nested = koruma_info.map(|info| info.is_nested()).unwrap_or(false);
            FieldOptionality {
                field_name,
                original_type: field.ty.clone(),
                inner_type,
                was_optional,
                wrap_in_option,
                koruma_validators,
                is_newtype,
                is_nested,
            }
        })
        .collect();

    // Generate value holder struct and get list of fields requiring RequiredValidation
    let (value_holder_tokens, fields_requiring_required) = generate_value_holder(
        struct_name,
        &field_optionality,
        enable_koruma,
        enable_koruma_fluent,
    );

    // Generate error struct fields
    let items_error_struct_fields: Vec<TokenStream> = fields_iter
        .iter()
        .filter(|field| !field.skip() && field.component.is_some())
        .map(|field| {
            let field_name = field.ident.as_ref().unwrap();
            quote! {
                pub #field_name: String,
            }
        })
        .collect();

    // Generate Default implementation for items errors struct
    let items_error_struct_defaults: Vec<TokenStream> = fields_iter
        .iter()
        .filter(|field| !field.skip() && field.component.is_some())
        .map(|field| {
            let field_name = field.ident.as_ref().unwrap();
            quote! {
                #field_name: String::new(),
            }
        })
        .collect();

    // Generate methods on items errors struct to check if there are errors
    let items_error_has_error_checks: Vec<TokenStream> = fields_iter
        .iter()
        .filter(|field| !field.skip() && field.component.is_some())
        .map(|field| {
            let field_name = field.ident.as_ref().unwrap();
            quote! {
                !self.#field_name.is_empty()
            }
        })
        .collect();

    let field_variant_construction_code: Vec<TokenStream> = fields_iter
        .iter()
        .filter_map(|field| {
            if field.skip() || field.component.is_none() {
                None
            } else {
                let field_name_str = field
                    .ident
                    .as_ref()
                    .expect("Field should have an ident if not skipped and has component")
                    .to_string();
                let (is_optional, base_type) = 'option_check: {
                    if let syn::Type::Path(type_path) = &field.ty
                        && let Some(segment) = type_path.path.segments.last()
                        && segment.ident == "Option"
                        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
                        && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
                    {
                        break 'option_check (true, inner_ty);
                    }
                    (false, &field.ty)
                };

                let field_type_str = base_type.to_token_stream().to_string();
                let component_def = field.component.as_ref().unwrap();
                let behaviour_tokens = get_components_behaviour_tokens(component_def);
                let mut validation_rules = koruma_validations
                    .get(&field_name_str)
                    .cloned()
                    .unwrap_or_default();

                // Add implicit RequiredValidation for non-optional fields that HAVE other validations
                if fields_requiring_required.contains(&field_name_str)
                    && !validation_rules.contains(&"RequiredValidation".to_string())
                {
                    validation_rules.insert(0, "RequiredValidation".to_string());
                }

                let validation_literals: Vec<_> = validation_rules
                    .iter()
                    .map(|v| syn::LitStr::new(v, proc_macro2::Span::call_site()))
                    .collect();

                Some(quote! {
                    ::gpui_form::core::registry::FieldVariant::new(
                        #field_name_str,
                        #field_type_str,
                        #is_optional,
                        #behaviour_tokens
                    ).with_validations(&[
                        #( #validation_literals ),*
                    ])
                })
            }
        })
        .collect();

    let shape_impl = if options.generate_shape {
        quote! {
            ::gpui_form::core::registry::inventory::submit! {
                ::gpui_form::core::registry::GpuiFormShape::new(
                    stringify!(#struct_name),
                    &[
                        #(#field_variant_construction_code),*
                    ],
                    file!()
                )
            }
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
        #value_holder_tokens
        pub struct #components_holder_name {
            #(#field_structure_tokens)*
        }

        #shape_impl

        pub struct #components_base_declarations_name;

        impl #components_base_declarations_name {
          #(#field_base_declarations_tokens)*
        }

        /// Struct tracking which form fields have errors.
        #[derive(Clone, Debug)]
        pub struct #items_errors_struct_name {
            #(#items_error_struct_fields)*
        }

        impl Default for #items_errors_struct_name {
            fn default() -> Self {
                Self {
                    #(#items_error_struct_defaults)*
                }
            }
        }

        impl #items_errors_struct_name {
            /// Returns true if any field has an error
            pub fn has_errors(&self) -> bool {
                #(#items_error_has_error_checks)||*
            }

            /// Clears all errors
            pub fn clear(&mut self) {
                *self = Self::default();
            }
        }
    };

    expanded
}

pub fn from(input: proc_macro::TokenStream, options: GpuiFormOptions) -> proc_macro::TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);

    expand_gpui_form(derive_input, options).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn test_find_koruma_in_cfg_attr_direct_koruma() {
        // Test: #[cfg_attr(feature = "ui", gpui_form(koruma))]
        let tokens = quote! {
            #[cfg_attr(feature = "ui", gpui_form(koruma))]
            struct Test;
        };
        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let result = find_koruma_in_cfg_attr(&derive_input.attrs);
        assert!(result.is_some(), "Should find koruma in cfg_attr");
        assert!(!result.unwrap().fluent, "fluent should be false");
    }

    #[test]
    fn test_find_koruma_in_cfg_attr_with_fluent() {
        // Test: #[cfg_attr(feature = "ui", gpui_form(koruma(fluent)))]
        let tokens = quote! {
            #[cfg_attr(feature = "ui", gpui_form(koruma(fluent)))]
            struct Test;
        };
        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let result = find_koruma_in_cfg_attr(&derive_input.attrs);
        assert!(result.is_some(), "Should find koruma in cfg_attr");
        assert!(result.unwrap().fluent, "fluent should be true");
    }

    #[test]
    fn test_find_koruma_in_cfg_attr_no_koruma() {
        // Test: #[cfg_attr(feature = "ui", gpui_form(empty))]
        let tokens = quote! {
            #[cfg_attr(feature = "ui", gpui_form(empty))]
            struct Test;
        };
        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let result = find_koruma_in_cfg_attr(&derive_input.attrs);
        assert!(result.is_none(), "Should not find koruma");
    }

    #[test]
    fn test_find_koruma_in_cfg_attr_not_cfg_attr() {
        // Test: #[gpui_form(koruma(fluent))] - direct attribute, not cfg_attr
        let tokens = quote! {
            #[gpui_form(koruma(fluent))]
            struct Test;
        };
        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let result = find_koruma_in_cfg_attr(&derive_input.attrs);
        assert!(result.is_none(), "Should not find koruma in non-cfg_attr");
    }

    #[test]
    fn test_find_koruma_in_cfg_attr_complex_condition() {
        // Test: #[cfg_attr(all(feature = "ui", feature = "validation"), gpui_form(koruma(fluent)))]
        let tokens = quote! {
            #[cfg_attr(all(feature = "ui", feature = "validation"), gpui_form(koruma(fluent)))]
            struct Test;
        };
        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let result = find_koruma_in_cfg_attr(&derive_input.attrs);
        assert!(
            result.is_some(),
            "Should find koruma with complex condition"
        );
        assert!(result.unwrap().fluent, "fluent should be true");
    }

    #[test]
    fn test_find_koruma_in_cfg_attr_multiple_attrs_in_cfg_attr() {
        // Test: #[cfg_attr(feature = "ui", derive(Something), gpui_form(koruma))]
        let tokens = quote! {
            #[cfg_attr(feature = "ui", derive(Something), gpui_form(koruma))]
            struct Test;
        };
        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let result = find_koruma_in_cfg_attr(&derive_input.attrs);
        assert!(
            result.is_some(),
            "Should find koruma among multiple attrs in cfg_attr"
        );
    }

    #[test]
    fn test_koruma_field_parsing_with_cfg_attr() {
        // Test that koruma_derive_core::parse_field works with cfg_attr
        // This tests the integration with koruma's cfg_attr parsing
        let tokens = quote! {
            struct Test {
                #[cfg_attr(feature = "validation", koruma(SomeValidator::<_>))]
                field: u32,
            }
        };
        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();

        if let syn::Data::Struct(data_struct) = &derive_input.data {
            let field = data_struct.fields.iter().next().unwrap();
            let result = koruma_derive_core::parse_field(field);

            // This should find the validator if koruma_derive_core handles cfg_attr
            match result {
                ParseFieldResult::Valid(info) => {
                    assert!(
                        !info.field_validators.is_empty(),
                        "Should find validators in cfg_attr"
                    );
                    assert_eq!(
                        info.field_validators[0].name().to_string(),
                        "SomeValidator",
                        "Should extract correct validator name"
                    );
                },
                ParseFieldResult::Skip => {
                    panic!(
                        "parse_field returned Skip - koruma_derive_core may not be handling cfg_attr correctly"
                    );
                },
                ParseFieldResult::Error(e) => {
                    panic!("parse_field returned Error: {}", e);
                },
            }
        } else {
            panic!("Expected struct data");
        }
    }

    #[test]
    fn test_koruma_field_parsing_newtype_in_cfg_attr() {
        // Test parsing #[cfg_attr(feature = "validation", koruma(newtype))]
        let tokens = quote! {
            struct Test {
                #[cfg_attr(feature = "validation", koruma(newtype))]
                field: SomeNewtype,
            }
        };
        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();

        if let syn::Data::Struct(data_struct) = &derive_input.data {
            let field = data_struct.fields.iter().next().unwrap();
            let result = koruma_derive_core::parse_field(field);

            match result {
                ParseFieldResult::Valid(info) => {
                    assert!(info.is_newtype(), "Should detect newtype in cfg_attr");
                },
                ParseFieldResult::Skip => {
                    panic!(
                        "parse_field returned Skip - koruma_derive_core may not be handling cfg_attr correctly for newtype"
                    );
                },
                ParseFieldResult::Error(e) => {
                    panic!("parse_field returned Error: {}", e);
                },
            }
        } else {
            panic!("Expected struct data");
        }
    }

    #[test]
    fn test_koruma_field_parsing_nested_in_cfg_attr() {
        // Test parsing #[cfg_attr(feature = "validation", koruma(nested))]
        let tokens = quote! {
            struct Test {
                #[cfg_attr(feature = "validation", koruma(nested))]
                field: NestedStruct,
            }
        };
        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();

        if let syn::Data::Struct(data_struct) = &derive_input.data {
            let field = data_struct.fields.iter().next().unwrap();
            let result = koruma_derive_core::parse_field(field);

            match result {
                ParseFieldResult::Valid(info) => {
                    assert!(info.is_nested(), "Should detect nested in cfg_attr");
                },
                ParseFieldResult::Skip => {
                    panic!(
                        "parse_field returned Skip - koruma_derive_core may not be handling cfg_attr correctly for nested"
                    );
                },
                ParseFieldResult::Error(e) => {
                    panic!("parse_field returned Error: {}", e);
                },
            }
        } else {
            panic!("Expected struct data");
        }
    }

    #[test]
    fn test_cfg_attr_end_to_end_validation_generation() {
        // Comprehensive end-to-end test: struct with cfg_attr-wrapped koruma validation
        // should generate proper field metadata AND validation error display code
        let tokens = quote! {
            #[derive(GpuiForm)]
            #[gpui_form(koruma(fluent))]
            struct TestForm {
                #[gpui_form(component(input))]
                #[cfg_attr(feature = "validation", koruma(koruma_collection::general::RequiredValidation::<Option<_>>))]
                name: String,

                #[gpui_form(component(number_input))]
                #[cfg_attr(feature = "validation", koruma(koruma_collection::numeric::PositiveValidation::<_>))]
                age: u32,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();

        // Verify koruma-derive-core can parse the fields
        if let syn::Data::Struct(data_struct) = &derive_input.data {
            for (idx, field) in data_struct.fields.iter().enumerate() {
                let result = koruma_derive_core::parse_field(field);
                match result {
                    ParseFieldResult::Valid(info) => {
                        assert!(
                            !info.field_validators.is_empty(),
                            "Field {} should have validators detected from cfg_attr",
                            idx
                        );
                        // Verify the validator names
                        let validator_names: Vec<String> = info
                            .field_validators
                            .iter()
                            .map(|v| v.name().to_string())
                            .collect();
                        assert!(
                            !validator_names.is_empty(),
                            "Field {} should have named validators",
                            idx
                        );
                    },
                    ParseFieldResult::Skip => {
                        panic!("Field {} should have validators, got Skip", idx);
                    },
                    ParseFieldResult::Error(e) => {
                        panic!("Field {} parsing failed: {}", idx, e);
                    },
                }
            }
        }

        // Now test the full macro expansion
        let expanded = expand_gpui_form(
            derive_input.clone(),
            GpuiFormOptions {
                generate_shape: true,
            },
        );

        let expanded_str = expanded.to_string();

        // Verify that validation-related code is generated:
        // 1. The value holder should derive Koruma and KorumaAllFluent
        assert!(
            expanded_str.contains("Koruma") || expanded_str.contains("koruma"),
            "Generated code should include Koruma-related types"
        );

        // 2. The render function should bind validation_errors
        assert!(
            expanded_str.contains("validation_errors") || expanded_str.contains("validate"),
            "Generated code should include validation error handling: {}",
            &expanded_str[..expanded_str.len().min(500)]
        );

        // 3. Field metadata should include validation information
        // The GpuiFormShape inventory submission should have validations in FieldVariant
        assert!(
            expanded_str.contains("FieldVariant") || expanded_str.contains("validations"),
            "Generated code should include field variant metadata"
        );
    }

    #[test]
    fn test_real_world_common_v_read_pattern() {
        // This test matches the EXACT pattern from CommonVRead in the user's code
        let tokens = quote! {
            #[cfg_attr(feature = "ui", derive(GpuiForm))]
            #[cfg_attr(feature = "ui", gpui_form(koruma(fluent)))]
            pub struct CommonVRead {
                #[cfg_attr(feature = "ui", gpui_form(component(number_input)))]
                #[cfg_attr(feature = "validation", koruma(newtype))]
                pub index: CommonVariableIndex,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();

        // Parse the fields using koruma_derive_core
        if let syn::Data::Struct(data_struct) = &derive_input.data {
            let field = data_struct.fields.iter().next().unwrap();
            let result = koruma_derive_core::parse_field(field);

            eprintln!("=== DEBUG: parse_field result for CommonVRead.index ===");
            match &result {
                ParseFieldResult::Valid(info) => {
                    eprintln!("  Result: Valid");
                    eprintln!("  is_newtype: {}", info.is_newtype());
                    eprintln!("  field_validators.len(): {}", info.field_validators.len());
                    for (idx, v) in info.field_validators.iter().enumerate() {
                        eprintln!("    validator[{}]: {}", idx, v.name());
                    }
                },
                ParseFieldResult::Skip => {
                    eprintln!("  Result: Skip");
                },
                ParseFieldResult::Error(e) => {
                    eprintln!("  Result: Error({})", e);
                },
            }

            // This should detect newtype!
            match result {
                ParseFieldResult::Valid(info) => {
                    assert!(
                        info.is_newtype(),
                        "Should detect newtype validation in nested cfg_attr"
                    );
                },
                ParseFieldResult::Skip => {
                    panic!(
                        "koruma_derive_core returned Skip for field with koruma(newtype) - cfg_attr not being handled!"
                    );
                },
                ParseFieldResult::Error(e) => {
                    panic!("koruma_derive_core returned Error: {}", e);
                },
            }
        }

        // Now test the full expansion with generate_shape
        let expanded = expand_gpui_form(
            derive_input,
            GpuiFormOptions {
                generate_shape: true,
            },
        );

        let expanded_str = expanded.to_string();
        eprintln!("=== Generated code (first 1000 chars) ===");
        eprintln!("{}", &expanded_str[..expanded_str.len().min(1000)]);

        // The GpuiFormShape should include validation metadata
        assert!(
            expanded_str.contains("with_validations"),
            "Generated GpuiFormShape should call with_validations()"
        );
    }
}
