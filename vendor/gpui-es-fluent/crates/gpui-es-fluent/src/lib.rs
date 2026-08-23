use es_fluent::{
    FluentLabel, FluentLocalizer, FluentLocalizerExt as _, FluentMessage, FluentValue,
};
use gpui::App;
use std::{borrow::Borrow, collections::HashMap};
use strum::IntoEnumIterator;
use unic_langid::LanguageIdentifier;

pub use es_fluent_manager_embedded::{EmbeddedI18n, EmbeddedInitError, LocalizationError};

#[derive(Clone)]
pub struct I18n {
    manager: EmbeddedI18n,
}

impl I18n {
    pub fn new() -> Result<Self, EmbeddedInitError> {
        Ok(Self {
            manager: EmbeddedI18n::try_new()?,
        })
    }

    pub fn new_with_language(
        language: impl Into<LanguageIdentifier>,
    ) -> Result<Self, EmbeddedInitError> {
        Ok(Self {
            manager: EmbeddedI18n::try_new_with_language(language)?,
        })
    }

    pub fn manager(&self) -> &EmbeddedI18n {
        &self.manager
    }

    pub fn select_language(
        &self,
        language: impl Into<LanguageIdentifier>,
    ) -> Result<(), LocalizationError> {
        self.manager.select_language(language)
    }

    pub fn localize_message<T>(&self, message: &T) -> String
    where
        T: FluentMessage + ?Sized,
    {
        self.manager.localize_message(message)
    }

    pub fn localize_label<T>(&self) -> String
    where
        T: FluentLabel,
    {
        T::localize_label(&self.manager)
    }
}

impl gpui::Global for I18n {}

pub trait Language:
    'static
    + Copy
    + Clone
    + Send
    + Sync
    + IntoEnumIterator
    + TryInto<LanguageIdentifier>
    + TryFrom<LanguageIdentifier>
    + FluentMessage
    + Default
    + std::fmt::Debug
{
}

impl<T> Language for T where
    T: 'static
        + Copy
        + Clone
        + Send
        + Sync
        + IntoEnumIterator
        + TryInto<LanguageIdentifier>
        + TryFrom<LanguageIdentifier>
        + FluentMessage
        + Default
        + std::fmt::Debug
{
}

#[derive(Clone, Copy)]
pub struct CurrentLanguage<L: Language>(pub L);

impl<L: Language> gpui::Global for CurrentLanguage<L> {}

pub fn init(cx: &mut App) -> Result<(), EmbeddedInitError> {
    if cx.try_global::<I18n>().is_none() {
        cx.set_global(I18n::new()?);
    }
    Ok(())
}

pub fn init_with_language(
    cx: &mut App,
    language: impl Into<LanguageIdentifier>,
) -> Result<(), EmbeddedInitError> {
    if cx.try_global::<I18n>().is_none() {
        cx.set_global(I18n::new_with_language(language)?);
    }
    Ok(())
}

pub fn replace_with_language(
    cx: &mut App,
    language: impl Into<LanguageIdentifier>,
) -> Result<(), EmbeddedInitError> {
    cx.set_global(I18n::new_with_language(language)?);
    Ok(())
}

pub fn change_locale(
    cx: &mut App,
    language: impl Into<LanguageIdentifier>,
) -> Result<(), LocalizationError> {
    cx.global::<I18n>().select_language(language)
}

pub fn try_localize_message<T>(cx: &impl Borrow<App>, message: &T) -> Option<String>
where
    T: FluentMessage + ?Sized,
{
    Some(cx.borrow().try_global::<I18n>()?.localize_message(message))
}

pub fn localize_message<T>(cx: &impl Borrow<App>, message: &T) -> String
where
    T: FluentMessage + ?Sized,
{
    cx.borrow()
        .try_global::<I18n>()
        .map(|i18n| i18n.localize_message(message))
        .unwrap_or_else(|| fallback_message(message))
}

pub fn localize_label<T>(cx: &impl Borrow<App>) -> String
where
    T: FluentLabel,
{
    cx.borrow()
        .try_global::<I18n>()
        .map(I18n::localize_label::<T>)
        .unwrap_or_else(fallback_label::<T>)
}

pub fn fallback_message<T>(message: &T) -> String
where
    T: FluentMessage + ?Sized,
{
    FallbackLocalizer.localize_message(message)
}

pub fn fallback_label<T>() -> String
where
    T: FluentLabel,
{
    T::localize_label(&FallbackLocalizer)
}

pub struct FallbackLocalizer;

impl FluentLocalizer for FallbackLocalizer {
    fn localize<'a>(
        &self,
        id: &str,
        _args: Option<&HashMap<&str, FluentValue<'a>>>,
    ) -> Option<String> {
        Some(humanize_key(id))
    }

    fn localize_in_domain<'a>(
        &self,
        _domain: &str,
        id: &str,
        _args: Option<&HashMap<&str, FluentValue<'a>>>,
    ) -> Option<String> {
        Some(humanize_key(id))
    }
}

pub fn humanize_key(id: &str) -> String {
    let id = id.strip_suffix("_label").unwrap_or(id);
    id.split(['_', '-'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(feature = "component")]
pub fn component_language(fallback: &str) -> LanguageIdentifier {
    gpui_component::locale()
        .parse::<LanguageIdentifier>()
        .or_else(|_| fallback.parse::<LanguageIdentifier>())
        .expect("fallback language must be a valid language identifier")
}

#[cfg(feature = "component")]
pub fn init_from_component_locale(cx: &mut App, fallback: &str) -> Result<(), EmbeddedInitError> {
    init_with_language(cx, component_language(fallback))
}

#[cfg(feature = "component")]
pub fn set_component_locale(
    cx: &mut App,
    locale: impl AsRef<str>,
    fallback: &str,
) -> Result<LanguageIdentifier, EmbeddedInitError> {
    let language = locale
        .as_ref()
        .parse::<LanguageIdentifier>()
        .unwrap_or_else(|_| {
            fallback
                .parse()
                .expect("fallback language must be a valid language identifier")
        });

    gpui_component::set_locale(&language.to_string());
    replace_with_language(cx, language.clone())?;
    Ok(language)
}

#[cfg(feature = "component")]
pub fn sync_component_locale(cx: &impl Borrow<App>, fallback: &str) -> LanguageIdentifier {
    let language = component_language(fallback);
    if let Some(i18n) = cx.borrow().try_global::<I18n>() {
        let _ = i18n.select_language(language.clone());
    }
    language
}
