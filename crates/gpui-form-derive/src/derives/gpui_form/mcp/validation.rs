use super::*;
use crate::derives::gpui_form::structs::FieldOptionality;
use heck::ToSnakeCase as _;
use koruma_derive_core::ValidatorAttr;

struct ValidationRulePlan {
    scope_tokens: TokenStream,
    validator: ValidatorAttr,
}

pub(super) fn field_validation_rules_tokens(
    facade_crate: &Path,
    form_ident: &syn::Ident,
    field_index: usize,
    field: &FieldOptionality,
    enable_koruma: bool,
) -> (Vec<TokenStream>, Option<TokenStream>) {
    let plans = validation_rule_plans(facade_crate, field, enable_koruma);
    if plans.is_empty() {
        return (Vec::new(), None);
    }

    let rules_const_ident = field_validation_rules_const_ident(form_ident, field_index);
    let mut consts = Vec::new();
    let mut rules = Vec::new();

    for (rule_index, plan) in plans.iter().enumerate() {
        let rule = validation_rule_tokens(facade_crate, plan, form_ident, field_index, rule_index);
        rules.push(rule.rule);
        consts.extend(rule.param_const);
    }

    consts.push(quote! {
        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        const #rules_const_ident: &[#facade_crate::mcp::McpValidationRule] = &[
            #(#rules),*
        ];
    });

    (
        consts,
        Some(quote! {
            .with_validation_rules(#rules_const_ident)
        }),
    )
}

