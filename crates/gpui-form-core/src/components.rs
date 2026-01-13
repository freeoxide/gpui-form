use darling::FromMeta;
use gpui_form_internal_macros::{ComponentDefinitions, ComponentOption};
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
    pub r#type: syn::Ident,
}

impl<T: ComponentOption> FieldInformation<T> {
    pub fn new(options: T, name: String, r#type: syn::Ident) -> Self {
        Self {
            options,
            name,
            r#type,
        }
    }
}

#[derive(Clone, ComponentOption, Debug, Default, Eq, FromMeta, PartialEq)]
pub struct BehaviourSelectOptions {
    #[darling(default)]
    pub partial: bool,
    #[darling(default)]
    pub searchable: bool,
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

/// Options for InfiniteSelect - a cascading select for infinite select enums.
///
/// InfiniteSelect generates multiple select fields that cascade:
/// when the master select changes, the slave selects update their options.
#[derive(Clone, ComponentOption, Debug, Default, Eq, FromMeta, PartialEq)]
pub struct BehaviourInfiniteSelectOptions {
    /// Whether each select level should be searchable
    #[darling(default)]
    pub searchable: bool,
    /// Maximum depth to expand (None = expand all levels)
    #[darling(default)]
    pub max_depth: Option<usize>,
}

#[derive(Clone, ComponentOption, Debug, FromMeta)]
pub struct InfiniteSelectOptions {
    #[darling(flatten)]
    pub behaviour: BehaviourInfiniteSelectOptions,
    /// Initial value path for the selection
    #[darling(default, rename = "index")]
    named_index: Option<syn::Path>,
    /// Use default value for initial selection
    #[darling(default, rename = "default")]
    index_default: bool,
}

impl InfiniteSelectOptions {
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
    InfiniteSelect(InfiniteSelectOptions),
    DatePicker,
}

impl Components {
    /// Returns whether this component's value should be wrapped in Option in the FormValueHolder.
    ///
    /// Components where an empty/missing value is meaningful (like text inputs) return true.
    /// Components that always have a defined value (like checkboxes, switches, selects) return false.
    pub fn wraps_in_option(&self) -> bool {
        match self {
            // Text-based inputs: empty string represents "no value"
            Components::Input | Components::NumberInput => true,
            // Always have a defined state (checked/unchecked, selected item)
            Components::Checkbox
            | Components::Switch
            | Components::Select(_)
            | Components::InfiniteSelect(_) => false,
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
