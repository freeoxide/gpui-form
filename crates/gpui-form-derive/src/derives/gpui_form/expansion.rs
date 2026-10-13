use darling::FromDeriveInput as _;
use itertools::Itertools as _;
use koruma_derive_core::{FieldInfo as KorumaFieldInfo, ParseFieldResult};
use proc_macro2::TokenStream;
use quote::{ToTokens as _, format_ident, quote};
use std::collections::HashMap;
use syn::DeriveInput;

use crate::derives::gpui_form::cfg_attr::flatten_cfg_attr_in_derive_input;
use crate::derives::gpui_form::components::generate_component_field;
use crate::derives::gpui_form::field_path::generate_field_path;
use crate::derives::gpui_form::form_state::generate_form_state;
use crate::derives::gpui_form::schema_export::generate_schema_export;
use crate::derives::gpui_form::structs::{
    ComponentStruct, FieldComposition, FieldOptionality, GpuiFormOptions, RequiredIfCondition,
    RequiredIfRule,
};
use crate::derives::gpui_form::submit::generate_submit_alias;
use crate::derives::gpui_form::utils::{
    canonical_token_string, canonical_type_string, extract_option_inner_type,
    extract_vec_inner_type, nested_form_holder_type,
};
use crate::derives::gpui_form::value_holder::{
    form_base_type, generate_value_holder, parse_field_default, should_wrap,
};

