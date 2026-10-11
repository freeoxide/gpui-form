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
    /// Milestone M4 (backlog #12): `depends_on = <sibling>` — metadata-only.
    pub depends_on: Option<Ident>,
    /// Milestone M4 (backlog #13): `required_if = <sibling>`, resolved at
    /// expansion time into the emitted validator's identity plus the
    /// sibling's truthiness shape (see `RequiredIfRule`).
    pub required_if: Option<RequiredIfRule>,
}

/// Milestone M4 (backlog #13): the fully-resolved `required_if` rule.
///
/// `expansion.rs` validates the sibling reference and computes this before
/// holder generation; `value_holder.rs` emits the validator struct
/// (`validator`) and mirrors the koruma attr (`.when(#condition)`), and
/// `errors.rs` mirrors the koruma error-field getter (`validator`'s
/// snake_case name).
#[derive(Clone, Debug)]
pub struct RequiredIfRule {
    /// The controlling sibling field, as written in the attribute.
    pub sibling: Ident,
    /// The emitted validator type: `<Name>FormRequiredIfValidation`. Named
    /// per form (not per field) so one emitted type serves every
    /// `required_if` field of the form, and following the reserved
    /// `<Name>Form*` prefix convention.
    pub validator: Ident,
    /// How the sibling's holder value decides "required" at validation
    /// time.
    pub condition: RequiredIfCondition,
}

/// The truthiness shape of a `required_if` controlling sibling's HOLDER
/// value. Computed from the sibling's holder field type at expansion time,
/// so the emitted validator needs no runtime trait resolution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RequiredIfCondition {
    /// Plain `bool` sibling (switch/checkbox on a non-optional field):
    /// required when `true`.
    Bool,
    /// `Option<bool>` sibling: required when `Some(true)` — an explicit
    /// `Some(false)` (switch off) must NOT require the value.
    SomeTrue,
    /// Any other `Option<T>` sibling: required when set (`Some(_)`).
    IsSome,
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

impl RequiredIfCondition {
    /// The `when(...)` builder-setter argument the mirrored koruma attr
    /// carries. The expression is evaluated inside koruma's generated
    /// `validate(&self)` on the HOLDER, so `self.<sibling>` reads the
    /// controlling sibling's live holder value — the typed, compile-checked
    /// reference R19 asks for (no stringly rule engine).
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
    /// Returns true if this field needs the `RequiredValidation` koruma validator.
    /// This applies to fields that:
    /// - Are wrapped in Option (for form handling)
    /// - Were not originally Optional in the source struct
    /// - Are not nested structs (nested fields have their own validation)
    ///
    /// Milestone M4: a field with `required_if` never gets the synthetic
    /// unconditional `RequiredValidation` — the conditional rule supersedes
    /// it (requiring the field always would defeat the condition).
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

/// Milestone M4 (backlog #12/#13): a reference to a sibling field of the
/// same struct — the value of `depends_on = <field>` and
/// `required_if = <field>`.
///
/// Parsed as a bare ident (the typed form, `depends_on = country`) or a
/// quoted ident (`required_if = "subscribe"`, the backlog's spelling) —
/// never as an arbitrary string or expression. The referenced sibling is
/// existence-checked at expansion time, and the generated validator closes
/// over `self.<field>` directly, so a typo is a compile error, not a
/// silently-dead rule (feature-parity plan R19: no stringly typed
/// expressions, no expression language).
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
    // Milestone M4 (backlog #12): `depends_on = <sibling field>` —
    // metadata-only dependency recording for conditional loading and
    // reset-on-parent-change consumers.
    #[darling(default)]
    pub depends_on: Option<SiblingField>,
    // Milestone M4 (backlog #13): `required_if = <sibling field>` —
    // conditional requiredness. Desugars to a generated koruma validator
    // (see `value_holder.rs`), so violations surface through the typed
    // field-error surface; the sibling reference is also recorded as
    // schema metadata.
    #[darling(default)]
    pub required_if: Option<SiblingField>,
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
