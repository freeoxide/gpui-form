use darling::{Error as DarlingError, FromField, FromMeta};
use gpui_form_codegen::components::Components;
use koruma_derive_core::ValidationInfo;
use proc_macro2::TokenStream;
use quote::{ToTokens as _, quote};
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
    pub composition: FieldComposition,
    pub depends_on: Option<Ident>,
    pub required_if: Option<RequiredIfRule>,
}

#[derive(Clone, Debug)]
pub struct RequiredIfRule {
    pub sibling: Ident,
    /// Emitted validator type: `<Name>FormRequiredIfValidation`, one per
    /// form serving all of its `required_if` fields.
    pub validator: Ident,
    pub condition: RequiredIfCondition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RequiredIfCondition {
    /// Plain `bool` sibling: required when `true`.
    Bool,
    /// `Option<bool>` sibling: required only on `Some(true)` — an explicit
    /// `Some(false)` must not require the value.
    SomeTrue,
    /// Any other `Option<T>` sibling: required when set (`Some(_)`).
    IsSome,
}

#[derive(Clone, Debug)]
pub enum FieldComposition {
    Flat,
    /// `component(nested)`; `holder` is the nested form's resolved
    /// `<Name>FormValueHolder` type.
    Nested {
        holder: Type,
    },
    /// `component(list(item = ...))`; the holder stores `Vec<item>`
    /// (or `Option<Vec<item>>` for originally-optional fields).
    List {
        item: Type,
    },
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

impl RequiredIfCondition {
    /// `when(...)` argument for the mirrored koruma attr; evaluated in
    /// koruma's `validate(&self)` on the holder, so `self.<sibling>` is the
    /// sibling's live holder value.
    pub fn when_argument_tokens(self, sibling: &Ident) -> TokenStream {
        match self {
            Self::Bool => quote! { ::core::clone::Clone::clone(&self.#sibling) },
            Self::SomeTrue => {
                quote! { self.#sibling.as_ref().is_some_and(|value| *value) }
            },
            Self::IsSome => quote! { self.#sibling.is_some() },
        }
    }
}

impl FieldOptionality {
    pub fn needs_required_validation(&self) -> bool {
        !self.skip
            && self.wrap_in_option
            && !self.was_optional
            && !self.validation.is_nested
            && matches!(self.composition, FieldComposition::Flat)
            && self.required_if.is_none()
    }
}

#[derive(Clone, Debug, Default, FromMeta)]
pub struct KorumaOptions {
    #[darling(default)]
    pub fluent: bool,
}

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

#[derive(Clone, Debug, FromMeta)]
pub struct AsyncValidate {
    pub async_fn: AsyncFnPath,
    #[allow(dead_code)]
    #[darling(default)]
    pub debounce: Option<Expr>,
}

/// A sibling field reference (`depends_on` / `required_if`): a bare or
/// quoted ident, existence-checked at expansion time and closed over as
/// `self.<field>`, so a typo is a compile error.
#[derive(Clone, Debug)]
pub struct SiblingField(pub Ident);

impl FromMeta for SiblingField {
    fn from_expr(expr: &Expr) -> darling::Result<Self> {
        match expr {
            Expr::Path(expr_path) => match expr_path.path.get_ident() {
                Some(ident) => Ok(SiblingField(ident.clone())),
                None => Err(DarlingError::unknown_value(
                    &expr_path.path.to_token_stream().to_string(),
                )),
            },
            Expr::Group(group) => Self::from_expr(&group.expr),
            Expr::Lit(expr_lit) => Self::from_value(&expr_lit.lit),
            _ => Err(DarlingError::unexpected_expr_type(expr)),
        }
    }

    fn from_string(value: &str) -> darling::Result<Self> {
        syn::parse_str::<Ident>(value)
            .map(SiblingField)
            .map_err(|_| DarlingError::unknown_value(value))
    }

    fn from_value(value: &Lit) -> darling::Result<Self> {
        if let Lit::Str(v) = value {
            Self::from_string(&v.value())
        } else {
            Err(DarlingError::unexpected_lit_type(value))
        }
    }
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
    pub validate: Option<AsyncValidate>,
    #[darling(default)]
    pub depends_on: Option<SiblingField>,
    #[darling(default)]
    pub required_if: Option<SiblingField>,
    #[darling(default)]
    pub skip: bool,
}

impl ComponentField {
    pub fn skip(&self) -> bool {
        self.skip
    }

    pub fn is_nested(&self) -> bool {
        matches!(self.component, Some(Components::Nested(_)))
    }

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