fn option_expr_string_tokens(expr: &Option<syn::Expr>) -> TokenStream {
    match expr {
        Some(expr) => {
            // Canonical Rust spelling, not the proc-macro printer's token
            // spacing: this string IS the exported schema surface.
            let expr_str = canonical_token_string(expr.to_token_stream());
            quote! { Some(#expr_str) }
        },
        None => quote! { None },
    }
}

pub fn expand_gpui_form(
    derive_input: DeriveInput,
    options: GpuiFormOptions,
) -> proc_macro2::TokenStream {
    let original_input = derive_input.clone();
    let derive_input = flatten_cfg_attr_in_derive_input(derive_input);

    let parsed = match ComponentStruct::from_derive_input(&derive_input) {
        Ok(parsed) => parsed,
        Err(e) => return e.write_errors(),
    };

    let struct_name = &parsed.ident;
    let components_holder_name = format_ident!("{}FormFields", struct_name);
    let components_base_declarations_name = format_ident!("{}FormComponents", struct_name);

    let koruma_options = parsed.koruma.as_ref().map(|k| k.0.clone());

    if parsed.empty {
        let enable_koruma = koruma_options.is_some();
        let enable_koruma_fluent = koruma_options.as_ref().map(|k| k.fluent).unwrap_or(false);
        let empty_fields: Vec<FieldOptionality> = Vec::new();
        let (value_holder_tokens, _, needs_koruma_derive) = generate_value_holder(
            &original_input,
            &empty_fields,
            enable_koruma,
            enable_koruma_fluent,
        );
        // Feature #8 (FLAT v1): emit the typed path type even for empty forms
        // (zero non-skipped fields => no per-field constructors, just the
        // wrapper with new()/path()/into_path()).
        let field_path_tokens = generate_field_path(&original_input, &empty_fields);
        // Field-level state model (M1): the facade + patch are emitted for
        // empty forms too, over an empty holder (zero per-field accessors).
        // No validation cache: there is nothing to validate.
        let form_state_tokens = generate_form_state(&original_input, &empty_fields, false);
        // Submit lifecycle (M2, backlog #7): the opt-in alias is legal on
        // empty forms too — an empty koruma form still has a ValidateExt
        // error type.
        let submit_alias_tokens =
            generate_submit_alias(&original_input, parsed.submit, needs_koruma_derive);
        // Schema export (M5, backlog #14): the empty shape chain lives once
        // in the export's `SHAPE` const; the inventory submission references
        // it (single metadata source). The export itself needs no inventory
        // feature.
        let empty_shape_construction = quote! {
            ::gpui_form::schema::registry::GpuiFormShape::new(
                stringify!(#struct_name),
                &[],
                file!(),
                #enable_koruma
            ).with_skipped_fields(false)
        };
        let schema_ident = format_ident!("{}FormSchema", struct_name);
        let shape_impl = if options.generate_shape {
            quote! {
                ::gpui_form::schema::registry::inventory::submit! {
                    #schema_ident::SHAPE
                }
            }
        } else {
            quote! {}
        };
        let schema_export_tokens = generate_schema_export(&parsed.ident, empty_shape_construction);

        return quote! {
            #value_holder_tokens
            #field_path_tokens
            #form_state_tokens
            #submit_alias_tokens
            #schema_export_tokens
            pub struct #components_holder_name;

            #shape_impl

            pub struct #components_base_declarations_name;
        };
    }

    let fields_iter = match &parsed.data {
        darling::ast::Data::Struct(s) => &s.fields,
        _ => unreachable!("GpuiForm derive only supports named structs"),
    };

    let has_skipped_fields = fields_iter.iter().any(|field| field.skip());

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

    let enable_koruma = koruma_options.is_some() || !parsed_koruma_fields.is_empty();
    let enable_koruma_fluent = koruma_options.as_ref().map(|k| k.fluent).unwrap_or(false);
    // Milestone M4 (backlog #13): `required_if` desugars to a koruma
    // validator on the holder, so a form using it is a koruma form even
    // without any other koruma opt-in — otherwise the rule would be
    // display-only metadata, which backlog #13 explicitly rejects.
    let has_required_if_fields = fields_iter
        .iter()
        .any(|field| !field.skip() && field.required_if.is_some());
    let enable_koruma = enable_koruma || has_required_if_fields;

    let mut koruma_validations: HashMap<String, Vec<String>> = parsed_koruma_fields
        .iter()
        .map(|(name, info)| {
            let mut validator_names: Vec<String> = info
                .validation
                .field_validators
                .iter()
                .map(|v| v.name().to_string())
                .collect();

            if info.is_newtype() {
                validator_names.push("NewtypeValidation".to_string());
            }

            if info.is_nested() {
                validator_names.push("NestedValidation".to_string());
            }

            (name.clone(), validator_names)
        })
        .collect();

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

    let component_field_pairs: Vec<crate::derives::gpui_form::structs::ComponentFieldContent> =
        fields_iter
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

    let mut field_optionality = Vec::new();
    let mut composition_errors: Vec<syn::Error> = Vec::new();
    // Milestone M4 (backlog #12/#13): raw `required_if` sibling references,
    // resolved against the full field list in a second pass below.
    let mut raw_required_if: Vec<Option<syn::Ident>> = Vec::new();
    for field in fields_iter {
        let field_name = field.ident.clone().unwrap();
        let field_name_str = field_name.to_string();
        let (was_optional, inner_type) = extract_option_inner_type(&field.ty);
        let wrap_in_option = !field.skip()
            && field.component.is_some()
            && wrap_in_option_map
                .get(&field_name_str)
                .copied()
                .unwrap_or(false);
        let koruma_info = parsed_koruma_fields.get(&field_name_str);
        let validation = koruma_info
            .map(|info| info.validation.clone())
            .unwrap_or_default();
        let default_expr = parse_field_default(field);

        // Milestone M4 (backlog #12/#13): both dependency attributes live on
        // component fields — the schema `FieldVariant` that records them is
        // only emitted for component fields, and a rule invisible to metadata
        // would silently degrade into runtime-only behavior.
        if field.component.is_none() && (field.depends_on.is_some() || field.required_if.is_some())
        {
            composition_errors.push(syn::Error::new_spanned(
                &field.ty,
                format!(
                    "field `{field_name_str}`: `depends_on`/`required_if` require a \
                     `component(...)` attribute on the same field"
                ),
            ));
        }

        // Milestone M3 (backlog #2/#3): resolve the composed holder shape.
        // Flat keeps the pre-M3 behavior byte-for-byte.
        let composition = if field.is_nested() {
            match nested_form_holder_type(&inner_type) {
                Some(holder) => FieldComposition::Nested { holder },
                None => {
                    composition_errors.push(syn::Error::new_spanned(
                        &field.ty,
                        format!(
                            "`component(nested)` on field `{field_name_str}` requires a named \
                             struct type; found a non-path type"
                        ),
                    ));
                    FieldComposition::Flat
                },
            }
        } else if field.is_list() {
            match extract_vec_inner_type(&inner_type) {
                Some(item) => FieldComposition::List { item },
                None => {
                    composition_errors.push(syn::Error::new_spanned(
                        &field.ty,
                        format!(
                            "`component(list(...))` on field `{field_name_str}` requires the \
                             field type to be Vec<T> (or Option<Vec<T>>)"
                        ),
                    ));
                    FieldComposition::Flat
                },
            }
        } else {
            FieldComposition::Flat
        };

        field_optionality.push(FieldOptionality {
            field_name,
            original_type: field.ty.clone(),
            inner_type,
            was_optional,
            wrap_in_option,
            validation,
            default_expr,
            override_type: field.r#type.as_ref().map(|ty| ty.0.clone()),
            into_expr: field.into.clone(),
            from_expr: field.from.clone(),
            skip: field.skip(),
            composition,
            depends_on: field.depends_on.as_ref().map(|sibling| sibling.0.clone()),
            required_if: None,
        });
        raw_required_if.push(field.required_if.as_ref().map(|sibling| sibling.0.clone()));
    }

    // Milestone M4 (backlog #12/#13): resolve sibling references.
    //
    // Both attributes name a sibling field of the SAME struct; the
    // reference is existence-checked here so a typo fails at derive time.
    // `required_if` additionally validates the target/sibling shapes and
    // resolves the emitted validator identity plus the sibling's
    // truthiness expression (R19: typed references desugared to a koruma
    // validator closure — never a stringly expression language).
    //
    // Both attributes also require `component(...)` on the carrying field:
    // the schema `FieldVariant` (their metadata home) is only emitted for
    // component fields, and a rule invisible to metadata would silently
    // degrade into runtime-only behavior.
    {
        let field_indexes: HashMap<String, usize> = field_optionality
            .iter()
            .enumerate()
            .filter(|(_, field)| !field.skip)
            .map(|(index, field)| (field.field_name.to_string(), index))
            .collect();

        for index in 0..field_optionality.len() {
            let field_name = field_optionality[index].field_name.to_string();

            if let Some(depends_on) = field_optionality[index].depends_on.clone() {
                let depends_on_str = depends_on.to_string();
                match field_indexes.get(&depends_on_str) {
                    Some(&sibling_index) if sibling_index == index => {
                        // Same identity guard `required_if` applies: a field
                        // cannot depend on itself (it would reset itself on
                        // its own change).
                        composition_errors.push(syn::Error::new_spanned(
                            &depends_on,
                            format!(
                                "`depends_on = {depends_on_str}` on field `{field_name}` must \
                                 name a different field"
                            ),
                        ));
                    },
                    None => {
                        composition_errors.push(syn::Error::new_spanned(
                            &depends_on,
                            format!(
                                "`depends_on = {depends_on_str}` on field `{field_name}` must \
                                 name a non-skipped field of this struct"
                            ),
                        ));
                    },
                    Some(_) => {},
                }
            }

            if let Some(sibling) = raw_required_if[index].clone() {
                let sibling_str = sibling.to_string();
                let Some(&sibling_index) = field_indexes.get(&sibling_str) else {
                    composition_errors.push(syn::Error::new_spanned(
                        &sibling,
                        format!(
                            "`required_if = {sibling_str}` on field `{field_name}` must name a \
                             non-skipped field of this struct"
                        ),
                    ));
                    continue;
                };
                if sibling_index == index {
                    composition_errors.push(syn::Error::new_spanned(
                        &sibling,
                        format!(
                            "`required_if = {sibling_str}` on field `{field_name}` must name a \
                             different field"
                        ),
                    ));
                    continue;
                }

                let target = &field_optionality[index];
                let target_option_wrapped = target.was_optional || target.wrap_in_option;
                if !target.composition.is_flat() || !target_option_wrapped {
                    composition_errors.push(syn::Error::new_spanned(
                        &sibling,
                        format!(
                            "`required_if` on field `{field_name}` requires a flat field whose \
                             holder stores `Option<T>` (an `Option<T>` source field, or a \
                             component that wraps in Option)"
                        ),
                    ));
                    continue;
                }

                let controller = &field_optionality[sibling_index];
                let controller_base = form_base_type(controller);
                let controller_wrapped = should_wrap(controller);
                let controller_is_bool = matches!(&controller_base, syn::Type::Path(type_path)
                    if type_path.path.is_ident("bool"));
                let condition = if !controller.composition.is_flat() {
                    composition_errors.push(syn::Error::new_spanned(
                        &sibling,
                        format!(
                            "`required_if = {sibling_str}` sibling must be a flat bool or \
                             Option field; composed (nested/list) fields cannot control \
                             conditional requiredness in v1"
                        ),
                    ));
                    continue;
                } else if controller_wrapped {
                    if controller_is_bool {
                        RequiredIfCondition::SomeTrue
                    } else {
                        RequiredIfCondition::IsSome
                    }
                } else if controller_is_bool {
                    RequiredIfCondition::Bool
                } else {
                    composition_errors.push(syn::Error::new_spanned(
                        &sibling,
                        format!(
                            "`required_if = {sibling_str}` sibling must hold a bool or Option \
                             value in the holder (found `{}`)",
                            controller_base.to_token_stream()
                        ),
                    ));
                    continue;
                };

                field_optionality[index].required_if = Some(RequiredIfRule {
                    sibling,
                    validator: format_ident!("{}FormRequiredIfValidation", struct_name),
                    condition,
                });
            }
        }

        // Metadata: the resolved `required_if` fields record the stable
        // "RequiredIfValidation" rule kind in their FieldVariant validation
        // list (schema consumers/prototyping see the rule alongside the
        // sibling reference emitted in the variant chain below).
        for (index, field) in field_optionality.iter().enumerate() {
            if raw_required_if[index].is_some() && field.required_if.is_some() {
                koruma_validations
                    .entry(field.field_name.to_string())
                    .or_default()
                    .push("RequiredIfValidation".to_string());
            }
        }
    }

    if let Some(first) = composition_errors.into_iter().reduce(|mut all, next| {
        all.combine(next);
        all
    }) {
        return first.to_compile_error();
    }

    // The holder generation below is fed plain `enable_koruma`: the call
    // site used to add a `has_fields_needing_required &&
    // has_any_koruma_validations` disjunct, but that term was absorbed —
    // every field's `validation` is populated from `parsed_koruma_fields`,
    // and any entry in that map already forces `enable_koruma` above (the
    // same absorption as the inner gate inside `generate_value_holder`).
    let (value_holder_tokens, fields_requiring_required, needs_koruma_derive) =
        generate_value_holder(
            &original_input,
            &field_optionality,
            enable_koruma,
            enable_koruma_fluent,
        );
    // Feature #8 (FLAT v1): emit the typed `<Name>FormPath` next to the value
    // holder. One constructor per non-skipped field; skipped fields are absent.
    let field_path_tokens = generate_field_path(&original_input, &field_optionality);
    // Field-level state model (M1): `<Name>FormState` facade + `<Name>FormPatch`
    // next to the path type, keyed by the typed paths above.
    //
    // Validation-state cache (M2, [D6]): gated on `needs_koruma_derive` —
    // the same gate that derives Koruma on the holder, NOT
    // `koruma_options.is_some()`, so field-attr-only koruma forms keep the
    // surface (hazard 6) — restricted to non-generic structs because the
    // `<Name>FormFieldErrors` map it stores is not emitted for generics.
    let enable_validation_cache = needs_koruma_derive && original_input.generics.params.is_empty();
    let form_state_tokens =
        generate_form_state(&original_input, &field_optionality, enable_validation_cache);
    // Submit lifecycle (M2, backlog #7): the opt-in
    // `<Name>FormSubmission` alias, gated on the attribute and
    // `needs_koruma_derive` for its error type.
    let submit_alias_tokens =
        generate_submit_alias(&original_input, parsed.submit, needs_koruma_derive);

    let field_variant_construction_code: Vec<TokenStream> = fields_iter
        .iter()
        .filter_map(|field| {
            if field.skip() {
                None
            } else if let Some(component_def) = field.component.as_ref() {
                let field_name_str = field
                    .ident
                    .as_ref()
                    .expect("Field should have an ident if not skipped and has component")
                    .to_string();
                let (was_optional, original_inner_type) = extract_option_inner_type(&field.ty);
                let base_type = field
                    .r#type
                    .as_ref()
                    .map(|ty| extract_option_inner_type(&ty.0).1)
                    .unwrap_or_else(|| original_inner_type.clone());
                let wraps_in_option = wrap_in_option_map
                    .get(&field_name_str)
                    .copied()
                    .unwrap_or(false);

                let is_optional = was_optional;
                // Canonical Rust spelling (`Vec<String>`), not the
                // proc-macro printer's token spacing (`Vec < String >`):
                // these strings ARE the exported schema surface.
                let field_type_str = canonical_type_string(&base_type);
                let source_value_type_str = canonical_type_string(&original_inner_type);
                let behaviour_tokens = component_def.behaviour_tokens(&base_type);
                let mut validation_rules = koruma_validations
                    .get(&field_name_str)
                    .cloned()
                    .unwrap_or_default();

                // Metadata gate mirrors the runtime gate in value_holder.rs:
                // the RequiredValidation attr is mirrored (and enforced by
                // the generated validate()) only on koruma forms, so a plain
                // form must not export required metadata its runtime never
                // enforces.
                if enable_koruma
                    && fields_requiring_required.contains(&field_name_str)
                    && !validation_rules.contains(&"RequiredValidation".to_string())
                {
                    validation_rules.insert(0, "RequiredValidation".to_string());
                }

                let validation_literals: Vec<_> = validation_rules
                    .iter()
                    .map(|v| syn::LitStr::new(v, proc_macro2::Span::call_site()))
                    .collect();

                let default_expr_tokens = field.default.as_ref().map(|expr| {
                    // Canonical spelling: the default expression is exported
                    // schema metadata, not just a runtime binding.
                    let expr_str = canonical_token_string(expr.0.to_token_stream());
                    quote! { .with_default(#expr_str) }
                });

                let custom_component_tokens = component_def.custom_component_tokens();
                let custom_shape_tokens = component_def.custom_shape_tokens();
                let custom_value_binding_tokens = component_def.custom_value_binding_tokens();
                let from_expr_tokens = option_expr_string_tokens(&field.from);
                let into_expr_tokens = option_expr_string_tokens(&field.into);
                // Async-validation attribute surface (M2 emission sketch):
                // `validate(async_fn = <path>)` records the validator path as
                // metadata only — nothing executes it until the future
                // async-validation wiring phase.
                let async_validator_tokens = match &field.validate {
                    Some(async_validate) => {
                        // Canonical spelling: the path is exported schema
                        // metadata (`::` glued, not `my :: check`).
                        let path_str =
                            canonical_token_string(async_validate.async_fn.0.to_token_stream());
                        quote! { .with_async_validator(Some(#path_str)) }
                    },
                    None => quote! {},
                };
                // Milestone M4 (backlog #12/#13): dependency payloads — the
                // sibling's field name, i.e. the FieldPath name-segment
                // representation (const-constructible `&'static str`, the
                // inventory contract).
                let depends_on_tokens = match field.depends_on.as_ref() {
                    Some(sibling) => {
                        let sibling_str = sibling.0.to_string();
                        quote! { .with_depends_on(Some(#sibling_str)) }
                    },
                    None => quote! {},
                };
                let required_if_tokens = match field.required_if.as_ref() {
                    Some(sibling) => {
                        let sibling_str = sibling.0.to_string();
                        quote! { .with_required_if(Some(#sibling_str)) }
                    },
                    None => quote! {},
                };

                // Feature #4 (METADATA-FIRST v1): emit non-rendering layout
                // hints. Each string hint contributes a builder call only when
                // present; width is always emitted (Full when absent, matching
                // `FieldLayout::new()`'s default).
                let layout_section = match field.section.as_deref() {
                    Some(v) => quote! { .with_section(Some(#v)) },
                    None => quote! {},
                };
                let layout_label = match field.label.as_deref() {
                    Some(v) => quote! { .with_label(Some(#v)) },
                    None => quote! {},
                };
                let layout_description = match field.description.as_deref() {
                    Some(v) => quote! { .with_description(Some(#v)) },
                    None => quote! {},
                };
                let layout_placeholder = match field.placeholder.as_deref() {
                    Some(v) => quote! { .with_placeholder(Some(#v)) },
                    None => quote! {},
                };
                let layout_width = match field.width {
                    Some(crate::derives::gpui_form::structs::LayoutWidthMeta::Full) => {
                        quote! { ::gpui_form::schema::layout::LayoutWidth::Full }
                    },
                    Some(crate::derives::gpui_form::structs::LayoutWidthMeta::Half) => {
                        quote! { ::gpui_form::schema::layout::LayoutWidth::Half }
                    },
                    Some(crate::derives::gpui_form::structs::LayoutWidthMeta::Third) => {
                        quote! { ::gpui_form::schema::layout::LayoutWidth::Third }
                    },
                    None => quote! { ::gpui_form::schema::layout::LayoutWidth::Full },
                };
                let layout_tokens = quote! {
                    .with_layout(
                        ::gpui_form::schema::layout::FieldLayout::new()
                            #layout_section
                            #layout_label
                            #layout_description
                            #layout_placeholder
                            .with_width(#layout_width)
                    )
                };

                Some(quote! {
                    ::gpui_form::schema::registry::FieldVariant::new(
                        #field_name_str,
                        #field_type_str,
                        #is_optional,
                        #behaviour_tokens
                    )
                    .with_source_value_type(#source_value_type_str)
                    .with_wraps_in_option(#wraps_in_option)
                    .with_conversions(#from_expr_tokens, #into_expr_tokens)
                    .with_validations(&[
                        #( #validation_literals ),*
                    ])
                    #default_expr_tokens
                    #custom_component_tokens
                    #custom_shape_tokens
                    #custom_value_binding_tokens
                    #layout_tokens
                    #async_validator_tokens
                    #depends_on_tokens
                    #required_if_tokens
                })
            } else {
                None
            }
        })
        .collect();

    // Schema export (M5, backlog #14): the `FieldVariant` construction
    // chain is emitted ONCE — as the `<Name>FormSchema::SHAPE` const — and
    // the inventory submission (behind the derive crate's `inventory`
    // feature) references that const. `inventory::submit!` evaluates its
    // value inside a `static` initializer (`value: &{ <expr> }`), so a
    // const path is a legal value; naming the const instead of re-emitting
    // the chain is the "single metadata source" contract made structural:
    // nested/list (M3) and dependency (M4) payloads cannot drift between
    // the inventory surface and the export because there is only one chain.
    let shape_construction = quote! {
        ::gpui_form::schema::registry::GpuiFormShape::new(
            stringify!(#struct_name),
            &[
                #(#field_variant_construction_code),*
            ],
            file!(),
            #enable_koruma
        ).with_skipped_fields(#has_skipped_fields)
    };
    let schema_ident = format_ident!("{}FormSchema", struct_name);
    let shape_impl = if options.generate_shape {
        quote! {
            ::gpui_form::schema::registry::inventory::submit! {
                #schema_ident::SHAPE
            }
        }
    } else {
        quote! {}
    };
    let schema_export_tokens = generate_schema_export(&parsed.ident, shape_construction);

    let expanded = quote! {
        #value_holder_tokens
        #field_path_tokens
        #form_state_tokens
        #submit_alias_tokens
        #schema_export_tokens
        pub struct #components_holder_name {
            #(#field_structure_tokens)*
        }

        #shape_impl

        pub struct #components_base_declarations_name;

        impl #components_base_declarations_name {
          #(#field_base_declarations_tokens)*
        }
    };

    expanded
}
