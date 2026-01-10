use std::collections::HashMap;

use darling::{FromDeriveInput, FromField, ast};
use gpui_form_core::components::*;
use gpui_form_core::implementations::ComponentLayout as _;
use itertools::Itertools as _;
use proc_macro2::TokenStream;
use quote::{ToTokens as _, format_ident, quote};
use syn::{DeriveInput, GenericArgument, Ident, PathArguments, Type, parse_macro_input};

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
}

fn get_components_behaviour_tokens(component: &Components) -> TokenStream {
    match component {
        Components::Input => {
            quote! { ::gpui_form::core::components::ComponentsBehaviour::Input }
        },
        Components::NumberInput => {
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
        Components::TupleSelect(options) => {
            let searchable = options.behaviour.searchable;
            let max_depth = match options.behaviour.max_depth {
                Some(d) => quote! { Some(#d) },
                None => quote! { None },
            };
            quote! {
                ::gpui_form::core::components::ComponentsBehaviour::TupleSelect(
                    ::gpui_form::core::components::BehaviourTupleSelectOptions {
                        searchable: #searchable,
                        max_depth: #max_depth,
                    }
                )
            }
        },
        Components::DatePicker => {
            quote! { ::gpui_form::core::components::ComponentsBehaviour::DatePicker }
        },
        Components::Custom(custom_options) => {
            let component_ident = &custom_options.behaviour.name;
            quote! { #component_ident }
        },
    }
}

struct ComponentFieldContent {
    field_structure_tokens: TokenStream,
    field_base_declarations_tokens: TokenStream,
    should_be_unwrapped: (String, bool),
}

fn generate_component_field(field: &ComponentField) -> ComponentFieldContent {
    let field_name = field.ident.as_ref().unwrap().to_string();
    let field_type = &field.ty;

    let mut field_structure_tokens = proc_macro2::TokenStream::new();
    let mut field_base_declarations_tokens = proc_macro2::TokenStream::new();
    let mut should_be_unwrapped = (field_name.clone(), false);

    let component_def = if field.component.is_some() {
        field.component.as_ref().unwrap()
    } else {
        return ComponentFieldContent {
            field_structure_tokens,
            field_base_declarations_tokens,
            should_be_unwrapped,
        };
    };

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
            should_be_unwrapped.1 = true;
        },
        Components::NumberInput => {
            let component = NumberInputComponent(FieldInformation::new(
                NumberInputOptions,
                field_name.clone(),
                extract_type_ident(field_type),
            ));
            component.field_tokens(
                &mut field_structure_tokens,
                &mut field_base_declarations_tokens,
            );
            should_be_unwrapped.1 = true;
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
            should_be_unwrapped.1 = true;
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
            should_be_unwrapped.1 = true;
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
            should_be_unwrapped.1 = true;
        },
        Components::TupleSelect(options) => {
            let component = TupleSelectComponent(FieldInformation::new(
                options.clone(),
                field_name.clone(),
                extract_type_ident(field_type),
            ));
            component.field_tokens(
                &mut field_structure_tokens,
                &mut field_base_declarations_tokens,
            );
            should_be_unwrapped.1 = true;
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
            should_be_unwrapped.1 = false;
        },
        Components::Custom(options) => {
            let component = CustomComponent(FieldInformation::new(
                options.clone(),
                field_name.clone(),
                extract_type_ident(field_type),
            ));
            component.field_tokens(
                &mut field_structure_tokens,
                &mut field_base_declarations_tokens,
            );
            should_be_unwrapped.1 = options.behaviour.should_be_unwrapped;
        },
    }

    ComponentFieldContent {
        field_structure_tokens,
        field_base_declarations_tokens,
        should_be_unwrapped,
    }
}

/// Extracts the last path segment identifier from an expression.
/// Handles both `Expr::Path` and nested expressions for call-like validators.
fn extract_path_last_ident(expr: &syn::Expr) -> Option<String> {
    match expr {
        syn::Expr::Path(path_expr) => path_expr.path.segments.last().map(|s| s.ident.to_string()),
        _ => None,
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

pub struct GpuiFormOptions {
    pub generate_shape: bool,
}

fn expand_gpui_form(
    derive_input: DeriveInput,
    options: GpuiFormOptions,
) -> proc_macro2::TokenStream {
    let parsed = match ComponentStruct::from_derive_input(&derive_input) {
        Ok(parsed) => parsed,
        Err(e) => return e.write_errors(),
    };

    let struct_name = &parsed.ident;
    let components_holder_name = format_ident!("{}FormFields", struct_name);
    let components_base_declarations_name = format_ident!("{}FormComponents", struct_name);
    let items_errors_struct_name = format_ident!("{}FormItemsErrors", struct_name);

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

    let koruma_validations: HashMap<String, Vec<String>> = match &derive_input.data {
        syn::Data::Struct(data_struct) => data_struct
            .fields
            .iter()
            .filter_map(|field| {
                let ident = field.ident.as_ref()?.to_string();
                let mut validations = Vec::new();
                for attr in &field.attrs {
                    if attr.path().is_ident("koruma") {
                        // Parse as expressions to handle both simple paths and call-like validators
                        // e.g., `NonEmptyValidation::<_>` and `PrefixValidation::<_>(prefix = "Xx")`
                        if let Ok(exprs) = attr.parse_args_with(
                            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
                        ) {
                            for expr in exprs {
                                // Extract the validator name from the expression
                                let validator_name = match &expr {
                                    // Handle call expressions like `PrefixValidation::<_>(prefix = "Xx")`
                                    syn::Expr::Call(call) => {
                                        extract_path_last_ident(&call.func)
                                    }
                                    // Handle simple path expressions like `NonEmptyValidation::<_>`
                                    syn::Expr::Path(path_expr) => {
                                        path_expr.path.segments.last().map(|s| s.ident.to_string())
                                    }
                                    _ => None,
                                };
                                if let Some(name) = validator_name {
                                    validations.push(name);
                                }
                            }
                        }
                    }
                }
                Some((ident, validations))
            })
            .collect(),
        _ => HashMap::new(),
    };

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

    let (field_structure_tokens, field_base_declarations_tokens, should_be_unwrapped): (
        Vec<TokenStream>,
        Vec<TokenStream>,
        HashMap<String, bool>,
    ) = component_field_pairs
        .into_iter()
        .map(|content| {
            (
                content.field_structure_tokens,
                content.field_base_declarations_tokens,
                content.should_be_unwrapped,
            )
        })
        .multiunzip();

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
                let validation_rules = koruma_validations
                    .get(&field_name_str)
                    .cloned()
                    .unwrap_or_default();
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

    let model_options = unwrapped_core::Opts::builder()
        .suffix(format_ident!("FormValueHolder"))
        .build();

    let macro_options =
        unwrapped_core::ProcUsageOpts::new(should_be_unwrapped, Some(format_ident!("gpui_form")));

    let model_struct = unwrapped_core::unwrapped(&derive_input, Some(model_options), macro_options);

    let expanded = quote! {
        #model_struct
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

    fn render(tokens: proc_macro2::TokenStream) -> String {
        let derive_input = syn::parse2::<syn::DeriveInput>(tokens)
            .expect("input should parse into a derive input");

        let expanded = super::expand_gpui_form(
            derive_input,
            GpuiFormOptions {
                generate_shape: false,
            },
        );

        let file = syn::parse2::<syn::File>(expanded).expect("macro output should parse back");

        prettyplease::unparse(&file)
    }

    #[test]
    fn renders_standard_components() {
        let input = quote! {
            struct StandardForm {
                #[gpui_form(component(input))]
                title: String,
                #[gpui_form(component(number_input))]
                count: Option<i64>,
                #[gpui_form(component(checkbox))]
                is_admin: bool,
                #[gpui_form(component(switch))]
                is_active: Option<bool>,
                #[gpui_form(component(date_picker))]
                availability: chrono::NaiveDate,
            }
        };

        insta::assert_snapshot!("standard_components", render(input));
    }

    #[test]
    fn renders_select_and_custom_components() {
        let input = quote! {
            struct AdvancedForm {
                #[gpui_form(component(select(searchable, index = Country::France)))]
                country: Country,
                #[gpui_form(component(select(partial, default)))]
                language: Language,
                #[gpui_form(component(custom(name = ExtraFancy, uw)))]
                bio: Option<ExtraFancyValue>,
            }
        };

        insta::assert_snapshot!("select_and_custom_components", render(input));
    }
}
