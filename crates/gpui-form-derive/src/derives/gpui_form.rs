use std::collections::HashMap;

use darling::{FromDeriveInput, FromField, FromMeta, ast};
use gpui_form_core::components::*;
use gpui_form_core::implementations::ComponentLayout as _;
use itertools::Itertools as _;
use koruma_derive_core::{
    FieldInfo as KorumaFieldInfo, ParseFieldResult, ValidationInfo, ValidatorAttr,
};
use proc_macro2::TokenStream;
use quote::{ToTokens as _, format_ident, quote};
use syn::{DeriveInput, Expr, GenericArgument, Ident, PathArguments, Type, parse_macro_input};
use syn_cfg_attr::{AttributeHelpers as _, ExpandedAttr};

/// Flattens `cfg_attr` attributes in a DeriveInput.
fn flatten_cfg_attr_in_derive_input(mut input: DeriveInput) -> DeriveInput {
    // Flatten struct-level attributes
    input.attrs = flatten_attrs(input.attrs);

    // Flatten field-level attributes
    if let syn::Data::Struct(ref mut data_struct) = input.data {
        for field in data_struct.fields.iter_mut() {
            field.attrs = flatten_attrs(field.attrs.clone());
        }
    }

    input
}

fn flatten_attrs(attrs: Vec<syn::Attribute>) -> Vec<syn::Attribute> {
    attrs
        .flattened_attributes()
        .into_iter()
        .map(|expanded| match expanded {
            ExpandedAttr::Direct(attr) => attr,
            ExpandedAttr::Nested { attr, .. } => syn::Attribute {
                pound_token: Default::default(),
                style: syn::AttrStyle::Outer,
                bracket_token: Default::default(),
                meta: attr,
            },
        })
        .collect()
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
    /// Validation info for this field (validators, modifiers, etc.)
    validation: ValidationInfo,
    /// Custom default expression for this field (original type), if provided.
    default_expr: Option<TokenStream>,
}

