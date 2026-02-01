use es_fluent::{EsFluentThis, EsFluentVariants};
use gpui_form::SelectItem;
use koruma_collection::{
    collection::NonEmptyValidation,
    format::EmailValidation,
    general::RequiredValidation,
    numeric::{NonNegativeValidation, PositiveValidation},
};
use rust_decimal::Decimal;
use strum::EnumIter;

/// Example enum demonstrating cfg_attr with SelectItem
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
#[derive(Clone, Debug, Default, EnumIter, PartialEq, SelectItem)]
#[select_item(fluent)]
pub enum AccountType {
    #[default]
    Free,
    Premium,
    Enterprise,
}

#[derive(
    koruma::Koruma,
    koruma::KorumaAllFluent,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Default,
    Ord,
    derive_more::Display,
    derive_more::From,
    derive_more::Into,
    derive_more::Deref,
    derive_more::AsRef,
    derive_more::FromStr,
)]
#[display("{}", value)]
#[koruma(try_new, newtype)]
pub struct Age {
    #[koruma(NonNegativeValidation::<_>)]
    pub value: i32,
}

/// Example struct demonstrating cfg_attr for all gpui_form and koruma attributes.
/// This shows that the derive macro correctly handles attributes wrapped in cfg_attr.
///
/// Features required: ui, validation, fluent
#[cfg_attr(feature = "ui", derive(gpui_form::GpuiForm))]
#[cfg_attr(feature = "ui", gpui_form(koruma(fluent)))]
#[cfg_attr(feature = "fluent", derive(EsFluentVariants, EsFluentThis))]
#[cfg_attr(feature = "fluent", fluent_this(origin, members))]
#[cfg_attr(feature = "fluent", fluent_variants(keys = ["description", "label"]))]
#[cfg_attr(
    feature = "validation",
    derive(koruma::Koruma, koruma::KorumaAllFluent)
)]
#[derive(Clone, Debug, Default)]
pub struct CfgAttrExample {
    /// Username field with validation wrapped in cfg_attr
    #[cfg_attr(feature = "ui", gpui_form(component(input)))]
    #[cfg_attr(feature = "validation", koruma(NonEmptyValidation::<_>, RequiredValidation::<Option<_>>))]
    pub username: Option<String>,

    /// Email field with both ui and validation behind cfg_attr
    #[cfg_attr(feature = "ui", gpui_form(component(input)))]
    #[cfg_attr(feature = "validation", koruma(EmailValidation::<_>))]
    pub email: String,

    /// Age field with number input component
    #[cfg_attr(feature = "ui", gpui_form(component(number_input)))]
    #[cfg_attr(feature = "validation", koruma(newtype))]
    pub age: Option<Age>,

    /// Balance field demonstrating decimal with positive validation
    #[cfg_attr(feature = "ui", gpui_form(component(number_input(as = f64))))]
    #[cfg_attr(feature = "validation", koruma(PositiveValidation::<_>))]
    pub balance: Decimal,

    /// Checkbox component
    #[cfg_attr(feature = "ui", gpui_form(component(checkbox)))]
    pub active: bool,

    /// Switch component
    #[cfg_attr(feature = "ui", gpui_form(component(switch)))]
    pub enabled: bool,

    /// Select with enum
    #[cfg_attr(feature = "ui", gpui_form(component(select)))]
    pub account_type: AccountType,

    /// Date picker
    #[cfg_attr(feature = "ui", gpui_form(component(date_picker)))]
    pub created_at: Option<chrono::NaiveDate>,

    /// Field to skip
    #[cfg_attr(feature = "ui", gpui_form(skip))]
    #[cfg_attr(feature = "fluent", fluent_variants(skip))]
    pub internal_id: u64,
}
