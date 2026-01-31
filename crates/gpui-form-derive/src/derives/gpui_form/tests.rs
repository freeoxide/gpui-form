#[cfg(test)]
mod tests {
    use super::super::*;
    use koruma_derive_core::ParseFieldResult;
    use quote::quote;
    use syn::DeriveInput;

    #[test]
    fn test_koruma_field_parsing_with_cfg_attr() {
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
                #[cfg_attr(feature = "validation", koruma(koruma_collection::general::RequiredValidation::<Option<_>>))]
                name: String,

                #[gpui_form(component(number_input))]
                #[cfg_attr(feature = "validation", koruma(koruma_collection::numeric::PositiveValidation::<_>))]
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
}
