use darling::{Error as DarlingError, FromMeta};
use gpui_form_schema::components::{ComponentKind, NumberInputKind};
use proc_macro2::TokenStream;
use quote::{ToTokens as _, quote};

use crate::implementations::ComponentLayout as _;

pub trait ComponentOption {}

pub trait ComponentDefinition {
    fn component_kind() -> ComponentKind;

    fn component_name() -> &'static str {
        Self::component_kind().component_name()
    }
}

pub struct FieldInformation<T: ComponentOption> {
    pub options: T,
    pub name: String,
    pub r#type: syn::Type,
}

impl<T: ComponentOption> FieldInformation<T> {
    pub fn new(options: T, name: String, r#type: syn::Type) -> Self {
        Self {
            options,
            name,
            r#type,
        }
    }
}

pub struct GeneratedFieldLayout {
    pub field_structure_tokens: TokenStream,
    pub field_base_declarations_tokens: TokenStream,
    pub wrap_in_option: bool,
}

macro_rules! impl_component_option {
    ($($option:ty),+ $(,)?) => {
        $(impl ComponentOption for $option {})+
    };
}

macro_rules! define_component_definition {
    ($component:ident, $options:ty, $kind:ident) => {
        pub struct $component(pub FieldInformation<$options>);

        impl ComponentDefinition for $component {
            fn component_kind() -> ComponentKind {
                ComponentKind::$kind
            }
        }
    };
}

#[derive(Clone, Debug, Default, Eq, FromMeta, PartialEq)]
pub struct BehaviourSelectOptions {
    #[darling(default)]
    pub partial: bool,
    #[darling(default)]
    pub searchable: bool,
}

#[derive(Clone, Debug, Default)]
pub struct SelectOptions {
    pub behaviour: BehaviourSelectOptions,
    /// Field-level default value expression (e.g., `EnumCountry::France` or
    /// `preferred_country()`).
    /// This is set by the derive macro when the field has a `default = ...` attribute.
    field_default: Option<syn::Expr>,
}

impl FromMeta for SelectOptions {
    fn from_word() -> darling::Result<Self> {
        Ok(Self::default())
    }

    fn from_list(items: &[darling::ast::NestedMeta]) -> darling::Result<Self> {
        let behaviour = BehaviourSelectOptions::from_list(items)?;
        Ok(Self {
            behaviour,
            field_default: None,
        })
    }
}

impl SelectOptions {
    pub fn with_field_default(mut self, default: Option<syn::Expr>) -> Self {
        self.field_default = default;
        self
    }

    pub fn field_default(&self) -> Option<&syn::Expr> {
        self.field_default.as_ref()
    }

    pub fn use_enum_default(&self) -> bool {
        self.field_default.is_none()
    }
}

#[derive(Clone, Debug, Default, FromMeta)]
pub struct InputOptions;

#[derive(Clone, Debug, Default)]
pub struct NumberInputOptions {
    /// For custom numeric types (like Decimal), specify a standard numeric type
    /// for validation purposes (e.g., f64, i32, u64)
    pub r#as: Option<syn::Ident>,
}

impl FromMeta for NumberInputOptions {
    fn from_word() -> darling::Result<Self> {
        Ok(Self::default())
    }

