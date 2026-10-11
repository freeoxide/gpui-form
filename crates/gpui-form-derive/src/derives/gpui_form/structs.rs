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
    pub original_type: Type,
    pub inner_type: Type,
    pub was_optional: bool,
    pub wrap_in_option: bool,
    pub validation: ValidationInfo,
    pub default_expr: Option<Expr>,
    pub override_type: Option<Type>,
    pub into_expr: Option<Expr>,
    pub from_expr: Option<Expr>,
    pub skip: bool,
    /// Milestone M3 (backlog #2/#3): which composed holder shape this field
    /// uses. `Flat` keeps the pre-M3 behavior byte-for-byte; `Nested` stores
    /// the nested form's own generated value holder; `List` stores the plain
    /// item `Vec` plus generated add/remove/items operations.
    pub composition: FieldComposition,
}

/// Milestone M3: the composed holder shape of a field.
#[derive(Clone, Debug)]
pub enum FieldComposition {
    /// Pre-M3 single-field shape (input/select/checkbox/...).
    Flat,
    /// `component(nested)` — `holder` is the resolved
    /// `<NestedName>FormValueHolder` type; conversions compose through the
    /// nested form's own `From` impls.
    Nested { holder: Type },
    /// `component(list(item = ...))` — `item` is the scalar item base type;
    /// the holder stores `Vec<item>` (or `Option<Vec<item>>` for
    /// originally-optional fields).
    List { item: Type },
}

impl FieldComposition {
    pub fn is_nested(&self) -> bool {
        matches!(self, Self::Nested { .. })
    }

    pub fn is_list(&self) -> bool {
        matches!(self, Self::List { .. })
    }

    pub fn is_flat(&self) -> bool {
        matches!(self, Self::Flat)
    }
}

impl FieldOptionality {
    /// Returns true if this field needs the `RequiredValidation` koruma validator.
    /// This applies to fields that:
    /// - Are wrapped in Option (for form handling)
    /// - Were not originally Optional in the source struct
    /// - Are not nested structs (nested fields have their own validation)
    pub fn needs_required_validation(&self) -> bool {
        !self.skip
            && self.wrap_in_option
            && !self.was_optional
            && !self.validation.is_nested
            && matches!(self.composition, FieldComposition::Flat)
    }
}

#[derive(Clone, Debug, Default, FromMeta)]
pub struct KorumaOptions {
    #[darling(default)]
    pub fluent: bool,
}

/// `validate(async_fn = <path>)`: the async validator path, parsed with the
/// same custom-`FromMeta` pattern as `TypeOverride` (bare path, or a quoted
/// string parsed as a path). Anything else (e.g. an integer literal) is a
/// darling error at parse time.
#[derive(Clone, Debug)]
pub struct AsyncFnPath(pub syn::Path);

impl FromMeta for AsyncFnPath {
    fn from_expr(expr: &Expr) -> darling::Result<Self> {
        match expr {
            Expr::Path(expr_path) => Ok(AsyncFnPath(expr_path.path.clone())),
            Expr::Group(group) => Self::from_expr(&group.expr),
            Expr::Lit(expr_lit) => Self::from_value(&expr_lit.lit),
            _ => Err(DarlingError::unexpected_expr_type(expr)),
        }
    }

    fn from_string(value: &str) -> darling::Result<Self> {
        syn::parse_str::<syn::Path>(value)
            .map(AsyncFnPath)
            .map_err(|_| DarlingError::unknown_value(value))
    }

    fn from_value(value: &Lit) -> darling::Result<Self> {
        if let Lit::Str(v) = value {
            v.parse::<syn::Path>()
                .map(AsyncFnPath)
                .map_err(|_| DarlingError::unknown_value(&v.value()).with_span(v))
        } else {
            Err(DarlingError::unexpected_lit_type(value))
        }
    }
}

/// `validate(async_fn = ..., debounce = ...)` — the async-validation
/// attribute surface (M2 emission sketch). The attr parses and records
/// metadata only in this milestone; execution wiring is deferred to the
/// GPUI-glue phase behind the reserved `async-validation` feature.
#[derive(Clone, Debug, FromMeta)]
pub struct AsyncValidate {
    pub async_fn: AsyncFnPath,
    // Parsed (grammar is accepted today) but not yet read by any emitter:
    // the debounce policy belongs to the future driving layer.
    #[allow(dead_code)]
    #[darling(default)]
    pub debounce: Option<Expr>,
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
    // Async-validation attribute surface (M2 emission sketch):
    // `#[gpui_form(validate(async_fn = my_check), component(input))]`.
    // Parse + metadata only; nothing executes until the reserved
    // `async-validation` wiring phase.
    #[darling(default)]
    pub validate: Option<AsyncValidate>,
    #[darling(default)]
    pub skip: bool,
}

impl ComponentField {
    pub fn skip(&self) -> bool {
        self.skip
    }

    /// Milestone M3 (backlog #2): `component(nested)` on this field.
    pub fn is_nested(&self) -> bool {
        matches!(self.component, Some(Components::Nested(_)))
    }

    /// Milestone M3 (backlog #3): `component(list(...))` on this field.
    pub fn is_list(&self) -> bool {
        matches!(self.component, Some(Components::List(_)))
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
    // Backlog #7 (submit lifecycle): opt-in `#[gpui_form(submit)]` — a bare
    // word. Parse-time only; it drives the `<Name>FormSubmission` alias
    // emitter (see `submit.rs`), never holder state.
    #[darling(default)]
    pub submit: bool,
}

pub struct ComponentFieldContent {
    pub field_structure_tokens: TokenStream,
    pub field_base_declarations_tokens: TokenStream,
    pub wrap_in_option: (String, bool),
}

#[derive(Default)]
pub struct GpuiFormOptions {
    pub generate_shape: bool,
}
