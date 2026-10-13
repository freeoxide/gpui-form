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
    fn schema_export_reuses_the_inventory_shape_chain() {
        let expand = |tokens: proc_macro2::TokenStream| {
            let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
            expansion::expand_gpui_form(
                derive_input,
                structs::GpuiFormOptions {
                    generate_shape: true,
                },
            )
            .to_string()
        };

        let flat = compact_tokens(&expand(quote! {
            #[derive(GpuiForm)]
            struct Plain {
                #[gpui_form(component(input))]
                name: String,
            }
        }));
        assert!(
            flat.contains("pubstructPlainFormSchema"),
            "every form gains the schema-export type: {flat}"
        );
        assert!(
            flat.contains(
                "constSHAPE:::gpui_form::schema::registry::GpuiFormShape=::gpui_form::schema::registry::GpuiFormShape::new"
            ),
            "SHAPE must be the registry shape chain: {flat}"
        );
        assert!(
            flat.contains(
                "pubfnschema()->::gpui_form::schema::registry::FormSchema{::gpui_form::schema::registry::FormSchema::from(&Self::SHAPE)}"
            ),
            "schema() must project the shape through From: {flat}"
        );
        assert_eq!(
            flat.matches("FieldVariant::new").count(),
            1,
            "the FieldVariant chain must be emitted exactly once per form: {flat}"
        );
        assert!(
            flat.contains("inventory::submit!{PlainFormSchema::SHAPE}"),
            "the inventory submission must reference the exported SHAPE const: {flat}"
        );

        let derive_input: DeriveInput = syn::parse2(quote! {
            #[derive(GpuiForm)]
            struct Plain {
                #[gpui_form(component(input))]
                name: String,
            }
        })
        .unwrap();
        let no_inventory = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: false,
            },
        )
        .to_string();
        let no_inventory = compact_tokens(&no_inventory);
        assert!(
            no_inventory.contains("pubstructPlainFormSchema")
                && no_inventory.contains("pubfnschema()"),
            "the export does not depend on the inventory feature: {no_inventory}"
        );
        assert!(
            !no_inventory.contains("inventory::submit"),
            "inventory registration stays feature-gated: {no_inventory}"
        );

        let derive_input: DeriveInput = syn::parse2(quote! {
            #[derive(GpuiForm)]
            #[gpui_form(empty)]
            struct Blank {}
        })
        .unwrap();
        let empty = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        )
        .to_string();
        let empty = compact_tokens(&empty);
        assert!(
            empty.contains("pubstructBlankFormSchema"),
            "empty forms emit the schema-export type too: {empty}"
        );
        assert!(
            empty.contains("GpuiFormShape::new(stringify!(Blank),&[],file!(),"),
            "the empty shape chain carries zero field variants: {empty}"
        );
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
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("pubenabled:bool"),
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
            compact.contains(".map(::gpui_kit::component::IndexPath::new)")
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
            "expected the visible FieldVariant only; the shape chain is emitted \
             ONCE (the schema-export SHAPE const, which the inventory submission \
             references), so the skipped field's layout hints can appear exactly \
             nowhere: {compact}"
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
    fn empty_form_still_emits_state_facade_and_patch() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            #[gpui_form(empty)]
            struct Blank {}
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
            compact.contains("pubstructBlankFormState"),
            "empty form must emit the facade: {compact}"
        );
        assert!(
            compact.contains("pubstructBlankFormPatch"),
            "empty form must emit the patch type: {compact}"
        );
        assert!(
            compact.contains("FIELD_COUNT:usize=0"),
            "empty form path table must count zero fields: {compact}"
        );
        assert!(
            !compact.contains("BlankFormFieldErrors"),
            "empty form without koruma must not emit the error surface: {compact}"
        );
    }

    #[test]
    fn field_errors_surface_follows_the_koruma_derive_gate() {
        let expand = |tokens: proc_macro2::TokenStream| {
            let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
            expansion::expand_gpui_form(
                derive_input,
                structs::GpuiFormOptions {
                    generate_shape: true,
                },
            )
            .to_string()
        };

        let plain = expand(quote! {
            #[derive(GpuiForm)]
            struct Plain {
                #[gpui_form(component(input))]
                name: String,
            }
        });
        let plain = compact_tokens(&plain);
        assert!(
            !plain.contains("PlainFormFieldErrors") && !plain.contains("validate_fields"),
            "non-koruma forms must see zero error-surface diff: {plain}"
        );
        assert!(
            plain.contains("pubstructPlainFormState") && plain.contains("pubstructPlainFormPatch"),
            "facade + patch are unconditional: {plain}"
        );

        let field_only = expand(quote! {
            #[derive(GpuiForm)]
            struct FieldAttr {
                #[gpui_form(component(number_input))]
                #[koruma(koruma_collection::numeric::RangeValidation::<_>::builder().min(1).max(9))]
                seats: u32,
            }
        });
        let field_only = compact_tokens(&field_only);
        assert!(
            field_only.contains("pubstructFieldAttrFormFieldErrors")
                && field_only.contains("validate_fields"),
            "field-attr-only koruma forms must gain the error surface: {field_only}"
        );
    }

    #[test]
    fn patch_serde_derives_follow_the_serde_feature() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct P {
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

        if cfg!(feature = "serde") {
            assert!(
                compact.contains("skip_serializing_if=\"Option::is_none\""),
                "serde builds emit the PATCH tri-state wire attrs: {compact}"
            );
        } else {
            assert!(
                !compact.contains("skip_serializing_if"),
                "default builds must not emit serde attrs on the patch: {compact}"
            );
        }
    }

    #[test]
    fn submit_alias_emits_only_with_the_attribute() {
        let expand = |tokens: proc_macro2::TokenStream| {
            let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
            expansion::expand_gpui_form(
                derive_input,
                structs::GpuiFormOptions {
                    generate_shape: true,
                },
            )
            .to_string()
        };

        let plain = expand(quote! {
            #[derive(GpuiForm)]
            struct Plain {
                #[gpui_form(component(input))]
                name: String,
            }
        });
        let plain = compact_tokens(&plain);
        assert!(
            !plain.contains("FormSubmission"),
            "non-opted forms must see zero submit diff: {plain}"
        );

        let koruma = expand(quote! {
            #[derive(GpuiForm)]
            #[gpui_form(koruma, submit)]
            struct K {
                #[gpui_form(component(input))]
                name: String,
            }
        });
        let koruma = compact_tokens(&koruma);
        assert!(
            koruma.contains(
                "pubtypeKFormSubmission=::gpui_form::core::submit::SubmitTracker<<KFormValueHolderas::koruma::ValidateExt>::Error>;"
            ),
            "koruma forms alias over the holder's ValidateExt::Error: {koruma}"
        );

        let field_only = expand(quote! {
            #[derive(GpuiForm)]
            #[gpui_form(submit)]
            struct F {
                #[gpui_form(component(number_input))]
                #[koruma(koruma_collection::numeric::RangeValidation::<_>::builder().min(1).max(9))]
                seats: u32,
            }
        });
        let field_only = compact_tokens(&field_only);
        assert!(
            field_only.contains("as::koruma::ValidateExt>::Error>"),
            "field-attr-only koruma forms keep the typed alias: {field_only}"
        );

        let no_koruma = expand(quote! {
            #[derive(GpuiForm)]
            #[gpui_form(submit)]
            struct N {
                #[gpui_form(component(input))]
                name: String,
            }
        });
        let no_koruma = compact_tokens(&no_koruma);
        assert!(
            no_koruma.contains("SubmitTracker<::gpui_form::core::submit::BoxError>"),
            "non-koruma forms alias over BoxError: {no_koruma}"
        );
        assert!(!no_koruma.contains("ValidateExt"));
    }

    #[test]
    fn validation_cache_facade_follows_the_koruma_derive_gate() {
        let expand = |tokens: proc_macro2::TokenStream| {
            let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
            expansion::expand_gpui_form(
                derive_input,
                structs::GpuiFormOptions {
                    generate_shape: true,
                },
            )
            .to_string()
        };

        let plain = expand(quote! {
            #[derive(GpuiForm)]
            struct Plain {
                #[gpui_form(component(input))]
                name: String,
            }
        });
        let plain = compact_tokens(&plain);
        assert!(
            !plain.contains("ValidationCache")
                && !plain.contains("pubfnvalidate")
                && !plain.contains("pubfnis_valid")
                && !plain.contains("invalidate_validation"),
            "non-koruma facades see zero validation-cache diff: {plain}"
        );

        let field_only = expand(quote! {
            #[derive(GpuiForm)]
            struct FieldAttr {
                #[gpui_form(component(number_input))]
                #[koruma(koruma_collection::numeric::RangeValidation::<_>::builder().min(1).max(9))]
                seats: u32,
            }
        });
        let field_only = compact_tokens(&field_only);
        assert!(
            field_only.contains("::gpui_form::core::ValidationCache<FieldAttrFormValueHolder,FieldAttrFormFieldErrors>"),
            "field-attr-only koruma facades gain the validation cache: {field_only}"
        );
        assert!(
            field_only.contains("pubfnvalidate(&mutself)->&FieldAttrFormFieldErrors")
                && field_only.contains("pubfnis_valid(&self)->bool")
                && field_only.contains("pubfninvalidate_validation(&mutself)"),
            "the cached-validation method set must emit: {field_only}"
        );

        let generic = expand(quote! {
            #[derive(GpuiForm)]
            struct G<T> {
                #[gpui_form(component(number_input))]
                #[koruma(koruma_collection::numeric::RangeValidation::<_>::builder().min(1).max(9))]
                seats: Option<T>,
            }
        });
        let generic = compact_tokens(&generic);
        assert!(
            !generic.contains("ValidationCache"),
            "generic koruma forms keep compiling without the cache: {generic}"
        );
    }

    #[test]
    fn async_validator_metadata_records_the_path() {
        let expand = |tokens: proc_macro2::TokenStream| {
            let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
            expansion::expand_gpui_form(
                derive_input,
                structs::GpuiFormOptions {
                    generate_shape: true,
                },
            )
            .to_string()
        };

        let with_attr = expand(quote! {
            #[derive(GpuiForm)]
            struct Form {
                #[gpui_form(validate(async_fn = my_check), component(input))]
                username: String,
            }
        });
        let with_attr = compact_tokens(&with_attr);
        assert!(
            with_attr.contains(".with_async_validator(Some(\"my_check\"))"),
            "the FieldVariant chain must record the parsed path: {with_attr}"
        );

        let without_attr = expand(quote! {
            #[derive(GpuiForm)]
            struct Form {
                #[gpui_form(component(input))]
                username: String,
            }
        });
        let without_attr = compact_tokens(&without_attr);
        assert!(
            !without_attr.contains("with_async_validator"),
            "forms without the attr must see zero metadata diff: {without_attr}"
        );
    }

    #[test]
    fn schema_metadata_expr_strings_keep_canonical_spacing() {
        let expand = |tokens: proc_macro2::TokenStream| {
            let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
            expansion::expand_gpui_form(
                derive_input,
                structs::GpuiFormOptions {
                    generate_shape: true,
                },
            )
            .to_string()
        };

        let expanded = expand(quote! {
            #[derive(GpuiForm)]
            struct Form {
                #[gpui_form(component(select), default = crate::defaults::country())]
                country: String,
            }
        });

        assert!(
            expanded.contains(r#"with_default ("crate::defaults::country()")"#),
            "the default expression must export in canonical spelling: {expanded}"
        );
        assert!(
            !expanded.contains(r#"with_default ("crate :: defaults :: country ()")"#),
            "the proc-macro printer's token spacing must not leak into the exported literal: {expanded}"
        );
    }

    #[test]
    fn dependency_metadata_and_required_if_desugaring_are_emitted() {
        let expand = |tokens: proc_macro2::TokenStream| {
            let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
            expansion::expand_gpui_form(
                derive_input,
                structs::GpuiFormOptions {
                    generate_shape: true,
                },
            )
            .to_string()
        };

        let expanded = expand(quote! {
            #[derive(GpuiForm)]
            struct Form {
                #[gpui_form(component(switch))]
                subscribe: bool,

                #[gpui_form(component(input), depends_on = subscribe, required_if = subscribe)]
                email: Option<String>,
            }
        });
        let compact = compact_tokens(&expanded);

        assert!(
            compact.contains(".with_depends_on(Some(\"subscribe\"))"),
            "depends_on must be recorded: {compact}"
        );
        assert!(
            compact.contains(".with_required_if(Some(\"subscribe\"))"),
            "required_if must be recorded: {compact}"
        );
        assert!(
            compact.contains("\"RequiredIfValidation\""),
            "the conditional rule kind rides the validations list: {compact}"
        );

        assert!(
            compact.contains("pubstructFormFormRequiredIfValidation<T>"),
            "the per-form validator struct is emitted: {compact}"
        );
        assert!(
            compact.contains(
                "FormFormRequiredIfValidation::<Option<_>>::builder()\
                 .when(::core::clone::Clone::clone(&self.subscribe))"
            ),
            "the holder attr desugars to the validator builder closing over \
             the sibling's holder value: {compact}"
        );
        assert!(
            compact.contains("::gpui_form::core::required_when(self.when,value.is_some())"),
            "the validator delegates to core's required_when: {compact}"
        );

        let plain = expand(quote! {
            #[derive(GpuiForm)]
            struct Form {
                #[gpui_form(component(input))]
                email: Option<String>,
            }
        });
        let plain = compact_tokens(&plain);
        assert!(
            !plain.contains("with_depends_on")
                && !plain.contains("with_required_if")
                && !plain.contains("FormRequiredIfValidation")
                && !plain.contains("RequiredIfValidation"),
            "forms without dependency attrs must see zero emission diff: {plain}"
        );
    }

    #[test]
    fn required_if_validator_maps_to_the_shared_required_key() {
        use crate::derives::gpui_form::value_holder::validation_issue_key_for_kind;

        assert_eq!(
            validation_issue_key_for_kind("SignupFormFormRequiredIfValidation"),
            "validation.required"
        );
        assert_eq!(
            validation_issue_key_for_kind("RangeValidation"),
            "validation.range"
        );
    }

    #[test]
    fn dependency_attrs_validate_their_sibling_references() {
        let expand = |tokens: proc_macro2::TokenStream| {
            let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
            expansion::expand_gpui_form(
                derive_input,
                structs::GpuiFormOptions {
                    generate_shape: true,
                },
            )
            .to_string()
        };

        let unknown_required_if = expand(quote! {
            #[derive(GpuiForm)]
            struct Form {
                #[gpui_form(component(input), required_if = nonexistent)]
                email: Option<String>,
            }
        });
        assert!(
            unknown_required_if.contains("must name a non-skipped field of this struct"),
            "unknown required_if sibling must fail at derive time: {unknown_required_if}"
        );

        let unknown_depends_on = expand(quote! {
            #[derive(GpuiForm)]
            struct Form {
                #[gpui_form(component(input), depends_on = missing)]
                email: Option<String>,
            }
        });
        assert!(
            unknown_depends_on.contains("must name a non-skipped field of this struct"),
            "unknown depends_on sibling must fail at derive time: {unknown_depends_on}"
        );

        let self_depends_on = expand(quote! {
            #[derive(GpuiForm)]
            struct Form {
                #[gpui_form(component(input), depends_on = email)]
                email: Option<String>,
            }
        });
        let self_error = "`depends_on = email` on field `email` must name a different field";
        assert!(
            self_depends_on.contains(self_error),
            "a self-referencing depends_on must fail at derive time: {self_depends_on}"
        );

        let non_optional_target = expand(quote! {
            #[derive(GpuiForm)]
            struct Form {
                #[gpui_form(component(switch))]
                subscribe: bool,

                #[gpui_form(component(select), required_if = subscribe)]
                email: String,
            }
        });
        assert!(
            non_optional_target.contains("holder stores `Option<T>`"),
            "a target whose holder is not Option-wrapped must fail: {non_optional_target}"
        );

        let bare_field = expand(quote! {
            #[derive(GpuiForm)]
            struct Form {
                #[gpui_form(component(switch))]
                subscribe: bool,

                #[gpui_form(required_if = subscribe)]
                email: String,
            }
        });
        assert!(
            bare_field.contains("require a `component(...)` attribute"),
            "dependency attrs on component-less fields must fail: {bare_field}"
        );
    }

    #[test]
    fn flat_form_expansion_snapshot_is_pinned() {
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
        let snapshot = if cfg!(feature = "serde") {
            include_str!("../../../tests/snapshots/flat_form_expansion_serde.txt")
        } else {
            include_str!("../../../tests/snapshots/flat_form_expansion.txt")
        };
        let snapshot: String = snapshot.chars().filter(|c| !c.is_whitespace()).collect();
        assert_eq!(
            compact.lines().next().unwrap_or_default(),
            snapshot.lines().next().unwrap_or_default(),
            "flat-form expansion changed; update the matching              tests/snapshots/flat_form_expansion*.txt DELIBERATELY if the change is intended"
        );
    }

    #[test]
    fn nested_and_list_fields_emit_composition_metadata_and_holder_ops() {
        let expand = |tokens: proc_macro2::TokenStream| {
            let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
            expansion::expand_gpui_form(
                derive_input,
                structs::GpuiFormOptions {
                    generate_shape: true,
                },
            )
            .to_string()
        };

        let nested = expand(quote! {
            #[derive(GpuiForm)]
            struct Resident {
                #[gpui_form(component(input))]
                name: String,

                #[gpui_form(component(nested))]
                address: Address,
            }
        });
        let nested = compact_tokens(&nested);
        assert!(
            nested.contains("pubaddress:AddressFormValueHolder"),
            "nested holder field is the nested form's holder (Plain storage): {nested}"
        );
        assert!(
            nested.contains("NestedBehaviour{form_struct:\"Address\","),
            "FieldVariant carries the nested form struct name: {nested}"
        );
        assert!(
            nested.contains("pubfnaddress_root()"),
            "nested fields gain a root path constructor: {nested}"
        );
        assert!(
            nested.contains("pubfnaddress(self)"),
            "nested fields gain a consuming path appender: {nested}"
        );
        assert!(
            !nested.contains("pubfnaddress()"),
            "nested fields have no zero-arg constructor to collide with the appender: {nested}"
        );

        let list = expand(quote! {
            #[derive(GpuiForm)]
            struct Tags {
                #[gpui_form(component(list(item = input)))]
                tags: Vec<String>,
            }
        });
        let list = compact_tokens(&list);
        assert!(
            list.contains("pubtags:Vec<String>"),
            "scalar list items store the plain Vec (v1: no per-item Option): {list}"
        );
        assert!(
            list.contains("ListBehaviour{item:"),
            "FieldVariant carries the scalar item kind: {list}"
        );
        assert!(
            list.contains("pubfnadd_tags(&mutself,value:String)"),
            "the holder gains add_<field>: {list}"
        );
        assert!(
            list.contains("pubfnremove_tags(&mutself,index:usize)"),
            "the holder gains remove_<field>: {list}"
        );
        assert!(
            list.contains("pubfntags_items(&self)->&[String]"),
            "the holder gains <field>_items: {list}"
        );
        assert!(
            list.contains("pubfntags(index:usize)"),
            "list fields gain a typed-index path constructor: {list}"
        );
        assert!(
            list.contains("PathSegment::Index(index)"),
            "the item path carries a typed index segment, never a string: {list}"
        );
        assert!(
            list.contains("pubfntags_root()"),
            "list fields keep a root path constructor: {list}"
        );
        assert!(
            !list.contains("reorder") && !list.contains("swap_"),
            "v1 must not emit reorder operations: {list}"
        );
    }

    #[test]
    fn list_fields_mirror_element_validators_as_each_and_errors_carry_index() {
        let tokens = quote! {
            #[derive(GpuiForm, koruma::Koruma)]
            struct Tags {
                #[gpui_form(component(list(item = input)))]
                #[koruma(each(koruma_collection::collection::LenValidation::<_>::builder().min(2).max(20)))]
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
            compact.contains("#[koruma(each(koruma_collection::collection::LenValidation::<_>::builder().min(2).max(20)))]"),
            "element validators mirror onto list holder fields inside each(..): {compact}"
        );
        assert!(
            compact.contains("pubfnTags_element_error(&self,index:usize)")
                || compact.contains("pubfntags_element_error(&self,index:usize)"),
            "the error map gains the index-addressable element-error accessor: {compact}"
        );
        assert!(
            compact.contains("pubindex:"),
            "the conversion error type carries the additive index slot: {compact}"
        );
    }

    #[test]
    fn nested_koruma_marker_mirrors_without_other_validators() {
        let tokens = quote! {
            #[derive(GpuiForm, koruma::Koruma)]
            struct Resident {
                #[gpui_form(component(nested))]
                #[koruma(nested)]
                address: Address,
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
            compact.contains("#[koruma(nested)]"),
            "a bare #[koruma(nested)] must mirror onto the holder so the top-level error getter exists: {compact}"
        );
        assert!(
            compact.contains("<AddressFormValueHolderas::koruma::ValidateExt>::Error"),
            "the nested slot's inner type is the nested form holder's ValidateExt error: {compact}"
        );
    }

    #[test]
    fn bare_koruma_nested_on_flat_fields_keeps_pre_m3_emission() {
        let expand = |tokens: proc_macro2::TokenStream| {
            let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
            expansion::expand_gpui_form(
                derive_input,
                structs::GpuiFormOptions {
                    generate_shape: true,
                },
            )
            .to_string()
        };

        let flat = expand(quote! {
            #[derive(GpuiForm, koruma::Koruma)]
            struct Legacy {
                #[koruma(nested)]
                address: Address,
            }
        });
        let flat = compact_tokens(&flat);
        assert!(
            !flat.contains("#[koruma(nested)]"),
            "flat fields with a bare #[koruma(nested)] must NOT gain a mirrored marker (pre-M3 emission): {flat}"
        );
        assert!(
            !flat.contains("as::koruma::ValidateExt>::Error"),
            "flat fields with a bare #[koruma(nested)] must not join the error map's nested slot: {flat}"
        );

        let composed = expand(quote! {
            #[derive(GpuiForm, koruma::Koruma)]
            struct Resident {
                #[gpui_form(component(nested))]
                #[koruma(nested)]
                address: Address,
            }
        });
        let composed = compact_tokens(&composed);
        assert!(
            composed.contains("#[koruma(nested)]"),
            "component(nested) fields keep the bare-marker mirroring: {composed}"
        );
    }

    #[test]
    fn list_component_requires_vec_field_type() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct Bad {
                #[gpui_form(component(list(item = input)))]
                tags: String,
            }
        };
        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );
        let message = expanded.to_string();
        assert!(
            message.contains("requires the field type to be Vec<T>"),
            "non-Vec list fields error at expansion time: {message}"
        );
    }
}
