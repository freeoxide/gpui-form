use es_fluent::{EsFluent, EsFluentKv, EsFluentThis};
use gpui_form::{GpuiForm, SelectItem};
use nutype::nutype;
use rust_decimal::Decimal;
use strum::EnumIter;

#[nutype(
    validate(greater_or_equal = 18, less_or_equal = 150),
    default = 18,
    derive(
        Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, FromStr, Display, Deref, Into
    )
)]
pub struct Age(u8);

#[nutype(
    validate(len_char_min = 3, len_char_max = 50),
    default = "example",
    derive(Clone, Debug, Default, PartialEq, Eq, FromStr, Display, Deref, Into)
)]
pub struct Username(String);

#[nutype(
    validate(not_empty),
    default = "example@example.com",
    derive(Clone, Debug, Default, PartialEq, Eq, FromStr, Display, Deref, Into)
)]
pub struct Email(String);

#[nutype(
    validate(predicate = |b| *b >= Decimal::ZERO),
    default = Decimal::ZERO,
    derive(Clone, Copy, Debug, Default, PartialEq, Eq, FromStr, Display, Deref, Into)
)]
pub struct Balance(Decimal);

#[derive(Clone, Debug, Default, EnumIter, EsFluent, PartialEq, SelectItem)]
pub enum NutypePreferedLanguage {
    #[default]
    English,
    French,
    Chinese,
}

#[derive(Clone, Debug, Default, EnumIter, EsFluent, PartialEq, SelectItem)]
pub enum NutypeEnumCountry {
    #[default]
    UnitedStates,
    France,
    China,
}

#[derive(Clone, Debug, Default, EsFluentKv, EsFluentThis, GpuiForm)]
#[fluent_this(origin, members)]
#[fluent_kv(keys = ["description", "label"])]
pub struct NutypeUser {
    /// item = String allows typing any string, validated on blur
    #[gpui_form(item = String, component(input))]
    pub username: Option<Username>,

    /// item = String allows typing any string, validated on blur
    #[gpui_form(item = String, component(input))]
    pub email: Email,

    /// item = u8 allows typing any u8 value, validated on blur
    #[gpui_form(item = u8, component(number_input))]
    pub age: Age,

    /// item = Decimal allows typing any decimal, validated on blur
    #[gpui_form(item = Decimal, component(number_input))]
    pub balance: Balance,

    #[gpui_form(component(checkbox))]
    pub subscribe_newsletter: bool,

    #[gpui_form(component(switch))]
    pub enable_notifications: bool,

    #[gpui_form(component(select(default)))]
    pub preferred: NutypePreferedLanguage,

    #[gpui_form(component(select(searchable, index = NutypeEnumCountry::France)))]
    pub country: Option<NutypeEnumCountry>,

    #[gpui_form(component(date_picker))]
    pub birth_date: Option<chrono::NaiveDate>,
}
