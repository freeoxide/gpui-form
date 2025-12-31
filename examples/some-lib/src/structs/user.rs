use crate::validators::{
    EmailValidation, NonEmptyStringValidation, NumberRangeValidation, PositiveNumberValidation,
};
use es_fluent::{EsFluent, EsFluentKv, EsFluentThis};
use gpui_form::{GpuiForm, SelectItem};
use koruma::{Koruma, Validate};
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

#[derive(Clone, Debug, Default, EsFluentKv, EsFluentThis, GpuiForm, Koruma)]
#[fluent_this(origin, members)]
#[fluent_kv(keys = ["description", "label"])]
pub struct User {
    #[gpui_form(component(input))]
    #[koruma(NonEmptyStringValidation)]
    pub username: Option<String>,

    #[gpui_form(component(input))]
    #[koruma(NonEmptyStringValidation, EmailValidation)]
    pub email: String,

    #[gpui_form(component(number_input))]
    #[koruma(NumberRangeValidation(min = 18, max = 167))]
    pub age: Option<u32>,

    #[gpui_form(component(number_input))]
    #[koruma(PositiveNumberValidation)]
    pub balance: f64,

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
