use es_fluent::{EsFluent, EsFluentKv, EsFluentThis};
use gpui_form::{GpuiForm, SelectItem};
use rust_decimal::Decimal;
use strum::EnumIter;

#[derive(Clone, Debug, Default, EnumIter, EsFluent, PartialEq, SelectItem)]
pub enum PreferedLanguage {
    #[default]
    English,
    French,
    Chinese,
}

#[derive(Clone, Debug, Default, EnumIter, EsFluent, PartialEq, SelectItem)]
pub enum EnumCountry {
    #[default]
    UnitedStates,
    France,
    China,
}

#[derive(Clone, Debug, Default, EsFluentKv, EsFluentThis, GpuiForm)]
#[fluent_this(origin, members)]
#[fluent_kv(keys = ["description", "label"])]
pub struct User {
    #[gpui_form(component(input))]
    pub username: Option<String>,

    #[gpui_form(component(input))]
    pub email: String,

    #[gpui_form(component(number_input))]
    pub age: Option<u32>,

    #[gpui_form(component(number_input))]
    pub balance: Decimal,

    #[gpui_form(component(checkbox))]
    pub subscribe_newsletter: bool,

    #[gpui_form(component(switch))]
    pub enable_notifications: bool,

    #[gpui_form(component(select(default)))]
    pub preferred: PreferedLanguage,

    #[gpui_form(component(select(searchable, index = EnumCountry::France)))]
    pub country: Option<EnumCountry>,

    #[gpui_form(component(date_picker))]
    pub birth_date: Option<chrono::NaiveDate>,

    #[gpui_form(skip)]
    #[fluent_kv(skip)]
    pub skip_me: bool,
}