    fn from_list(items: &[darling::ast::NestedMeta]) -> darling::Result<Self> {
        let mut r#as = None;

        for item in items {
            if let darling::ast::NestedMeta::Meta(syn::Meta::NameValue(nv)) = item
                && nv.path.is_ident("as")
                && let syn::Expr::Path(expr_path) = &nv.value
                && let Some(ident) = expr_path.path.get_ident()
            {
                r#as = Some(ident.clone());
            }
        }

        Ok(Self { r#as })
    }
}

#[derive(Clone, Debug, Default)]
pub struct PhoneInputOptions {
    /// Optional sibling field whose selected country the number must match
    /// (`phone_input(country = <field>)`). When absent, any globally valid
    /// number is accepted.
    pub country: Option<syn::Ident>,
}

impl FromMeta for PhoneInputOptions {
    fn from_word() -> darling::Result<Self> {
        Ok(Self::default())
    }

    fn from_list(items: &[darling::ast::NestedMeta]) -> darling::Result<Self> {
        let mut country = None;

        for item in items {
            if let darling::ast::NestedMeta::Meta(syn::Meta::NameValue(nv)) = item
                && nv.path.is_ident("country")
                && let syn::Expr::Path(expr_path) = &nv.value
                && let Some(ident) = expr_path.path.get_ident()
            {
                country = Some(ident.clone());
            }
        }

        Ok(Self { country })
    }
}

#[derive(Clone, Debug, FromMeta)]
pub struct CheckboxOptions;

#[derive(Clone, Debug, FromMeta)]
pub struct SwitchOptions;

#[derive(Clone, Debug, FromMeta)]
pub struct DatePickerOptions;

#[derive(Clone, Debug, FromMeta)]
pub struct FilePickerOptions;

fn default_custom_wraps_in_option() -> bool {
    true
}

#[derive(Clone, Debug)]
pub struct CustomOptions {
    /// Path to a type implementing `gpui_form::custom::CustomComponentShape`.
    pub shape: syn::Path,
    /// UI component type path (e.g. `TagsInput`).
    /// When provided, the prototyping code generator emits `Component::new(&entity)`.
    pub component: Option<syn::Path>,
    /// Whether the value holder should store this field as `Option<T>`.
    /// Defaults to `true`.
    pub wraps_in_option: bool,
    /// Whether prototyping code should wire this custom component through
    /// `CustomComponentValueAdapter`.
    pub value_binding: bool,
}

#[derive(Debug, Default, FromMeta)]
struct CustomOptionsMeta {
    #[darling(default)]
    shape: Option<syn::Path>,
    #[darling(default)]
    state: Option<syn::Path>,
    #[darling(default)]
    component: Option<syn::Path>,
    #[darling(default = "default_custom_wraps_in_option")]
    wraps_in_option: bool,
    #[darling(default)]
    value_binding: bool,
}

impl CustomOptions {
    fn from_meta(meta: CustomOptionsMeta) -> darling::Result<Self> {
        let CustomOptionsMeta {
            shape,
            state,
            component,
            wraps_in_option,
            value_binding,
        } = meta;

        let shape = match (shape, state) {
            (Some(shape), None) | (None, Some(shape)) => shape,
            (Some(_), Some(_)) => {
                return Err(DarlingError::custom(
                    "custom component may specify only one of `shape` or `state`",
                ));
            },
            (None, None) => {
                return Err(DarlingError::custom(
                    "custom component requires `shape = ...` or `state = ...`",
                ));
            },
        };

        Ok(Self {
            shape,
            component,
            wraps_in_option,
            value_binding,
        })
    }
}

impl FromMeta for CustomOptions {
    fn from_word() -> darling::Result<Self> {
        Err(DarlingError::custom(
            "custom component requires `shape = ...` or `state = ...`",
        ))
    }

    fn from_list(items: &[darling::ast::NestedMeta]) -> darling::Result<Self> {
        let meta = CustomOptionsMeta::from_list(items)?;
        Self::from_meta(meta)
    }
}

#[derive(Clone, Debug, Default, Eq, FromMeta, PartialEq)]
pub struct BehaviourInfiniteSelectOptions {
    #[darling(default)]
    pub searchable: bool,
    #[darling(default)]
    pub max_depth: Option<usize>,
}

/// Milestone M3 (backlog #2): `component(nested)` — a word-only marker, like
/// `CheckboxOptions`. The field's own type names the nested form struct.
#[derive(Clone, Debug, Eq, FromMeta, PartialEq)]
pub struct NestedOptions;

/// Milestone M3 (backlog #3): the scalar item component rendered per item of
/// a `component(list(item = ...))` field. Custom FromMeta mirrors
/// `LayoutWidthMeta`: a bare ident arrives as `Expr::Path`, a quoted kind as
/// `Lit::Str`. `nested` and `custom(...)` are rejected in v1 (backlog #3:
/// "scalar item components before nested item forms"; custom components stay
/// app-owned escape hatches, not list items).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ListItemSpec {
    Input,
    NumberInput,
    PhoneInput,
    Checkbox,
    Switch,
    Select,
    InfiniteSelect,
    DatePicker,
    FilePicker,
}

impl ListItemSpec {
    pub const fn kind(self) -> ComponentKind {
        match self {
            Self::Input => ComponentKind::Input,
            Self::NumberInput => ComponentKind::NumberInput,
            Self::PhoneInput => ComponentKind::PhoneInput,
            Self::Checkbox => ComponentKind::Checkbox,
            Self::Switch => ComponentKind::Switch,
            Self::Select => ComponentKind::Select,
            Self::InfiniteSelect => ComponentKind::InfiniteSelect,
            Self::DatePicker => ComponentKind::DatePicker,
            Self::FilePicker => ComponentKind::FilePicker,
        }
    }

