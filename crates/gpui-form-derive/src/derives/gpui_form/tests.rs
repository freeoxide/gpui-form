#[cfg(test)]
mod gpui_form_tests {
    use super::super::*;
    use quote::quote;
    use syn::DeriveInput;

    fn compact_tokens(tokens: &str) -> String {
        tokens.chars().filter(|c| !c.is_whitespace()).collect()
    }

    fn compact_tokenish(tokens: &str) -> String {
        tokens.split_whitespace().collect::<String>()
    }

    #[test]
    fn test_validation_issue_key_mapping_covers_koruma_collection_validators() {
        use crate::derives::gpui_form::value_holder::validation_issue_key_for_kind;

        let expected = [
            ("RequiredValidation", "validation.required"),
            ("NonEmptyValidation", "validation.non_empty"),
            ("LenValidation", "validation.len"),
            ("CreditCardValidation", "validation.credit_card"),
            ("EmailValidation", "validation.email"),
            ("IpValidation", "validation.ip"),
            ("PhoneNumberValidation", "validation.phone_number"),
            ("UrlValidation", "validation.url"),
            ("NegativeValidation", "validation.negative"),
            ("NonNegativeValidation", "validation.non_negative"),
            ("NonPositiveValidation", "validation.non_positive"),
            ("PositiveValidation", "validation.positive"),
            ("RangeValidation", "validation.range"),
            ("AlphanumericValidation", "validation.alphanumeric"),
            ("AsciiValidation", "validation.ascii"),
            ("ContainsValidation", "validation.contains"),
            ("MatchesValidation", "validation.matches"),
            ("PatternValidation", "validation.pattern"),
            ("PrefixValidation", "validation.prefix"),
            ("SuffixValidation", "validation.suffix"),
        ];

        for (kind, key) in expected {
            assert_eq!(
                validation_issue_key_for_kind(kind),
                key,
                "kind {kind} must map to its namespaced validation.* key"
            );
        }
        assert_eq!(
            validation_issue_key_for_kind("MysteryValidation"),
            "validation.mystery",
            "unlisted kinds still derive a namespaced key; the emitted match falls back to validation.invalid"
        );
    }

    // token-level: no behavior surface
    #[test]
    fn test_koruma_without_fluent_keeps_plain_koruma_derive() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            #[gpui_form(koruma)]
            struct PlainForm {
                #[gpui_form(component(input))]
                name: String,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
                generate_mcp: false,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("::koruma::Koruma") && !compact.contains("KorumaAllDisplay"),
            "koruma without fluent must not emit KorumaAllDisplay: {compact}"
        );
        assert!(
            !compact.contains("I18N_KEY_PREFIX") && !compact.contains("::rust_i18n::t!"),
            "koruma without fluent must not emit rust-i18n lookups: {compact}"
        );
    }

    // token-level: no behavior surface
    #[test]
    fn test_number_input_override_drives_validation_and_metadata() {
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
                generate_mcp: false,
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
            compact.contains("validation_type:Some(\"f64\")")
                && compact.contains("NumberInputKind::Float"),
            "Number input metadata should preserve the validation override and numeric family"
        );
        assert!(
            compact.contains("FieldVariant::new(\"account_id\",\"crate::ids::AccountId\",false"),
            "FieldVariant should keep the fully-qualified unsigned override type in metadata"
        );
        assert!(
            compact.contains("validate_unsigned_numeric::<u64>(value,true)"),
            "Unsigned validation override should parse against the override type"
        );
        assert!(
            compact.contains("validation_type:Some(\"u64\")")
                && compact.contains("NumberInputKind::UnsignedInteger"),
            "Unsigned validation override should drive number input metadata"
        );
    }

    // token-level: no behavior surface
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
                generate_mcp: false,
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

    // token-level: no behavior surface
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
                generate_mcp: false,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact
                .contains("FieldVariant::new(\"account_no\",\"crate::types::AccountCode\",false"),
            "FieldVariant should store the form-side value type: {compact}"
        );
        assert!(
            compact.contains("FieldVariant::new(\"birth_date\",\"chrono::NaiveDate\",true"),
            "Option source fields with an override type record optional=true and the override type: {compact}"
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

    // token-level: no behavior surface
    #[test]
    fn test_custom_component_wraps_in_option_and_state_alias() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(component(custom(shape = crate::shapes::ToggleShape, wraps_in_option = false)))]
                enabled: bool,

                #[gpui_form(component(custom(state = crate::state::TagsState, wraps_in_option = false)))]
                tags: Vec<String>,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
                generate_mcp: false,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("pubenabled:bool") && !compact.contains("pubenabled:Option<bool>"),
            "wraps_in_option = false should keep value holder field non-optional"
        );
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

    // token-level: no behavior surface
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
                generate_mcp: false,
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
            compact.contains(".map(::gpui_kit::component::IndexPath::new)")
                && !compact.contains(".position(|x|x==__gpui_form_default).unwrap()"),
            "Select component initialization should skip invalid defaults instead of panicking"
        );
    }

    // token-level: no behavior surface
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
                generate_mcp: false,
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

    // token-level: no behavior surface
    #[test]
    fn layout_all_five_hints_plus_width_bare_ident_emit_chain() {
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
                generate_mcp: false,
            },
        );

        let compact = compact_tokenish(&expanded.to_string());

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
        assert!(
            compact.contains(".with_layout("),
            "with_layout call missing: {compact}"
        );
    }

    // token-level: no behavior surface
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
                generate_mcp: false,
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

    // token-level: no behavior surface
    #[test]
    fn layout_absent_hints_default_to_full_and_no_string_builders() {
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
                generate_mcp: false,
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

    // token-level: no behavior surface
    #[test]
    fn layout_on_skipped_field_is_ignored() {
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
                generate_mcp: false,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

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
        let layout_count = compact.matches(".with_layout(").count();
        assert_eq!(
            layout_count, 1,
            "expected exactly one FieldVariant (visible only): {compact}"
        );
    }

    #[test]
    fn layout_width_unknown_variant_is_rejected() {
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

    #[test]
    fn test_mcp_attribute_requires_mcp_feature() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            #[gpui_form(mcp)]
            struct TestForm {
                value: String,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expand_gpui_form(
            derive_input,
            GpuiFormOptions {
                generate_shape: true,
                generate_mcp: false,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("requiresthe`gpui-form/mcp`feature"),
            "mcp attribute should require the mcp feature: {compact}"
        );
    }

    #[test]
    fn test_mcp_attribute_rejects_generic_forms() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            #[gpui_form(mcp)]
            struct TestForm<T> {
                value: T,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expand_gpui_form(
            derive_input,
            GpuiFormOptions {
                generate_shape: true,
                generate_mcp: true,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("doesnotsupportgenericforms"),
            "mcp attribute should reject generic forms: {compact}"
        );
    }
}
