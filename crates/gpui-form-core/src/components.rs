use darling::{Error as DarlingError, FromMeta};
use heck::ToPascalCase as _;
use quote::quote;
use strum::{Display, EnumDiscriminants, EnumString, IntoStaticStr};

pub trait ComponentOption {}

pub trait ComponentDefinition {
    fn component_name() -> &'static str;
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

macro_rules! impl_component_option {
    ($($option:ty),+ $(,)?) => {
        $(impl ComponentOption for $option {})+
    };
}

macro_rules! define_component_definition {
    ($component:ident, $options:ty, $variant:ident) => {
        pub struct $component(pub FieldInformation<$options>);

        impl ComponentDefinition for $component {
            fn component_name() -> &'static str {
                ComponentsDiscriminants::$variant.into()
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
    /// Field-level default value as a path expression (e.g., EnumCountry::France)
    /// This is set by the derive macro when the field has a `default = ...` attribute
    field_default: Option<syn::Path>,
}

impl FromMeta for SelectOptions {
    fn from_word() -> darling::Result<Self> {
        Ok(SelectOptions::default())
    }

    fn from_list(items: &[darling::ast::NestedMeta]) -> darling::Result<Self> {
        let behaviour = BehaviourSelectOptions::from_list(items)?;
        Ok(SelectOptions {
            behaviour,
            field_default: None,
        })
    }
}

impl SelectOptions {
    /// Set the field-level default value
    pub fn with_field_default(mut self, default: Option<syn::Path>) -> Self {
        self.field_default = default;
        self
    }

    /// Get the named index from field default, if specified
    pub fn named_index(&self) -> Option<&syn::Path> {
        self.field_default.as_ref()
    }

    /// Check if we should use the enum's default variant
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
        Ok(NumberInputOptions::default())
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

        Ok(NumberInputOptions { r#as })
    }
}

#[derive(Clone, Debug, FromMeta)]
pub struct CheckboxOptions;
#[derive(Clone, Debug, FromMeta)]
pub struct SwitchOptions;
#[derive(Clone, Debug, FromMeta)]
pub struct DatePickerOptions;

fn default_custom_wraps_in_option() -> bool {
    true
}

#[derive(Clone, Debug)]
pub struct CustomOptions {
    /// Path to a type implementing `gpui_form_component::custom::CustomComponentShape`.
    pub shape: syn::Path,
    /// UI component type path (e.g. `TagsInput`).
    /// When provided, the prototyping code generator emits `Component::new(&entity)`.
    pub component: Option<syn::Path>,
    /// Whether the value holder should store this field as `Option<T>`.
    /// Defaults to `true`.
    pub wraps_in_option: bool,
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
}

impl CustomOptions {
    fn from_meta(meta: CustomOptionsMeta) -> darling::Result<Self> {
        let CustomOptionsMeta {
            shape,
            state,
            component,
            wraps_in_option,
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

/// Options for InfiniteSelect - a cascading select for infinite select enums.
///
/// InfiniteSelect generates multiple select fields that cascade:
/// when the master select changes, the slave selects update their options.
#[derive(Clone, Debug, Default, Eq, FromMeta, PartialEq)]
pub struct BehaviourInfiniteSelectOptions {
    /// Whether each select level should be searchable
    #[darling(default)]
    pub searchable: bool,
    /// Maximum depth to expand (None = expand all levels)
    #[darling(default)]
    pub max_depth: Option<usize>,
}

#[derive(Clone, Debug, Default)]
pub struct InfiniteSelectOptions {
    pub behaviour: BehaviourInfiniteSelectOptions,
    /// Field-level default value as a path expression (e.g., EnumCountry::France)
    /// This is set by the derive macro when the field has a `default = ...` attribute
    field_default: Option<syn::Path>,
}

impl FromMeta for InfiniteSelectOptions {
    fn from_word() -> darling::Result<Self> {
        Ok(InfiniteSelectOptions::default())
    }

    fn from_list(items: &[darling::ast::NestedMeta]) -> darling::Result<Self> {
        let behaviour = BehaviourInfiniteSelectOptions::from_list(items)?;
        Ok(InfiniteSelectOptions {
            behaviour,
            field_default: None,
        })
    }
}

impl InfiniteSelectOptions {
    /// Set the field-level default value
    pub fn with_field_default(mut self, default: Option<syn::Path>) -> Self {
        self.field_default = default;
        self
    }

    /// Get the named index from field default, if specified
    pub fn named_index(&self) -> Option<&syn::Path> {
        self.field_default.as_ref()
    }

    /// Check if we should use the enum's default variant
    pub fn use_enum_default(&self) -> bool {
        self.field_default.is_none()
    }
}

impl_component_option!(
    BehaviourSelectOptions,
    SelectOptions,
    InputOptions,
    NumberInputOptions,
    CheckboxOptions,
    SwitchOptions,
    DatePickerOptions,
    CustomOptions,
    BehaviourInfiniteSelectOptions,
    InfiniteSelectOptions,
);

#[derive(Clone, Debug, EnumDiscriminants, FromMeta)]
#[strum_discriminants(derive(EnumString, Display, IntoStaticStr))]
#[strum_discriminants(vis(pub))]
#[strum_discriminants(strum(serialize_all = "snake_case"))]
#[darling(rename_all = "snake_case")]
pub enum Components {
    Input,
    NumberInput(NumberInputOptions),
    Checkbox,
    Switch,
    Select(SelectOptions),
    InfiniteSelect(InfiniteSelectOptions),
    Custom(CustomOptions),
    DatePicker,
}

define_component_definition!(InputComponent, InputOptions, Input);
define_component_definition!(NumberInputComponent, NumberInputOptions, NumberInput);
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

impl Components {
    /// Returns whether this component's value should be wrapped in Option in the FormValueHolder.
    ///
    /// Components where an empty/missing value is meaningful (like text inputs) return true.
    /// Components that always have a defined value (like checkboxes, switches, selects) return false.
    pub fn wraps_in_option(&self) -> bool {
        match self {
            // Text-based inputs: empty string represents "no value"
            Components::Input | Components::NumberInput(_) => true,
            // Always have a defined state (checked/unchecked, selected item)
            Components::Checkbox
            | Components::Switch
            | Components::Select(_)
            | Components::InfiniteSelect(_) => false,
            Components::Custom(options) => options.wraps_in_option,
            // Date picker already handles Option internally
            Components::DatePicker => false,
        }
    }
}

#[derive(Clone, Debug, Display, EnumString, Eq, IntoStaticStr, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub enum ComponentsBehaviour {
    Input,
    NumberInput,
    Checkbox,
    Switch,
    Select(BehaviourSelectOptions),
    InfiniteSelect(BehaviourInfiniteSelectOptions),
    Custom,
    DatePicker,
}

impl ComponentsBehaviour {
    pub fn as_component_ident(&self) -> proc_macro2::TokenStream {
        let variant: &'static str = self.clone().into();
        let ident = syn::parse_str::<syn::Ident>(&variant.to_pascal_case()).unwrap();
        quote! { #ident }
    }

    pub fn is_value_only_field(&self) -> bool {
        matches!(
            self,
            ComponentsBehaviour::Checkbox | ComponentsBehaviour::Switch
        )
    }

    pub fn needs_value_field(&self) -> bool {
        matches!(self, ComponentsBehaviour::NumberInput)
    }

    pub fn partial(&self) -> bool {
        match self {
            ComponentsBehaviour::Select(options) => options.partial,
            _ => false,
        }
    }

    pub fn subscribable(&self) -> bool {
        matches!(
            self,
            ComponentsBehaviour::Input
                | ComponentsBehaviour::NumberInput
                | ComponentsBehaviour::Select(_)
                | ComponentsBehaviour::InfiniteSelect(_)
                | ComponentsBehaviour::DatePicker
        )
    }

    pub fn focusable(&self) -> bool {
        matches!(
            self,
            ComponentsBehaviour::Input
                | ComponentsBehaviour::NumberInput
                | ComponentsBehaviour::Select(_)
                | ComponentsBehaviour::InfiniteSelect(_)
        )
    }
}
