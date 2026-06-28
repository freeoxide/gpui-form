use darling::{Error as DarlingError, FromField, FromMeta};
use gpui_form_codegen::components::Components;
use koruma_derive_core::ValidationInfo;
use proc_macro2::TokenStream;
use quote::ToTokens as _;
use syn::{Expr, Ident, Lit, Type, TypePath};

#[derive(Clone, Debug)]
pub struct TypeOverride(pub Type);

impl FromMeta for TypeOverride {
    fn from_expr(expr: &Expr) -> darling::Result<Self> {
        match expr {
            Expr::Path(expr_path) => Ok(TypeOverride(Type::Path(TypePath {
                qself: expr_path.qself.clone(),
                path: expr_path.path.clone(),
            }))),
            Expr::Group(group) => Self::from_expr(&group.expr),
            Expr::Lit(expr_lit) => Self::from_value(&expr_lit.lit),
            _ => Err(DarlingError::unexpected_expr_type(expr)),
        }
    }

    fn from_string(value: &str) -> darling::Result<Self> {
        syn::parse_str::<Type>(value)
            .map(TypeOverride)
            .map_err(|_| DarlingError::unknown_value(value))
    }

    fn from_value(value: &Lit) -> darling::Result<Self> {
        if let Lit::Str(v) = value {
            v.parse::<Type>()
                .map(TypeOverride)
                .map_err(|_| DarlingError::unknown_value(&v.value()).with_span(v))
        } else {
            Err(DarlingError::unexpected_lit_type(value))
        }
    }
}

#[derive(Clone, Debug)]
pub struct DefaultExpr(pub Expr);

impl FromMeta for DefaultExpr {
    fn from_expr(expr: &Expr) -> darling::Result<Self> {
        Ok(DefaultExpr(expr.clone()))
    }

    fn from_string(value: &str) -> darling::Result<Self> {
        syn::parse_str::<Expr>(value)
            .map(DefaultExpr)
            .map_err(|_| DarlingError::unknown_value(value))
    }

    fn from_value(value: &Lit) -> darling::Result<Self> {
        Ok(DefaultExpr(Expr::Lit(syn::ExprLit {
            attrs: Vec::new(),
            lit: value.clone(),
        })))
    }
}

/// Feature #4 (METADATA-FIRST v1): derive-side mirror of
/// `gpui_form::schema::layout::LayoutWidth`, used only to parse the
/// `width = full | half | third` attribute. Accepts both the bare ident form
/// (`width = half`) and the quoted form (`width = "half"`). Modeled on the
/// existing `TypeOverride`/`DefaultExpr` custom `FromMeta` impls above: a bare
/// ident arrives as `Expr::Path`, a quoted string arrives as `Lit::Str`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LayoutWidthMeta {
    #[default]
    Full,
    Half,
    Third,
}

impl FromMeta for LayoutWidthMeta {
    fn from_expr(expr: &Expr) -> darling::Result<Self> {
        match expr {
            Expr::Path(expr_path) => match expr_path.path.get_ident() {
                Some(id) if id == "full" => Ok(Self::Full),
                Some(id) if id == "half" => Ok(Self::Half),
                Some(id) if id == "third" => Ok(Self::Third),
                _ => Err(DarlingError::unknown_value(
                    &expr_path.path.to_token_stream().to_string(),
                )),
            },
            // darling sometimes wraps the value in invisible grouping; recurse
            // (mirrors `TypeOverride::from_expr`).
            Expr::Group(group) => Self::from_expr(&group.expr),
            Expr::Lit(expr_lit) => Self::from_value(&expr_lit.lit),
            _ => Err(DarlingError::unexpected_expr_type(expr)),
        }
    }

    fn from_string(value: &str) -> darling::Result<Self> {
        match value {
            "full" => Ok(Self::Full),
            "half" => Ok(Self::Half),
            "third" => Ok(Self::Third),
            _ => Err(DarlingError::unknown_value(value)),
        }
    }

    fn from_value(value: &Lit) -> darling::Result<Self> {
        if let Lit::Str(v) = value {
            Self::from_string(&v.value())
        } else {
            Err(DarlingError::unexpected_lit_type(value))
        }
    }
}

/// Information about a field for value holder generation.
pub struct FieldOptionality {
    pub field_name: Ident,
    #[allow(dead_code)]
    pub original_type: Type,
    #[allow(dead_code)]
    pub inner_type: Type,
    pub was_optional: bool,
    pub wrap_in_option: bool,
    pub validation: ValidationInfo,
    pub default_expr: Option<Expr>,
    pub override_type: Option<Type>,
    pub into_expr: Option<Expr>,
    pub from_expr: Option<Expr>,
    pub skip: bool,
}

impl FieldOptionality {
    /// Returns true if this field needs the `RequiredValidation` koruma validator.
    /// This applies to fields that:
    /// - Are wrapped in Option (for form handling)
    /// - Were not originally Optional in the source struct
    /// - Are not nested structs (nested fields have their own validation)
    pub fn needs_required_validation(&self) -> bool {
        !self.skip && self.wrap_in_option && !self.was_optional && !self.validation.is_nested
    }
}

#[derive(Clone, Debug, Default, FromMeta)]
pub struct KorumaOptions {
    #[darling(default)]
    pub fluent: bool,
}

#[derive(Clone, Debug)]
pub struct KorumaField(pub KorumaOptions);

impl FromMeta for KorumaField {
    fn from_word() -> darling::Result<Self> {
        Ok(KorumaField(KorumaOptions::default()))
    }

    fn from_list(items: &[darling::ast::NestedMeta]) -> darling::Result<Self> {
        KorumaOptions::from_list(items).map(KorumaField)
    }
}

#[derive(Debug, FromField)]
#[darling(attributes(gpui_form))]
pub struct ComponentField {
    pub ident: Option<Ident>,
    pub ty: Type,
    #[darling(default, rename = "type")]
    pub r#type: Option<TypeOverride>,
    #[darling(default)]
    pub into: Option<Expr>,
    #[darling(default)]
    pub from: Option<Expr>,
    #[darling(default)]
    pub component: Option<Components>,
    #[darling(default)]
    pub default: Option<DefaultExpr>,
    // Feature #4 (METADATA-FIRST v1): non-rendering layout hints. All
    // optional; absence means the consumer falls back to its own default
    // (e.g. label defaults to the field name at consumption time).
    #[darling(default)]
    pub section: Option<String>,
    #[darling(default)]
    pub label: Option<String>,
    #[darling(default)]
    pub description: Option<String>,
    #[darling(default)]
    pub placeholder: Option<String>,
    #[darling(default)]
    pub width: Option<LayoutWidthMeta>,
    #[darling(default)]
    pub skip: bool,
}

impl ComponentField {
    pub fn skip(&self) -> bool {
        self.skip
    }
}

#[derive(Debug, darling::FromDeriveInput)]
#[darling(attributes(gpui_form), supports(struct_named, struct_unit))]
pub struct ComponentStruct {
    pub ident: Ident,
    pub data: darling::ast::Data<(), ComponentField>,
    #[darling(default)]
    pub empty: bool,
    #[darling(default)]
    pub koruma: Option<KorumaField>,
}

pub struct ComponentFieldContent {
    pub field_structure_tokens: TokenStream,
    pub field_base_declarations_tokens: TokenStream,
    pub wrap_in_option: (String, bool),
}

pub struct GpuiFormOptions {
    pub generate_shape: bool,
}