#[derive(Debug, FromField)]
#[darling(attributes(gpui_form))]
struct ComponentField {
    pub ident: Option<Ident>,
    pub ty: Type,
    #[darling(default)]
    pub component: Option<Components>,
    #[darling(default)]
    pub default: Option<Expr>,
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

fn parse_field_default(field: &ComponentField) -> Option<TokenStream> {
    field.default.as_ref().map(|expr| quote! { #expr })
}

/// Generates the FormValueHolder struct and its implementations.
/// Component fields that need unwrapping become Option<T> in the value holder.
/// Other fields retain their original type.
/// The value holder derives Koruma with copied attributes from original fields.
/// When koruma support is enabled, it also adds RequiredValidation::<Option<_>>
/// for non-optional fields wrapped in Option.
/// Returns (value_holder_tokens, fields_requiring_required_validation).
fn generate_value_holder(
    struct_name: &Ident,
    struct_is_unit: bool,
    fields: &[FieldOptionality],
    enable_koruma: bool,
    enable_koruma_fluent: bool,
) -> (TokenStream, Vec<String>) {
    let value_holder_name = format_ident!("{}FormValueHolder", struct_name);

    // Check if we need to derive Koruma:
    // - Any field has koruma validators (including newtype/nested flags)
    let has_any_koruma = fields.iter().any(|f| {
        !f.validation.field_validators.is_empty()
            || !f.validation.element_validators.is_empty()
            || f.validation.is_nested
            || f.validation.is_newtype
    });
    // - Any field needs RequiredValidation (non-optional wrapped in Option, excluding newtype/nested)
    let has_any_required = fields.iter().any(|f| {
        f.wrap_in_option && !f.was_optional && !f.validation.is_newtype && !f.validation.is_nested
    });
    let _needs_koruma_derive = has_any_koruma || has_any_required;

    // Generate value holder fields with koruma attributes
    // - Fields with wrap_in_option=true become Option<inner_type>
    // - Other fields keep their original type
    // - Copy koruma validators from original fields (using koruma_derive_core parsed data)
    // - Add RequiredValidation::<Option<_>> for non-optional fields wrapped in Option
    let value_holder_fields: Vec<TokenStream> = fields
        .iter()
        .map(|f| {
            let name = &f.field_name;

            // Determine if we need to add RequiredValidation
            // (non-optional field that gets wrapped in Option)
            // This ensures that non-optional fields cannot be None in the value holder
            // Only add RequiredValidation when koruma support is enabled for this value holder.
            // Koruma treats `newtype`/`nested` as standalone modifiers, so we avoid
            // mixing them with additional validators.
            let needs_required = enable_koruma
                && f.wrap_in_option
                && !f.was_optional
                && !f.validation.is_newtype
                && !f.validation.is_nested;

            // Build the koruma attribute(s) for this field
            // Only generate koruma attribute if:
            // - We need RequiredValidation for non-optional wrapped fields, OR
            // - The field has existing koruma validators (and koruma is enabled), OR
            // - The field has newtype flag (and koruma is enabled)
            let has_existing_validations = !f.validation.field_validators.is_empty()
                || !f.validation.element_validators.is_empty();
            // Only include newtype in the condition if koruma is enabled
            let has_newtype = enable_koruma && f.validation.is_newtype;
            let koruma_attr =
                if needs_required || (enable_koruma && has_existing_validations) || has_newtype {
                    let mut koruma_attrs: Vec<TokenStream> = Vec::new();

                    // Emit modifiers as separate attributes (koruma parses these only when standalone).
                    if enable_koruma && f.validation.is_newtype {
                        koruma_attrs.push(quote! { #[koruma(newtype)] });
                    }
                    if enable_koruma && f.validation.is_nested {
                        koruma_attrs.push(quote! { #[koruma(nested)] });
                    }

                    // Convert parsed validators to token streams
                    let existing_validations: Vec<TokenStream> = f
                        .validation
                        .field_validators
                        .iter()
                        .chain(f.validation.element_validators.iter())
                        .map(validator_attr_to_tokens)
                        .collect();

                    // Build a combined list of validator items
                    let mut koruma_items: Vec<TokenStream> = Vec::new();

                    // Add RequiredValidation if needed
                    if needs_required {
                        koruma_items.push(
                            quote! { koruma_collection::general::RequiredValidation::<Option<_>> },
                        );
                    }

                    // Add existing validators (only if koruma is enabled)
                    if enable_koruma {
                        koruma_items.extend(existing_validations);
                    }

                    if !koruma_items.is_empty() {
                        koruma_attrs.push(quote! { #[koruma(#(#koruma_items),*)] });
                    }

                    if !koruma_attrs.is_empty() {
                        quote! { #(#koruma_attrs)* }
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
                    let default_expr = f
                        .default_expr
                        .as_ref()
                        .map(|expr| quote! { #expr })
                        .unwrap_or_else(|| quote! { <#inner_ty as Default>::default() });
                    quote! { #name: if from.#name == #default_expr { None } else { Some(from.#name) }, }
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
                    if let Some(default_expr) = &f.default_expr {
                        quote! { #name: from.#name.unwrap_or_else(|| #default_expr), }
                    } else {
                        quote! { #name: from.#name.unwrap_or_default(), }
                    }
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
                if f.was_optional {
                    if let Some(default_expr) = &f.default_expr {
                        quote! { #name: #default_expr, }
                    } else {
                        quote! { #name: None, }
                    }
                } else {
                    quote! { #name: None, }
                }
            } else if let Some(default_expr) = &f.default_expr {
                quote! { #name: #default_expr, }
            } else {
                quote! { #name: Default::default(), }
            }
        })
        .collect();

    // Collect field names that were originally non-optional and are wrapped in option
    // (these require RequiredValidation)
    let fields_requiring_required: Vec<String> = fields
        .iter()
        .filter(|f| {
            f.wrap_in_option
                && !f.was_optional
                && !f.validation.is_newtype
                && !f.validation.is_nested
        })
        .map(|f| f.field_name.to_string())
        .collect();

    // Generate derive attributes conditionally
    // Derive Koruma if:
    // - enable_koruma is true AND there are actual koruma validators (existing behavior), OR
    // - There are fields that need RequiredValidation (even without other koruma validators)
    //   to ensure RequiredValidation is applied to non-optional fields wrapped in Option
    let needs_koruma_for_required = has_any_required && enable_koruma;
    let derive_attrs = if (enable_koruma && has_any_koruma) || needs_koruma_for_required {
        if enable_koruma_fluent {
            quote! { #[derive(Clone, Debug, ::koruma::Koruma, ::koruma::KorumaAllFluent)] }
        } else {
            quote! { #[derive(Clone, Debug, ::koruma::Koruma)] }
        }
    } else {
        quote! { #[derive(Clone, Debug)] }
    };

    let from_holder_constructor = if struct_is_unit {
        quote! { Self }
    } else {
        quote! {
            Self {
                #(#from_holder_fields)*
            }
        }
    };

    let from_unused_suppress = if fields.is_empty() {
        quote! { let _ = from; }
    } else {
        quote! {}
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
                #from_unused_suppress
                Self {
                    #(#from_original_fields)*
                }
            }
        }
        impl From<#value_holder_name> for #struct_name {
            fn from(from: #value_holder_name) -> Self {
                #from_unused_suppress
                #from_holder_constructor
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
    let struct_is_unit = matches!(
        derive_input.data,
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Unit,
            ..
        })
    );
    let components_holder_name = format_ident!("{}FormFields", struct_name);
    let components_base_declarations_name = format_ident!("{}FormComponents", struct_name);
    let items_errors_struct_name = format_ident!("{}FormItemsErrors", struct_name);

    // Check for koruma options from direct attribute
    let koruma_options = parsed.koruma.as_ref().map(|k| k.0.clone());

    // Handle empty structs with #[gpui_form(empty)] attribute
    if parsed.empty {
        let enable_koruma = koruma_options.is_some();
        let enable_koruma_fluent = koruma_options.as_ref().map(|k| k.fluent).unwrap_or(false);
        let empty_fields: Vec<FieldOptionality> = Vec::new();
        let (value_holder_tokens, _) = generate_value_holder(
            struct_name,
            struct_is_unit,
            &empty_fields,
            enable_koruma,
            enable_koruma_fluent,
        );
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
            #value_holder_tokens
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
                match koruma_derive_core::parse_field(field, 0) {
                    ParseFieldResult::Valid(info) => Some((ident, *info)),
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
                .validation
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
    let mut field_optionality = Vec::new();
    for field in fields_iter {
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
        let validation = koruma_info
            .map(|info| info.validation.clone())
            .unwrap_or_default();
        let default_expr = parse_field_default(field);
        field_optionality.push(FieldOptionality {
            field_name,
            original_type: field.ty.clone(),
            inner_type,
            was_optional,
            wrap_in_option,
            validation,
            default_expr,
        });
    }

    // Generate value holder struct and get list of fields requiring RequiredValidation
    // Pass true for enable_koruma if there are any fields that need RequiredValidation,
    // even if the original struct doesn't have koruma attributes
    let has_fields_needing_required = field_optionality.iter().any(|f| {
        f.wrap_in_option && !f.was_optional && !f.validation.is_newtype && !f.validation.is_nested
    });
    // Only enable koruma for the value holder if:
    // - The original struct has koruma attributes (enable_koruma), OR
    // - There are fields needing RequiredValidation AND there are existing koruma validations
    //   (to avoid generating koruma derive for structs without any koruma setup)
    // Note: is_newtype alone doesn't count as a koruma validation for this purpose
    let has_any_koruma_validations = field_optionality.iter().any(|f| {
        !f.validation.field_validators.is_empty()
            || !f.validation.element_validators.is_empty()
            || f.validation.is_nested
    });
    let effective_enable_koruma =
        enable_koruma || (has_fields_needing_required && has_any_koruma_validations);
    let (value_holder_tokens, fields_requiring_required) = generate_value_holder(
        struct_name,
        struct_is_unit,
        &field_optionality,
        effective_enable_koruma,
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
            let result = koruma_derive_core::parse_field(field, 0);

            // This should find the validator if koruma_derive_core handles cfg_attr
            match result {
                ParseFieldResult::Valid(info) => {
                    assert!(
                        !info.validation.field_validators.is_empty(),
                        "Should find validators in cfg_attr"
                    );
                    assert_eq!(
                        info.validation.field_validators[0].name().to_string(),
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
            let result = koruma_derive_core::parse_field(field, 0);

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
            let result = koruma_derive_core::parse_field(field, 0);

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
                let result = koruma_derive_core::parse_field(field, idx);
                match result {
                    ParseFieldResult::Valid(info) => {
                        assert!(
                            !info.validation.field_validators.is_empty(),
                            "Field {} should have validators detected from cfg_attr",
                            idx
                        );
                        // Verify the validator names
                        let validator_names: Vec<String> = info
                            .validation
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
            let result = koruma_derive_core::parse_field(field, 0);

            eprintln!("=== DEBUG: parse_field result for CommonVRead.index ===");
            match &result {
                ParseFieldResult::Valid(info) => {
                    eprintln!("  Result: Valid");
                    eprintln!("  is_newtype: {}", info.is_newtype());
                    eprintln!(
                        "  field_validators.len(): {}",
                        info.validation.field_validators.len()
                    );
                    for (idx, v) in info.validation.field_validators.iter().enumerate() {
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
