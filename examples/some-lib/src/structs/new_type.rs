use koruma_collection::numeric::NonNegativeValidation;

#[derive(
    koruma::Koruma,
    koruma::KorumaAllDisplay,
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[display("{}", value)]
#[koruma(try_new, newtype)]
pub struct Age {
    #[koruma(NonNegativeValidation::<_>::builder())]
    pub value: i32,
}

#[cfg(feature = "validation")]
impl std::fmt::Display for AgeKorumaValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "ui", derive(gpui_form::GpuiForm,))]
#[cfg_attr(
    feature = "validation",
    derive(koruma::Koruma, koruma::KorumaAllDisplay)
)]
#[cfg_attr(feature = "ui", gpui_form(koruma(fluent)))]
pub struct Item {
    #[cfg_attr(feature = "ui", gpui_form(component(number_input)))]
    #[cfg_attr(feature = "validation", koruma(newtype))]
    pub index: Age,
}
