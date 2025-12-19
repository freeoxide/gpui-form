use darling::FromMeta;
use gpui_form_internal_macros::{ComponentDefinitions, ComponentOption};
use heck::ToPascalCase as _;
use quote::quote;
use strum::{Display, EnumDiscriminants, EnumString, IntoStaticStr};

fn default_true() -> bool {
    true
}

pub trait ComponentOption {}

pub trait ComponentDefinition {
    fn component_name() -> &'static str;
}

pub struct FieldInformation<T: ComponentOption> {
    pub options: T,
    pub name: String,
    pub r#type: syn::Ident,
    /// The intermediate type used for parsing during input.
    /// If Some, this type is used for parsing and the field type is used for final validation.
    /// If None, the field type is used for both parsing and validation.
    pub item_type: Option<syn::Ident>,
}

impl<T: ComponentOption> FieldInformation<T> {
    pub fn new(options: T, name: String, r#type: syn::Ident) -> Self {
        Self {
            options,
            name,
            r#type,
            item_type: None,
        }
    }

    /// Create a FieldInformation with a separate item type for two-phase validation.
    /// The item_type is used for intermediate parsing (e.g., u32),
    /// while the field type (r#type) is used for final validation (e.g., Age nutype).
    pub fn with_item_type(
        options: T,
        name: String,
        r#type: syn::Ident,
        item_type: Option<syn::Ident>,
    ) -> Self {
        Self {
            options,
            name,
            r#type,
            item_type,
        }
    }

    /// Returns true if this field uses two-phase validation (has a separate item type)
    pub fn has_item_type(&self) -> bool {
        self.item_type.is_some()
    }

    /// Returns the type to use for parsing input values.
    /// If item_type is set, returns that; otherwise returns the field type.
    pub fn parse_type(&self) -> &syn::Ident {
        self.item_type.as_ref().unwrap_or(&self.r#type)
    }

    /// Returns the type to use for final validation.
    /// This is always the field type.
    pub fn validation_type(&self) -> &syn::Ident {
        &self.r#type
    }
}

#[derive(Clone, ComponentOption, Debug, Default, Eq, FromMeta, PartialEq)]
pub struct BehaviourSelectOptions {
    #[darling(default)]
    pub partial: bool,
    #[darling(default)]
    pub searchable: bool,
}

#[derive(Clone, ComponentOption, Debug, Eq, FromMeta, PartialEq)]
pub struct BehaviourCustomOptions {
    #[darling(default = "default_true", rename = "uw")]
    pub should_be_unwrapped: bool,
    #[darling(default)]
    pub partial: bool,
    pub name: syn::Ident,
}

#[derive(Clone, ComponentOption, Debug, FromMeta)]
pub struct CustomOptions {
    #[darling(flatten)]
    pub behaviour: BehaviourCustomOptions,
}

#[derive(Clone, ComponentOption, Debug, FromMeta)]
pub struct SelectOptions {
    #[darling(flatten)]
    pub behaviour: BehaviourSelectOptions,
    #[darling(default, rename = "index")]
    named_index: Option<syn::Path>,
    #[darling(default, rename = "default")]
    index_default: bool,
}

impl SelectOptions {
    pub fn named_index(&self) -> Option<&syn::Path> {
        if self.named_index.is_some() && self.index_default {
            panic!("Cannot specify both named_index and index_default");
        }
        self.named_index.as_ref()
    }

    pub fn index_default(&self) -> bool {
        if self.named_index.is_some() && self.index_default {
            panic!("Cannot specify both named_index and index_default");
        }
        self.index_default
    }
}

#[derive(Clone, ComponentOption, Debug, Default, FromMeta)]
pub struct InputOptions;

#[derive(Clone, ComponentOption, Debug, Default, FromMeta)]
pub struct NumberInputOptions;

#[derive(Clone, ComponentOption, Debug, FromMeta)]
pub struct CheckboxOptions;
#[derive(Clone, ComponentOption, Debug, FromMeta)]
pub struct SwitchOptions;
#[derive(Clone, ComponentOption, Debug, FromMeta)]
pub struct DatePickerOptions;

/// Options for TupleSelect - a cascading select for tuple enums.
///
/// TupleSelect generates multiple select fields that cascade:
/// when the master select changes, the slave selects update their options.
#[derive(Clone, ComponentOption, Debug, Default, Eq, FromMeta, PartialEq)]
pub struct BehaviourTupleSelectOptions {
    /// Whether each select level should be searchable
    #[darling(default)]
    pub searchable: bool,
    /// Maximum depth to expand (None = expand all levels)
    #[darling(default)]
    pub max_depth: Option<usize>,
}

#[derive(Clone, ComponentOption, Debug, FromMeta)]
pub struct TupleSelectOptions {
    #[darling(flatten)]
    pub behaviour: BehaviourTupleSelectOptions,
    /// Initial value path for the selection
    #[darling(default, rename = "index")]
    named_index: Option<syn::Path>,
    /// Use default value for initial selection
    #[darling(default, rename = "default")]
    index_default: bool,
}

impl TupleSelectOptions {
    pub fn named_index(&self) -> Option<&syn::Path> {
        if self.named_index.is_some() && self.index_default {
            panic!("Cannot specify both named_index and index_default");
        }
        self.named_index.as_ref()
    }

    pub fn index_default(&self) -> bool {
        if self.named_index.is_some() && self.index_default {
            panic!("Cannot specify both named_index and index_default");
        }
        self.index_default
    }
}

#[derive(Clone, ComponentDefinitions, Debug, EnumDiscriminants, FromMeta)]
#[strum_discriminants(derive(EnumString, Display, IntoStaticStr))]
#[strum_discriminants(vis(pub))]
#[strum_discriminants(strum(serialize_all = "snake_case"))]
#[darling(rename_all = "snake_case")]
pub enum Components {
    Input,
    NumberInput,
    Checkbox,
    Switch,
    Select(SelectOptions),
    TupleSelect(TupleSelectOptions),
    DatePicker,
    Custom(CustomOptions),
}

#[derive(Clone, Debug, Display, EnumString, Eq, IntoStaticStr, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub enum ComponentsBehaviour {
    Input,
    NumberInput,
    Checkbox,
    Switch,
    Select(BehaviourSelectOptions),
    TupleSelect(BehaviourTupleSelectOptions),
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
                | ComponentsBehaviour::TupleSelect(_)
                | ComponentsBehaviour::DatePicker
        )
    }

    pub fn focusable(&self) -> bool {
        matches!(
            self,
            ComponentsBehaviour::Input
                | ComponentsBehaviour::NumberInput
                | ComponentsBehaviour::Select(_)
                | ComponentsBehaviour::TupleSelect(_)
        )
    }
}
