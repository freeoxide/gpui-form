use es_fluent::{EsFluent, EsFluentThis, EsFluentVariants};
use gpui_form::{GpuiForm, SelectItem};
use koruma::{Koruma, KorumaAllFluent};
use koruma_collection::{
    collection::NonEmptyValidation,
    format::EmailValidation,
    general::RequiredValidation,
    numeric::{NegativeValidation, PositiveValidation, RangeValidation},
    string::{PrefixValidation, SuffixValidation},
};
use rust_decimal::Decimal;
use strum::EnumIter;

#[derive(Clone, Debug, Default, EnumIter, EsFluent, PartialEq, SelectItem)]
#[select_item(fluent)]
pub enum PreferedLanguage {
    #[default]
    English,
    French,
    Chinese,
}

#[derive(Clone, Debug, Default, EnumIter, EsFluent, PartialEq, SelectItem)]
#[select_item(fluent)]
pub enum EnumCountry {
    #[default]
    UnitedStates,
    France,
    China,
}

#[derive(Clone, Debug, EsFluentThis, EsFluentVariants, GpuiForm, Koruma, KorumaAllFluent)]
#[fluent_this(origin, members)]
#[fluent_variants(keys = ["description", "label"])]
#[gpui_form(koruma(fluent))]
pub struct User {
    #[gpui_form(component(input))]
    #[koruma(NonEmptyValidation::<_>, RequiredValidation::<Option<_>>, PrefixValidation::<_>(prefix = "Xx"), SuffixValidation::<_>(suffix = "xX"))]
    pub username: Option<String>,

    #[gpui_form(component(input))]
    #[koruma(EmailValidation::<_>)]
    pub email: String,

    #[gpui_form(component(number_input))]
    #[koruma(RangeValidation::<_>(min = 18, max = 167))]
    pub age: Option<u32>,

    #[gpui_form(component(number_input(as = f64)))]
    #[koruma(PositiveValidation::<_>)]
    pub balance: Decimal,

    #[gpui_form(component(number_input(as = f64)))]
    #[koruma(NegativeValidation::<_>)]
    pub debt: Decimal,

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
    #[fluent_variants(skip)]
    pub skip_me: bool,
}
