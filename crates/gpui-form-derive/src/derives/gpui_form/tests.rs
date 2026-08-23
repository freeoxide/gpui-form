#[cfg(test)]
mod gpui_form_tests {
    use super::super::*;
    use crate::derives::gpui_form::koruma;
    use koruma_derive_core::{ParseFieldResult, ValidatorAttr};
    use quote::quote;
    use syn::DeriveInput;

    fn compact_tokens(tokens: &str) -> String {
        tokens.chars().filter(|c| !c.is_whitespace()).collect()
    }

    #[test]
    fn field_path_type_emitted_next_to_value_holder() {
        // Feature #8 (FLAT v1): the expansion emits a `<Name>FormPath` newtype
        // alongside the value holder. `<Name>` is the source struct ident, so a
        // struct named `Profile` gets `ProfileFormPath` (mirroring
        // `ProfileFormValueHolder`). The constructor name matches the field.
        use crate::derives::gpui_form::expansion::expand_gpui_form;
        use crate::derives::gpui_form::structs::GpuiFormOptions;

        let input: DeriveInput = syn::parse_quote! {
            struct Profile {
                name: String,
                email: String,
            }
        };
        let out = expand_gpui_form(
            input,
            GpuiFormOptions {
                generate_shape: false,
            },
        );
        let s = out.to_string();

        // The path type appears after the value holder (both derive from the
        // source struct ident).
        assert!(
            s.contains("ProfileFormPath"),
            "no path type in expansion: {s}"
        );
        assert!(s.contains("ProfileFormValueHolder"), "no value holder: {s}");
        // Per-field constructors named after the fields. Tokenized output
        // spaces out `fn name` / `fn email`, so match on the freestanding form.
        assert!(s.contains("fn name"), "no name ctor: {s}");
        assert!(s.contains("fn email"), "no email ctor: {s}");
        // Trait impls are emitted unconditionally (no feature gating).
        assert!(s.contains("Deref"), "no Deref: {s}");
        assert!(s.contains("AsRef"), "no AsRef: {s}");
        assert!(s.contains("Display"), "no Display: {s}");
        // No generics on the path type (paths are field-name only).
        assert!(
            !s.contains("ProfileFormPath <"),
            "path type must not be generic: {s}"
        );
    }