    fn from_ident_str(value: &str) -> darling::Result<Self> {
        match value {
            "input" => Ok(Self::Input),
            "number_input" => Ok(Self::NumberInput),
            "phone_input" => Ok(Self::PhoneInput),
            "checkbox" => Ok(Self::Checkbox),
            "switch" => Ok(Self::Switch),
            "select" => Ok(Self::Select),
            "infinite_select" => Ok(Self::InfiniteSelect),
            "date_picker" => Ok(Self::DatePicker),
            "file_picker" => Ok(Self::FilePicker),
            "nested" => Err(DarlingError::custom(
                "list(item = nested) is not supported yet: v1 ships scalar item components \
                 only (backlog #3 sequences nested item forms after scalar items)",
            )),
            "custom" => Err(DarlingError::custom(
                "list(item = custom(...)) is not supported: custom components are app-owned \
                 escape hatches, not list items",
            )),
            other => Err(DarlingError::unknown_value(other)),
        }
    }
}

impl FromMeta for ListItemSpec {
    fn from_expr(expr: &syn::Expr) -> darling::Result<Self> {
        match expr {
            syn::Expr::Path(expr_path) => match expr_path.path.get_ident() {
                Some(ident) => Self::from_ident_str(&ident.to_string()),
                None => Err(DarlingError::unknown_value(
                    &expr_path.path.to_token_stream().to_string(),
                )),
            },
            // darling sometimes wraps the value in invisible grouping; recurse
            // (mirrors `LayoutWidthMeta::from_expr` in the derive crate).
            syn::Expr::Group(group) => Self::from_expr(&group.expr),
            syn::Expr::Lit(expr_lit) => Self::from_value(&expr_lit.lit),
            _ => Err(DarlingError::unexpected_expr_type(expr)),
        }
    }

    fn from_string(value: &str) -> darling::Result<Self> {
        Self::from_ident_str(value)
    }

    fn from_value(value: &syn::Lit) -> darling::Result<Self> {
        if let syn::Lit::Str(v) = value {
            Self::from_string(&v.value())
        } else {
            Err(DarlingError::unexpected_lit_type(value))
        }
    }
}

/// Milestone M3 (backlog #3): `component(list(item = <kind>))`. `item` is
/// required — a bare `component(list)` is a parse error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListOptions {
    pub item: ListItemSpec,
    /// Optional label override for the generated add button; falls back to
    /// the `{form}.{field}_add` i18n key.
    pub add_label: Option<String>,
}

impl FromMeta for ListOptions {
    fn from_word() -> darling::Result<Self> {
        Err(DarlingError::custom(
            "list requires an item component: component(list(item = input))",
        ))
    }