fn validation_rule_plans(
    facade_crate: &Path,
    field: &FieldOptionality,
    enable_koruma: bool,
) -> Vec<ValidationRulePlan> {
    if !enable_koruma {
        return Vec::new();
    }

    let mut plans = Vec::new();
    if field.needs_required_validation() {
        let required: ValidatorAttr = syn::parse_quote!(
            ::koruma_collection::general::RequiredValidation::<Option<_>>::builder()
        );
        plans.push(ValidationRulePlan {
            scope_tokens: quote! { #facade_crate::mcp::McpValidationScope::Field },
            validator: required,
        });
    }
    for validator in &field.validation.field_validators {
        plans.push(ValidationRulePlan {
            scope_tokens: quote! { #facade_crate::mcp::McpValidationScope::Field },
            validator: validator.clone(),
        });
    }
    for validator in &field.validation.element_validators {
        plans.push(ValidationRulePlan {
            scope_tokens: quote! { #facade_crate::mcp::McpValidationScope::Element },
            validator: validator.clone(),
        });
    }
    plans
}

fn validation_rule_tokens(
    facade_crate: &Path,
    plan: &ValidationRulePlan,
    form_ident: &syn::Ident,
    field_index: usize,
    rule_index: usize,
) -> ValidationRuleTokenPlan {
    let validator = &plan.validator;
    let validator_name = LitStr::new(&validator.name().to_string(), validator.name().span());
    let validator_path = LitStr::new(&validator.path_name(), plan.validator.validator.span());
    let target_tokens = quote! { #facade_crate::mcp::McpValidationTarget::Default };
    let type_arg_tokens = if validator.infer_type {
        quote! { #facade_crate::mcp::McpValidationTypeArgMode::Infer }
    } else if validator.explicit_type.is_some() {
        quote! { #facade_crate::mcp::McpValidationTypeArgMode::Explicit }
    } else {
        quote! { #facade_crate::mcp::McpValidationTypeArgMode::None }
    };
    let params = validator
        .setter_calls()
        .iter()
        .flat_map(|call| {
            let method_name = call.method.to_string();
            let arg_count = call.args.len();
            call.args
                .iter()
                .enumerate()
                .map(move |(index, arg)| {
                    let param_name = if arg_count == 1 {
                        method_name.clone()
                    } else {
                        format!("{method_name}[{index}]")
                    };
                    let param_name = LitStr::new(&param_name, call.method.span());
                    if let Some(literal) = literal_expr_string(arg) {
                        let literal = LitStr::new(&literal, arg.span());
                        quote! {
                            #facade_crate::mcp::McpValidationParam::literal(
                                #param_name,
                                #literal
                            )
                        }
                    } else {
                        let expr = LitStr::new(&arg.to_token_stream().to_string(), arg.span());
                        quote! {
                            #facade_crate::mcp::McpValidationParam::expr(
                                #param_name,
                                #expr
                            )
                        }
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let params_ident = field_validation_params_const_ident(form_ident, field_index, rule_index);
    let (param_const, params_tokens) = if params.is_empty() {
        (
            None,
            quote! { #facade_crate::mcp::MCP_VALIDATION_PARAMS_NONE },
        )
    } else {
        (
            Some(quote! {
                #[doc(hidden)]
                #[allow(non_upper_case_globals)]
                const #params_ident: &[#facade_crate::mcp::McpValidationParam] = &[
                    #(#params),*
                ];
            }),
            quote! { #params_ident },
        )
    };
    let scope_tokens = &plan.scope_tokens;

    let rule = quote! {
        #facade_crate::mcp::McpValidationRule::new(
            #scope_tokens,
            #validator_name,
            #validator_path,
            ::std::option::Option::None,
            #type_arg_tokens,
            #params_tokens,
        )
        .with_target(#target_tokens)
    };

    ValidationRuleTokenPlan { param_const, rule }
}

struct ValidationRuleTokenPlan {
    param_const: Option<TokenStream>,
    rule: TokenStream,
}

fn field_validation_rules_const_ident(form_ident: &syn::Ident, field_index: usize) -> syn::Ident {
    format_ident!("__{}GpuiFormMcpValidationRules{field_index}", form_ident)
}

fn field_validation_params_const_ident(
    form_ident: &syn::Ident,
    field_index: usize,
    rule_index: usize,
) -> syn::Ident {
    format_ident!(
        "__{}GpuiFormMcpValidationParams{field_index}_{rule_index}",
        form_ident
    )
}

fn literal_expr_string(expr: &syn::Expr) -> Option<String> {
    match expr {
        syn::Expr::Lit(lit) => literal_string(&lit.lit),
        syn::Expr::Unary(unary)
            if matches!(unary.op, syn::UnOp::Neg(_))
                && matches!(
                    unary.expr.as_ref(),
                    syn::Expr::Lit(expr_lit)
                        if matches!(expr_lit.lit, Lit::Int(_) | Lit::Float(_))
                ) =>
        {
            let syn::Expr::Lit(lit) = unary.expr.as_ref() else {
                return None;
            };
            literal_string(&lit.lit).map(|literal| format!("-{literal}"))
        },
        _ => None,
    }
}

fn literal_string(lit: &Lit) -> Option<String> {
    match lit {
        Lit::Int(lit) => Some(lit.base10_digits().to_string()),
        Lit::Float(lit) => Some(lit.base10_digits().to_string()),
        Lit::Bool(lit) => Some(lit.value.to_string()),
        Lit::Str(lit) => Some(lit.value()),
        _ => None,
    }
}

pub(super) fn field_validation_issue_tokens(
    facade_crate: &Path,
    form_ident: &syn::Ident,
    field_index: usize,
    field: &FieldOptionality,
    enable_koruma: bool,
) -> Option<TokenStream> {
    if field.validation.is_nested {
        return None;
    }
    if field.validation.is_newtype {
        let field_ident = &field.field_name;
        let field_name = field.field_name.to_string();
        return Some(quote! {
            let __gpui_form_field_error = __gpui_form_validation_error.#field_ident();
            if ::koruma::ValidationError::has_errors(__gpui_form_field_error) {
                __gpui_form_issues.push(
                    #facade_crate::mcp::McpValidationIssue::custom(
                        #facade_crate::mcp::McpValidationScope::Field,
                        ::std::format!("{__gpui_form_field_error:?}")
                    )
                    .with_field(#field_name)
                );
            }
        });
    }
    let plans = validation_rule_plans(facade_crate, field, enable_koruma);
    if plans.is_empty() {
        return None;
    }

    let field_ident = &field.field_name;
    let field_name = field.field_name.to_string();
    let rules_const_ident = field_validation_rules_const_ident(form_ident, field_index);
    let mut checks = Vec::new();
    for (rule_index, plan) in plans.iter().enumerate() {
        let getter = validator_getter_ident(&plan.validator, &plans);
        checks.push(quote! {
            if let Some(__gpui_form_validator) = __gpui_form_field_error.#getter() {
                __gpui_form_issues.push(
                    #facade_crate::mcp::McpValidationIssue::for_rule(
                        #field_name,
                        #rules_const_ident[#rule_index],
                        ::std::format!("{__gpui_form_validator:?}")
                    )
                );
            }
        });
    }

    Some(quote! {
        let __gpui_form_field_error = __gpui_form_validation_error.#field_ident();
        #(#checks)*
    })
}

fn validator_getter_ident(
    validator: &ValidatorAttr,
    siblings: &[ValidationRulePlan],
) -> syn::Ident {
    let sibling_names: Vec<ValidatorAttr> =
        siblings.iter().map(|plan| plan.validator.clone()).collect();
    let simple = validator.name().to_string().to_snake_case();
    if !has_name_collision(&simple, &sibling_names, |sibling| {
        sibling.name().to_string().to_snake_case()
    }) {
        return format_ident!("{}", simple);
    }

    let fallback = validator.codegen_snake_name();
    let resolved = if has_name_collision(&fallback, &sibling_names, |sibling| {
        sibling.codegen_snake_name()
    }) {
        format!("{}_{}", fallback, stable_hash_hex(&validator.path_name()))
    } else {
        fallback
    };
    format_ident!("{}", resolved)
}

fn has_name_collision(
    target_name: &str,
    siblings: &[ValidatorAttr],
    name_fn: impl Fn(&ValidatorAttr) -> String,
) -> bool {
    siblings
        .iter()
        .filter(|sibling| name_fn(sibling) == target_name)
        .nth(1)
        .is_some()
}

fn stable_hash_hex(input: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{:08x}", (hash & 0xffff_ffff) as u32)
}