    #[test]
    fn test_koruma_field_parsing_with_cfg_attr() {
        let tokens = quote! {
            struct Test {
                #[cfg_attr(feature = "validation", koruma(SomeValidator::<_>::builder()))]
                field: u32,
            }
        };
        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();

        if let syn::Data::Struct(data_struct) = &derive_input.data {
            let field = data_struct.fields.iter().next().unwrap();
            let result = koruma_derive_core::parse_field(field, 0);

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
        let tokens = quote! {
            #[derive(GpuiForm)]
            #[gpui_form(koruma(fluent))]
            struct TestForm {
                #[gpui_form(component(input))]
                #[cfg_attr(feature = "validation", koruma(koruma_collection::general::RequiredValidation::<Option<_>>::builder()))]
                name: String,

                #[gpui_form(component(number_input))]
                #[cfg_attr(feature = "validation", koruma(koruma_collection::numeric::PositiveValidation::<_>::builder()))]
                age: u32,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();

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

        let expanded = expansion::expand_gpui_form(
            derive_input.clone(),
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let expanded_str = expanded.to_string();

        assert!(
            expanded_str.contains("Koruma") || expanded_str.contains("koruma"),
            "Generated code should include Koruma-related types"
        );

        assert!(
            expanded_str.contains("validation_errors") || expanded_str.contains("validate"),
            "Generated code should include validation error handling: {}",
            &expanded_str[..expanded_str.len().min(500)]
        );

        assert!(
            expanded_str.contains("FieldVariant") || expanded_str.contains("validations"),
            "Generated code should include field variant metadata"
        );
    }

    #[test]
    fn test_real_world_common_v_read_pattern() {
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

        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let expanded_str = expanded.to_string();
        eprintln!("=== Generated code (first 1000 chars) ===");
        eprintln!("{}", &expanded_str[..expanded_str.len().min(1000)]);

        assert!(
            expanded_str.contains("with_validations"),
            "Generated GpuiFormShape should call with_validations()"
        );
    }

    #[test]
    fn test_koruma_enabled_without_validators_derives_validate() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            #[gpui_form(koruma(fluent))]
            struct OptionalOnlyForm {
                note: Option<String>,
                kind: Option<u8>,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let expanded_str = compact_tokens(&expanded.to_string());

        assert!(
            expanded_str.contains("::koruma::Koruma"),
            "Koruma derive should be emitted when gpui_form(koruma) is enabled, even without validators"
        );
        assert!(
            expanded_str.contains("::koruma::KorumaAllFluent"),
            "KorumaAllFluent derive should be emitted when gpui_form(koruma(fluent)) is enabled"
        );
    }

    #[test]
    fn test_validator_attr_to_tokens_normalizes_builder_chain() {
        let validator: ValidatorAttr = syn::parse_quote!(
            koruma_collection::numeric::RangeValidation::<_>::builder()
                .min(18)
                .max(167)
        );

        let tokens = koruma::validator_attr_to_tokens(&validator);
        let compact = compact_tokens(&tokens.to_string());

        assert_eq!(
            compact,
            compact_tokens(
                "koruma_collection::numeric::RangeValidation::<_>::builder().min(18).max(167)"
            )
        );
    }

    #[test]
    fn test_gpui_form_preserves_builder_chain_koruma_validators() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            #[gpui_form(koruma)]
            struct TestForm {
                #[gpui_form(component(number_input))]
                #[koruma(koruma_collection::numeric::RangeValidation::<_>::builder().min(18).max(167))]
                age: u32,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains(&compact_tokens(
                "koruma_collection::numeric::RangeValidation::<_>::builder().min(18).max(167)"
            )),
            "Generated value holder should preserve builder-chain koruma validators: {compact}"
        );
    }

    #[test]
    fn test_gpui_form_preserves_zero_setter_koruma_builder_chains() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            #[gpui_form(koruma)]
            struct TestForm {
                #[gpui_form(component(number_input))]
                #[koruma(koruma_collection::numeric::PositiveValidation::<_>::builder())]
                age: u32,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains(&compact_tokens(
                "koruma_collection::numeric::PositiveValidation::<_>::builder()"
            )),
            "Generated value holder should preserve zero-setter koruma builder chains: {compact}"
        );
    }

    #[test]
    fn test_gpui_form_emits_required_validation_as_builder_chain() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            #[gpui_form(koruma)]
            struct TestForm {
                #[gpui_form(component(input))]
                name: String,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains(&compact_tokens(
                "koruma_collection::general::RequiredValidation::<Option<_>>::builder()"
            )),
            "Generated value holder should emit synthetic required validation as a builder chain: {compact}"
        );
    }

    #[test]
    fn test_type_override_and_conversions() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(
                    type = chrono::NaiveDate,
                    from = |ts| to_form(ts),
                    into = |dt| to_model(dt),
                    component(date_picker)
                )]
                birth_date: Option<Timestamp>,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("FieldVariant::new(\"birth_date\",\"chrono::NaiveDate\",true"),
            "FieldVariant should use override type for metadata"
        );

        assert!(
            compact.contains("birth_date:from.birth_date.map(") && compact.contains("to_form"),
            "From<Original> for FormValueHolder should apply `from` conversion"
        );

        assert!(
            compact.contains("birth_date:from.birth_date.map(") && compact.contains("to_model"),
            "From<FormValueHolder> for Original should apply `into` conversion"
        );
    }

    #[test]
    fn test_number_input_override_keeps_full_type_path() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(
                    type = rust_decimal::Decimal,
                    from = |value| value,
                    into = |value| value,
                    component(number_input(as = f64))
                )]
                amount: f64,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("FieldVariant::new(\"amount\",\"rust_decimal::Decimal\",false"),
            "FieldVariant should keep the fully-qualified override type in metadata"
        );
        assert!(
            compact.contains("validate_signed_numeric::<f64>(value,true)"),
            "Number input validation should parse against the validation override type"
        );
        assert!(
            compact.contains(
                "ComponentsBehaviour::NumberInput(::gpui_form::schema::components::NumberInputBehaviour{validation_type:Some(\"f64\"),kind:::gpui_form::schema::components::NumberInputKind::Float,})"
            ),
            "Number input metadata should preserve the validation override and numeric family"
        );
    }

    #[test]
    fn test_number_input_unsigned_override_drives_validation_family() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(
                    type = crate::ids::AccountId,
                    from = crate::ids::AccountId::from_u64,
                    into = crate::ids::AccountId::into_u64,
                    component(number_input(as = u64))
                )]
                account_id: u64,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("validate_unsigned_numeric::<u64>(value,true)"),
            "Unsigned validation override should parse against the override type"
        );
        assert!(
            compact.contains(
                "ComponentsBehaviour::NumberInput(::gpui_form::schema::components::NumberInputBehaviour{validation_type:Some(\"u64\"),kind:::gpui_form::schema::components::NumberInputKind::UnsignedInteger,})"
            ),
            "Unsigned validation override should drive number input metadata"
        );
    }

    #[test]
    fn test_skipped_fields_still_generate_from_original() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(
                    type = chrono::NaiveDate,
                    from = |ts| to_form(ts),
                    into = |dt| to_model(dt),
                    component(date_picker)
                )]
                birth_date: Option<Timestamp>,

                #[gpui_form(skip)]
                skip_me: bool,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            !compact.contains("compile_error!"),
            "skip + from should no longer emit a compile_error"
        );
        assert!(
            compact.contains("impl::core::convert::From<TestForm>forTestFormFormValueHolder"),
            "From<Original> for FormValueHolder should be generated even with skipped fields"
        );
        assert!(
            compact.contains("birth_date:from.birth_date.map(") && compact.contains("to_form"),
            "From<Original> for FormValueHolder should still apply `from` conversion"
        );
        assert!(
            !compact.contains("impl::core::convert::From<TestFormFormValueHolder>forTestForm"),
            "Reverse From<FormValueHolder> for Original should remain disabled when skipped fields exist"
        );
        assert!(
            compact.contains(
                "pubfninto_original(self,skip_me:bool)->Result<TestForm,TestFormFormValueHolderConversionError>"
            ),
            "Skipped-field forms should keep strict into_original(self, skipped...) conversion"
        );
    }

    #[test]
    fn test_present_fields_json_uses_into_converted_debug_values_for_skipped_forms() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(
                    type = chrono::NaiveDate,
                    into = |dt| to_model(dt),
                    component(date_picker)
                )]
                birth_date: Option<Timestamp>,

                #[gpui_form(skip)]
                skip_me: bool,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("pubfnpresent_fields_json(&self)->String"),
            "Skipped-field value holders should generate present_fields_json()"
        );
        assert!(
            compact.contains(
                "letconverted=self.birth_date.clone().map(|value|(|dt|to_model(dt))(value));"
            ),
            "present_fields_json() should apply `into` conversion for optional override fields"
        );
        assert!(
            compact.contains("format!(\"{:?}\",converted)"),
            "present_fields_json() should emit debug-formatted converted values"
        );
    }

    #[test]
    fn test_default_uses_into_conversion() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(component(input), default = "test@example.com")]
                email: String,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("Into::into(\"test@example.com\")"),
            "Default should be wrapped in Into::into for string literals"
        );
    }

    #[test]
    fn test_skipped_forms_with_string_default_emit_typed_default_comparison() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(component(input), default = "test@example.com")]
                email: String,

                #[gpui_form(skip)]
                skip_me: bool,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("impl::core::convert::From<TestForm>forTestFormFormValueHolder"),
            "Skipped-field forms should still generate From<Original> for value holder"
        );
        assert!(
            compact.contains(
                "letdefault_original:String=::core::convert::Into::into(\"test@example.com\")"
            ),
            "From<Original> should emit a typed default value to avoid Into inference ambiguity"
        );
    }

    #[test]
    fn test_custom_component_generates_shape_based_state_and_constructor() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(component(custom(shape = crate::shapes::BioInputShape, component = crate::ui::BioInput, value_binding)))]
                bio: String,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("pubbio_custom:::gpui::Entity<")
                && compact.contains(
                    "<crate::shapes::BioInputShapeas::gpui_form::custom::CustomComponentShape>::State"
                ),
            "Custom component field should use shape state type"
        );

        assert!(
            compact.contains(
                "<crate::shapes::BioInputShapeas::gpui_form::custom::CustomComponentShape>::new(window,cx)"
            ),
            "Custom component constructor should delegate to shape::new"
        );

        assert!(
            compact.contains("ComponentsBehaviour::Custom"),
            "FieldVariant should carry Custom behaviour metadata"
        );

        assert!(
            compact.contains("with_custom_component("),
            "FieldVariant should carry the custom component path: {compact}"
        );
        assert!(
            compact.contains("with_custom_shape(\"crate::shapes::BioInputShape\")"),
            "FieldVariant should carry the custom shape path: {compact}"
        );
        assert!(
            compact.contains("with_custom_value_binding(true)"),
            "FieldVariant should record opt-in custom value binding: {compact}"
        );
    }

    #[test]
    fn test_field_variant_metadata_records_form_value_holder_conversion_shape() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(
                    component(input),
                    type = crate::types::AccountCode,
                    from = crate::types::AccountCode::new,
                    into = crate::types::AccountCode::into_string
                )]
                account_no: String,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact
                .contains("FieldVariant::new(\"account_no\",\"crate::types::AccountCode\",false"),
            "FieldVariant should store the form-side value type: {compact}"
        );
        assert!(
            compact.contains("with_source_value_type(\"String\")"),
            "FieldVariant should store the source model value type: {compact}"
        );
        assert!(
            compact.contains("with_wraps_in_option(true)"),
            "FieldVariant should store generated holder wrapping policy: {compact}"
        );
        assert!(
            compact.contains("with_conversions(Some(\"crate::types::AccountCode::new\"),Some(\"crate::types::AccountCode::into_string\"))"),
            "FieldVariant should store source/form conversion expressions: {compact}"
        );
    }

    #[test]
    fn test_custom_component_wraps_in_option_controls_value_holder_field() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(component(custom(shape = crate::shapes::ToggleShape, wraps_in_option = false)))]
                enabled: bool,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("pubenabled:bool"),
            "wraps_in_option = false should keep value holder field non-optional"
        );
        assert!(
            !compact.contains("pubenabled:Option<bool>"),
            "wraps_in_option = false should avoid wrapping in Option"
        );
    }

    #[test]
    fn test_custom_component_supports_state_alias() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(component(custom(state = crate::state::TagsState, wraps_in_option = false)))]
                tags: Vec<String>,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("pubtags_custom:::gpui::Entity<")
                && compact.contains(
                    "<crate::state::TagsStateas::gpui_form::custom::CustomComponentShape>::State"
                ),
            "`state = ...` should map to custom shape path"
        );
        assert!(
            compact.contains("pubtags:Vec<String>"),
            "wraps_in_option = false should keep field as Vec<String>"
        );
    }

    #[test]
    fn test_select_default_expression_initializes_component_selection() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(component(select), default = crate::defaults::country())]
                country: Country,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("let__gpui_form_default=crate::defaults::country()"),
            "Select component initialization should bind the full default expression once"
        );
        assert!(
            compact.contains(".position(|x|x==__gpui_form_default)"),
            "Select component initialization should compare against the bound default expression"
        );
        assert!(
            compact.contains(".map(::gpui_component::IndexPath::new)")
                && !compact.contains(".position(|x|x==__gpui_form_default).unwrap()"),
            "Select component initialization should skip invalid defaults instead of panicking"
        );
    }

    #[test]
    fn test_infinite_select_default_expression_and_max_depth_are_honored() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(component(infinite_select(max_depth = 2)), default = crate::defaults::country())]
                location: Country,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("let__gpui_form_default=crate::defaults::country()"),
            "InfiniteSelect initialization should bind the full default expression once"
        );
        assert!(
            compact.contains("new_with_options(__gpui_form_default,")
                && !compact.contains("new_with_options(crate::defaults::country(),"),
            "InfiniteSelect initialization should use the bound default expression for runtime construction"
        );
        assert!(
            compact.contains("InfiniteSelectState::new_with_options("),
            "InfiniteSelect initialization should pass the bound default expression into the runtime state"
        );
        assert!(
            compact.contains("InfiniteSelectStateOptions::default()")
                && compact.contains(".searchable(false)")
                && compact.contains(".max_depth(2"),
            "InfiniteSelect initialization should forward max_depth into the runtime options"
        );
    }

    // ------------------------------------------------------------------
    // Feature #4 (METADATA-FIRST v1): layout & section metadata.
    //
    // These tests drive `expand_gpui_form` directly and inspect the tokenized
    // output. The shape (inventory submission) is what carries the
    // `FieldVariant` chain, so `generate_shape: true` is used throughout and we
    // assert the emitted `.with_layout(...)` / `.with_section(...)` /
    // `.with_width(...)` fragments resolve against the facade path
    // `::gpui_form::schema::layout::...`.
    // ------------------------------------------------------------------

    #[test]
    fn layout_all_five_hints_plus_width_bare_ident_emit_chain() {
        // A field exercising every v1 hint, including the bare-ident width.
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct Signup {
                #[gpui_form(
                    section = "Account",
                    label = "Username",
                    description = "Shown publicly",
                    placeholder = "pick a name",
                    width = half,
                    component(input)
                )]
                username: String,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let compact = compact_tokenish(&expanded.to_string());

        // The facade path the derive is contracted to emit.
        assert!(
            compact.contains("::gpui_form::schema::layout::FieldLayout::new()"),
            "expected FieldLayout::new() via facade path: {compact}"
        );
        assert!(
            compact.contains(".with_section(Some(\"Account\"))"),
            "section hint not emitted: {compact}"
        );
        assert!(
            compact.contains(".with_label(Some(\"Username\"))"),
            "label hint not emitted: {compact}"
        );
        assert!(
            compact.contains(".with_description(Some(\"Shownpublicly\"))")
                || compact.contains(".with_description(Some(\"Shown publicly\"))"),
            "description hint not emitted: {compact}"
        );
        assert!(
            compact.contains(".with_placeholder(Some(\"pickaname\"))")
                || compact.contains(".with_placeholder(Some(\"pick a name\"))"),
            "placeholder hint not emitted: {compact}"
        );
        assert!(
            compact.contains(".with_width(::gpui_form::schema::layout::LayoutWidth::Half)"),
            "bare-ident width=half must map to LayoutWidth::Half: {compact}"
        );
        // The layout chain is appended to the FieldVariant construction.
        assert!(
            compact.contains(".with_layout("),
            "with_layout call missing: {compact}"
        );
    }

    #[test]
    fn layout_quoted_width_and_third_variant_map_correctly() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct Form {
                #[gpui_form(width = "third", component(input))]
                a: String,
                #[gpui_form(width = full, component(input))]
                b: String,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let compact = compact_tokenish(&expanded.to_string());

        assert!(
            compact.contains(".with_width(::gpui_form::schema::layout::LayoutWidth::Third)"),
            "quoted width=\"third\" must map to LayoutWidth::Third: {compact}"
        );
        assert!(
            compact.contains(".with_width(::gpui_form::schema::layout::LayoutWidth::Full)"),
            "bare width=full must map to LayoutWidth::Full: {compact}"
        );
    }

    #[test]
    fn layout_absent_hints_default_to_full_and_no_string_builders() {
        // No layout hints at all: width still defaults to Full (unconditional
        // builder), but none of the string `.with_*` builders appear.
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct Plain {
                #[gpui_form(component(input))]
                name: String,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains(".with_width(::gpui_form::schema::layout::LayoutWidth::Full)"),
            "absent width must default to Full: {compact}"
        );
        assert!(
            !compact.contains(".with_section("),
            "absent section must not emit with_section: {compact}"
        );
        assert!(
            !compact.contains(".with_label("),
            "absent label must not emit with_label: {compact}"
        );
        assert!(
            !compact.contains(".with_description("),
            "absent description must not emit with_description: {compact}"
        );
        assert!(
            !compact.contains(".with_placeholder("),
            "absent placeholder must not emit with_placeholder: {compact}"
        );
    }

    #[test]
    fn layout_on_skipped_field_is_ignored() {
        // A skipped field with layout hints: NO FieldVariant is emitted for it,
        // so the layout hints must not appear anywhere in the expansion. The
        // non-skipped field's width still defaults to Full.
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct Mixed {
                #[gpui_form(component(input))]
                visible: String,

                #[gpui_form(skip, section = "Secret", label = "Hidden", width = half)]
                #[allow(dead_code)]
                secret: String,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        // The skipped field's hints must NOT reach the FieldVariant chain.
        assert!(
            !compact.contains("\"Secret\""),
            "skipped field section leaked into expansion: {compact}"
        );
        assert!(
            !compact.contains("\"Hidden\""),
            "skipped field label leaked into expansion: {compact}"
        );
        assert!(
            !compact.contains("LayoutWidth::Half"),
            "skipped field width leaked into expansion: {compact}"
        );
        // Only ONE FieldVariant (the visible field), so exactly one with_layout
        // call and one with_width(Full).
        let layout_count = compact.matches(".with_layout(").count();
        assert_eq!(
            layout_count, 1,
            "expected exactly one FieldVariant (visible only): {compact}"
        );
    }

    #[test]
    fn layout_width_unknown_variant_is_rejected() {
        // An invalid width value should fail to parse (darling error), not
        // silently default. We exercise this through the full derive-input path
        // because that is where darling runs `FromMeta`.
        use darling::FromDeriveInput as _;
        let tokens = quote! {
            struct Form {
                #[gpui_form(width = bogus, component(input))]
                a: String,
            }
        };
        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let result =
            crate::derives::gpui_form::structs::ComponentStruct::from_derive_input(&derive_input);
        assert!(
            result.is_err(),
            "unknown width value must be rejected by the attribute parser"
        );
    }

    /// Like `compact_tokens` but keeps whitespace-to-single-space so string
    /// literals with spaces survive intact while still normalizing the token
    /// stream for substring matching.
    fn compact_tokenish(tokens: &str) -> String {
        tokens.split_whitespace().collect::<String>()
    }
}