    fn from_list(items: &[darling::ast::NestedMeta]) -> darling::Result<Self> {
        let mut item = None;
        let mut add_label = None;

        for entry in items {
            if let darling::ast::NestedMeta::Meta(syn::Meta::NameValue(nv)) = entry
                && nv.path.is_ident("item")
            {
                item = Some(ListItemSpec::from_expr(&nv.value)?);
            } else if let darling::ast::NestedMeta::Meta(syn::Meta::NameValue(nv)) = entry
                && nv.path.is_ident("add_label")
            {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(value),
                    ..
                }) = &nv.value
                {
                    add_label = Some(value.value());
                } else {
                    return Err(DarlingError::custom(
                        "list(add_label = ...) expects a string literal",
                    ));
                }
            }
        }

        Ok(Self {
            item: item.ok_or_else(|| {
                DarlingError::custom(
                    "list requires an item component: component(list(item = input))",
                )
            })?,
            add_label,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct InfiniteSelectOptions {
    pub behaviour: BehaviourInfiniteSelectOptions,
    /// Field-level default value expression (e.g., `Country::France(...)` or
    /// `default_country()`).
    /// This is set by the derive macro when the field has a `default = ...` attribute.
    field_default: Option<syn::Expr>,
}

impl FromMeta for InfiniteSelectOptions {
    fn from_word() -> darling::Result<Self> {
        Ok(Self::default())
    }

    fn from_list(items: &[darling::ast::NestedMeta]) -> darling::Result<Self> {
        let behaviour = BehaviourInfiniteSelectOptions::from_list(items)?;
        Ok(Self {
            behaviour,
            field_default: None,
        })
    }
}

impl InfiniteSelectOptions {
    pub fn with_field_default(mut self, default: Option<syn::Expr>) -> Self {
        self.field_default = default;
        self
    }

    pub fn field_default(&self) -> Option<&syn::Expr> {
        self.field_default.as_ref()
    }

    pub fn use_enum_default(&self) -> bool {
        self.field_default.is_none()
    }
}

impl_component_option!(
    BehaviourSelectOptions,
    SelectOptions,
    InputOptions,
    NumberInputOptions,
    PhoneInputOptions,
    CheckboxOptions,
    SwitchOptions,
    DatePickerOptions,
    FilePickerOptions,
    CustomOptions,
    BehaviourInfiniteSelectOptions,
    InfiniteSelectOptions,
    NestedOptions,
    ListOptions,
);

#[derive(Clone, Debug, FromMeta)]
#[darling(rename_all = "snake_case")]
pub enum Components {
    Input,
    NumberInput(NumberInputOptions),
    PhoneInput(PhoneInputOptions),
    Checkbox,
    Switch,
    Select(SelectOptions),
    InfiniteSelect(InfiniteSelectOptions),
    Custom(CustomOptions),
    DatePicker,
    FilePicker,
    /// Milestone M3 (backlog #2): nested-form composition.
    Nested(NestedOptions),
    /// Milestone M3 (backlog #3): repeated `Vec<T>` fields.
    List(ListOptions),
}

define_component_definition!(InputComponent, InputOptions, Input);
define_component_definition!(NumberInputComponent, NumberInputOptions, NumberInput);
define_component_definition!(PhoneInputComponent, PhoneInputOptions, PhoneInput);
define_component_definition!(CheckboxComponent, CheckboxOptions, Checkbox);
define_component_definition!(SwitchComponent, SwitchOptions, Switch);
define_component_definition!(SelectComponent, SelectOptions, Select);
define_component_definition!(
    InfiniteSelectComponent,
    InfiniteSelectOptions,
    InfiniteSelect
);
define_component_definition!(CustomComponent, CustomOptions, Custom);
define_component_definition!(DatePickerComponent, DatePickerOptions, DatePicker);
define_component_definition!(FilePickerComponent, FilePickerOptions, FilePicker);
define_component_definition!(NestedComponent, NestedOptions, Nested);
define_component_definition!(ListComponent, ListOptions, List);

fn number_input_kind(type_str: &str) -> NumberInputKind {
    if type_str.starts_with('f') {
        NumberInputKind::Float
    } else if type_str.starts_with('u') {
        NumberInputKind::UnsignedInteger
    } else if type_str.starts_with('i') {
        NumberInputKind::SignedInteger
    } else {
        NumberInputKind::Custom
    }
}

/// The last path segment of a type, as a string literal: `Address` and
/// `some_lib::Address` both yield `"Address"`. Used for the
/// `NestedBehaviour::form_struct` metadata — the shared naming-convention
/// hook the derive and prototyping use to resolve the nested form's
/// generated types (`<Name>FormValueHolder` etc.).
fn type_last_segment_str(field_type: &syn::Type) -> String {
    match field_type {
        syn::Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
            .unwrap_or_default(),
        _ => field_type.to_token_stream().to_string(),
    }
}

fn number_input_behaviour_tokens(
    options: &NumberInputOptions,
    field_type: &syn::Type,
) -> TokenStream {
    let type_str = options
        .r#as
        .as_ref()
        .map(|ty| ty.to_string())
        .unwrap_or_else(|| field_type.to_token_stream().to_string());

    let kind_tokens = match number_input_kind(&type_str) {
        NumberInputKind::Float => {
            quote! { ::gpui_form::schema::components::NumberInputKind::Float }
        },
        NumberInputKind::SignedInteger => {
            quote! { ::gpui_form::schema::components::NumberInputKind::SignedInteger }
        },
        NumberInputKind::UnsignedInteger => {
            quote! { ::gpui_form::schema::components::NumberInputKind::UnsignedInteger }
        },
        NumberInputKind::Custom => {
            quote! { ::gpui_form::schema::components::NumberInputKind::Custom }
        },
    };
    let validation_type = options.r#as.as_ref().map(|value| value.to_string());
    let validation_type = match validation_type {
        Some(value) => quote! { Some(#value) },
        None => quote! { None },
    };

    quote! {
        ::gpui_form::schema::components::NumberInputBehaviour {
            validation_type: #validation_type,
            kind: #kind_tokens,
        }
    }
}

impl Components {
    pub const fn kind(&self) -> ComponentKind {
        match self {
            Self::Input => ComponentKind::Input,
            Self::NumberInput(_) => ComponentKind::NumberInput,
            Self::PhoneInput(_) => ComponentKind::PhoneInput,
            Self::Checkbox => ComponentKind::Checkbox,
            Self::Switch => ComponentKind::Switch,
            Self::Select(_) => ComponentKind::Select,
            Self::InfiniteSelect(_) => ComponentKind::InfiniteSelect,
            Self::Custom(_) => ComponentKind::Custom,
            Self::DatePicker => ComponentKind::DatePicker,
            Self::FilePicker => ComponentKind::FilePicker,
            Self::Nested(_) => ComponentKind::Nested,
            Self::List(_) => ComponentKind::List,
        }
    }

    pub fn wraps_in_option(&self) -> bool {
        match self {
            Self::Custom(options) => options.wraps_in_option,
            // M3 (hazard 5): nested/list fields are never Option-wrapped by
            // the component — the nested holder / plain item Vec stores as-is
            // so `koruma(nested)` and `each(...)` apply directly.
            Self::Nested(_) | Self::List(_) => false,
            _ => self.kind().default_wraps_in_option(),
        }
    }

    pub fn generate_field_layout(
        &self,
        field_name: String,
        field_type: syn::Type,
        field_default: Option<syn::Expr>,
    ) -> GeneratedFieldLayout {
        let mut field_structure_tokens = TokenStream::new();
        let mut field_base_declarations_tokens = TokenStream::new();

        match self {
            Self::Input => {
                let component =
                    InputComponent(FieldInformation::new(InputOptions, field_name, field_type));
                component.field_tokens(
                    &mut field_structure_tokens,
                    &mut field_base_declarations_tokens,
                );
            },
            Self::NumberInput(options) => {
                let component = NumberInputComponent(FieldInformation::new(
                    options.clone(),
                    field_name,
                    field_type,
                ));
                component.field_tokens(
                    &mut field_structure_tokens,
                    &mut field_base_declarations_tokens,
                );
            },
            Self::PhoneInput(options) => {
                let component = PhoneInputComponent(FieldInformation::new(
                    options.clone(),
                    field_name,
                    field_type,
                ));
                component.field_tokens(
                    &mut field_structure_tokens,
                    &mut field_base_declarations_tokens,
                );
            },
            Self::Checkbox => {
                let component = CheckboxComponent(FieldInformation::new(
                    CheckboxOptions,
                    field_name,
                    field_type,
                ));
                component.field_tokens(
                    &mut field_structure_tokens,
                    &mut field_base_declarations_tokens,
                );
            },
            Self::Switch => {
                let component =
                    SwitchComponent(FieldInformation::new(SwitchOptions, field_name, field_type));
                component.field_tokens(
                    &mut field_structure_tokens,
                    &mut field_base_declarations_tokens,
                );
            },
            Self::Select(options) => {
                let component = SelectComponent(FieldInformation::new(
                    options.clone().with_field_default(field_default),
                    field_name,
                    field_type,
                ));
                component.field_tokens(
                    &mut field_structure_tokens,
                    &mut field_base_declarations_tokens,
                );
            },
            Self::InfiniteSelect(options) => {
                let component = InfiniteSelectComponent(FieldInformation::new(
                    options.clone().with_field_default(field_default),
                    field_name,
                    field_type,
                ));
                component.field_tokens(
                    &mut field_structure_tokens,
                    &mut field_base_declarations_tokens,
                );
            },
            Self::Custom(options) => {
                let component = CustomComponent(FieldInformation::new(
                    options.clone(),
                    field_name,
                    field_type,
                ));
                component.field_tokens(
                    &mut field_structure_tokens,
                    &mut field_base_declarations_tokens,
                );
            },
            Self::DatePicker => {
                let component = DatePickerComponent(FieldInformation::new(
                    DatePickerOptions,
                    field_name,
                    field_type,
                ));
                component.field_tokens(
                    &mut field_structure_tokens,
                    &mut field_base_declarations_tokens,
                );
            },
            Self::FilePicker => {
                let component = FilePickerComponent(FieldInformation::new(
                    FilePickerOptions,
                    field_name,
                    field_type,
                ));
                component.field_tokens(
                    &mut field_structure_tokens,
                    &mut field_base_declarations_tokens,
                );
            },
            // M3: nested/list are value-only in v1 — no `FormFields` entity
            // field and no `FormComponents` constructor (like checkbox and
            // switch). The holder stores the composed value.
            Self::Nested(options) => {
                let component = NestedComponent(FieldInformation::new(
                    options.clone(),
                    field_name,
                    field_type,
                ));
                component.field_tokens(
                    &mut field_structure_tokens,
                    &mut field_base_declarations_tokens,
                );
            },
            Self::List(options) => {
                let component = ListComponent(FieldInformation::new(
                    options.clone(),
                    field_name,
                    field_type,
                ));
                component.field_tokens(
                    &mut field_structure_tokens,
                    &mut field_base_declarations_tokens,
                );
            },
        }

        GeneratedFieldLayout {
            field_structure_tokens,
            field_base_declarations_tokens,
            wrap_in_option: self.wraps_in_option(),
        }
    }

    pub fn behaviour_tokens(&self, field_type: &syn::Type) -> TokenStream {
        match self {
            Self::Input => {
                quote! { ::gpui_form::schema::components::ComponentsBehaviour::Input }
            },
            Self::NumberInput(options) => {
                let behaviour = number_input_behaviour_tokens(options, field_type);

                quote! {
                    ::gpui_form::schema::components::ComponentsBehaviour::NumberInput(
                        #behaviour
                    )
                }
            },
            Self::PhoneInput(options) => {
                let country_field = match options.country.as_ref() {
                    Some(ident) => {
                        let name = ident.to_string();
                        quote! { Some(#name) }
                    },
                    None => quote! { None },
                };
                quote! {
                    ::gpui_form::schema::components::ComponentsBehaviour::PhoneInput(
                        ::gpui_form::schema::components::PhoneInputBehaviour {
                            country_field: #country_field,
                        }
                    )
                }
            },
            Self::Checkbox => {
                quote! { ::gpui_form::schema::components::ComponentsBehaviour::Checkbox }
            },
            Self::Switch => {
                quote! { ::gpui_form::schema::components::ComponentsBehaviour::Switch }
            },
            Self::Select(options) => {
                let searchable = options.behaviour.searchable;
                let partial = options.behaviour.partial;
                quote! {
                    ::gpui_form::schema::components::ComponentsBehaviour::Select(
                        ::gpui_form::schema::components::SelectBehaviour {
                            searchable: #searchable,
                            partial: #partial,
                        }
                    )
                }
            },
            Self::InfiniteSelect(options) => {
                let searchable = options.behaviour.searchable;
                let max_depth = match options.behaviour.max_depth {
                    Some(depth) => quote! { Some(#depth) },
                    None => quote! { None },
                };
                quote! {
                    ::gpui_form::schema::components::ComponentsBehaviour::InfiniteSelect(
                        ::gpui_form::schema::components::InfiniteSelectBehaviour {
                            searchable: #searchable,
                            max_depth: #max_depth,
                        }
                    )
                }
            },
            Self::Custom(_) => {
                quote! { ::gpui_form::schema::components::ComponentsBehaviour::Custom }
            },
            Self::DatePicker => {
                quote! { ::gpui_form::schema::components::ComponentsBehaviour::DatePicker }
            },
            Self::FilePicker => {
                quote! { ::gpui_form::schema::components::ComponentsBehaviour::FilePicker }
            },
            Self::Nested(_) => {
                // The nested form struct name: the last path segment of the
                // field's base type (`Address`, `some_lib::Address` ->
                // "Address"), matching the naming convention the derive and
                // prototyping use to resolve the nested form's generated
                // types.
                let form_struct = type_last_segment_str(field_type);
                quote! {
                    ::gpui_form::schema::components::ComponentsBehaviour::Nested(
                        ::gpui_form::schema::components::NestedBehaviour {
                            form_struct: #form_struct,
                        }
                    )
                }
            },
            Self::List(options) => {
                let item_kind = match options.item.kind() {
                    ComponentKind::Input => {
                        quote! { ::gpui_form::schema::components::ComponentKind::Input }
                    },
                    ComponentKind::NumberInput => {
                        quote! { ::gpui_form::schema::components::ComponentKind::NumberInput }
                    },
                    ComponentKind::PhoneInput => {
                        quote! { ::gpui_form::schema::components::ComponentKind::PhoneInput }
                    },
                    ComponentKind::Checkbox => {
                        quote! { ::gpui_form::schema::components::ComponentKind::Checkbox }
                    },
                    ComponentKind::Switch => {
                        quote! { ::gpui_form::schema::components::ComponentKind::Switch }
                    },
                    ComponentKind::Select => {
                        quote! { ::gpui_form::schema::components::ComponentKind::Select }
                    },
                    ComponentKind::InfiniteSelect => {
                        quote! { ::gpui_form::schema::components::ComponentKind::InfiniteSelect }
                    },
                    ComponentKind::DatePicker => {
                        quote! { ::gpui_form::schema::components::ComponentKind::DatePicker }
                    },
                    ComponentKind::FilePicker => {
                        quote! { ::gpui_form::schema::components::ComponentKind::FilePicker }
                    },
                    // `ListItemSpec` cannot name these kinds (parse-time
                    // rejection), so the metadata arm is unreachable for
                    // them; keep the match exhaustive for future kinds.
                    ComponentKind::Custom | ComponentKind::Nested | ComponentKind::List => {
                        quote! { ::gpui_form::schema::components::ComponentKind::Input }
                    },
                };
                // v1 ships scalar items only; `item_struct` stays `None`
                // until nested item forms ship (backlog #3 sequencing).
                let item_struct = quote! { None };
                let add_label = match options.add_label.as_deref() {
                    Some(label) => quote! { ::core::option::Option::Some(#label) },
                    None => quote! { ::core::option::Option::None },
                };
                quote! {
                    ::gpui_form::schema::components::ComponentsBehaviour::List(
                        ::gpui_form::schema::components::ListBehaviour {
                            item: #item_kind,
                            item_struct: #item_struct,
                            add_label: #add_label,
                        }
                    )
                }
            },
        }
    }

    pub fn custom_component_tokens(&self) -> Option<TokenStream> {
        let Self::Custom(options) = self else {
            return None;
        };

        let shape = &options.shape;
        if let Some(component) = options.component.as_ref() {
            let component_str = component.to_token_stream().to_string();
            Some(quote! { .with_custom_component(#component_str) })
        } else {
            Some(quote! {
                .with_custom_component_opt(
                    <#shape as ::gpui_form::custom::CustomComponentShape>::COMPONENT_PATH
                )
            })
        }
    }

    pub fn custom_shape_tokens(&self) -> Option<TokenStream> {
        let Self::Custom(options) = self else {
            return None;
        };

        let shape = options.shape.to_token_stream().to_string();
        Some(quote! { .with_custom_shape(#shape) })
    }

    pub fn custom_value_binding_tokens(&self) -> Option<TokenStream> {
        let Self::Custom(options) = self else {
            return None;
        };

        options
            .value_binding
            .then(|| quote! { .with_custom_value_binding(true) })
    }
}

#[cfg(test)]
mod tests {
    use super::{Components, ListItemSpec, ListOptions};
    use darling::FromMeta as _;
    use quote::quote;

    fn list_options_from_tokens(tokens: proc_macro2::TokenStream) -> darling::Result<ListOptions> {
        let items = darling::ast::NestedMeta::parse_meta_list(tokens)?;
        ListOptions::from_list(&items)
    }

    /// Build `Components` the way the derive does: from the meta-list items
    /// of `component(...)` (a bare `nested` word is a Path item, not a
    /// string literal).
    fn components_from_tokens(tokens: proc_macro2::TokenStream) -> darling::Result<Components> {
        let items = darling::ast::NestedMeta::parse_meta_list(tokens)?;
        Components::from_list(&items)
    }

    #[test]
    fn list_options_from_list_parses_bare_ident_and_quoted_items() {
        // Bare ident (`item = input`) — the documented grammar.
        let options = list_options_from_tokens(quote! { item = input }).expect("bare ident item");
        assert_eq!(options.item, ListItemSpec::Input);
        assert_eq!(options.add_label, None);

        // Quoted kind (`item = "number_input"`).
        let options =
            list_options_from_tokens(quote! { item = "number_input" }).expect("quoted item");
        assert_eq!(options.item, ListItemSpec::NumberInput);

        // add_label rides along.
        let options = list_options_from_tokens(quote! { item = select, add_label = "Add tag" })
            .expect("item plus add_label");
        assert_eq!(options.item, ListItemSpec::Select);
        assert_eq!(options.add_label.as_deref(), Some("Add tag"));
    }

    #[test]
    fn list_requires_an_item_component() {
        // Bare `component(list)` arrives as from_word.
        let error = ListOptions::from_word().expect_err("bare list must be rejected");
        assert!(
            error
                .to_string()
                .contains("list requires an item component"),
            "unexpected error: {error}"
        );

        // `component(list(other = 1))` — no item key.
        let error = list_options_from_tokens(quote! { add_label = "x" })
            .expect_err("missing item must be rejected");
        assert!(
            error
                .to_string()
                .contains("list requires an item component"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn list_item_nested_and_custom_are_rejected_in_v1() {
        let nested = list_options_from_tokens(quote! { item = nested })
            .expect_err("item = nested must be rejected in v1");
        assert!(
            nested.to_string().contains("not supported yet"),
            "unexpected error: {nested}"
        );

        let custom =
            ListItemSpec::from_string("custom").expect_err("item = custom must be rejected in v1");
        assert!(
            custom.to_string().contains("not supported"),
            "unexpected error: {custom}"
        );
    }

    #[test]
    fn list_item_unknown_kind_is_rejected() {
        let error = ListItemSpec::from_string("banana").expect_err("unknown kinds are rejected");
        assert!(
            error.to_string().contains("banana"),
            "the unknown value should appear in the error: {error}"
        );
    }

    #[test]
    fn nested_and_list_never_wrap_in_option_and_are_value_only() {
        let nested = components_from_tokens(quote! { nested }).expect("component(nested) word");
        assert_eq!(
            nested.kind(),
            gpui_form_schema::components::ComponentKind::Nested
        );
        assert!(!nested.wraps_in_option());

        let list = list_options_from_tokens(quote! { item = input }).expect("list options");
        let list = Components::List(list);
        assert_eq!(
            list.kind(),
            gpui_form_schema::components::ComponentKind::List
        );
        assert!(!list.wraps_in_option());
    }

    #[test]
    fn nested_and_list_emit_no_entity_layout() {
        // Value-only in v1: no `FormFields` entity field / `FormComponents`
        // constructor — the same no-op layout shape as checkbox/switch.
        let nested = components_from_tokens(quote! { nested }).expect("component(nested) word");
        let layout =
            nested.generate_field_layout("address".to_string(), syn::parse_quote!(Address), None);
        assert!(layout.field_structure_tokens.is_empty());
        assert!(layout.field_base_declarations_tokens.is_empty());
        assert!(!layout.wrap_in_option);

        let list = Components::List(
            list_options_from_tokens(quote! { item = input }).expect("list options"),
        );
        let layout =
            list.generate_field_layout("tags".to_string(), syn::parse_quote!(Vec<String>), None);
        assert!(layout.field_structure_tokens.is_empty());
        assert!(layout.field_base_declarations_tokens.is_empty());
        assert!(!layout.wrap_in_option);
    }

    #[test]
    fn behaviour_tokens_carry_nested_form_struct_and_list_item_kind() {
        let nested = components_from_tokens(quote! { nested }).expect("component(nested) word");
        let tokens = nested
            .behaviour_tokens(&syn::parse_quote!(some_lib::structs::address::Address))
            .to_string();
        assert!(
            tokens.contains("ComponentsBehaviour :: Nested"),
            "nested behaviour tokens: {tokens}"
        );
        assert!(
            tokens.contains("form_struct : \"Address\""),
            "the nested form struct is the type's last segment: {tokens}"
        );

        let list = Components::List(
            list_options_from_tokens(quote! { item = number_input }).expect("list options"),
        );
        let tokens = list
            .behaviour_tokens(&syn::parse_quote!(Vec<u32>))
            .to_string();
        assert!(
            tokens.contains("ComponentsBehaviour :: List"),
            "list behaviour tokens: {tokens}"
        );
        assert!(
            tokens.contains(
                "item : :: gpui_form :: schema :: components :: ComponentKind :: NumberInput"
            ),
            "the scalar item kind is recorded: {tokens}"
        );
        assert!(
            tokens.contains("item_struct : None"),
            "v1 records no item struct: {tokens}"
        );
    }
}
