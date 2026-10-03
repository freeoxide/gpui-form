use std::borrow::Borrow;
use std::ops::Deref;
use strum::IntoEnumIterator;
use unic_langid::LanguageIdentifier;

use gpui::App;

rust_i18n::i18n!("locales", fallback = "en");

#[derive(Debug, thiserror::Error)]
pub enum LocalizationError {
    #[error("unsupported language: {0}")]
    UnsupportedLanguage(#[from] unic_langid::LanguageIdentifierError),
}

pub struct I18n {
    locale: rust_i18n::AtomicStr,
}

impl Clone for I18n {
    fn clone(&self) -> Self {
        let locale = self.locale.as_str().to_string();
        Self {
            locale: rust_i18n::AtomicStr::from(locale.as_str()),
        }
    }
}

impl Default for I18n {
    fn default() -> Self {
        Self::new()
    }
}

impl I18n {
    pub fn new() -> Self {
        Self {
            locale: rust_i18n::AtomicStr::from("en"),
        }
    }

    pub fn new_with_language(language: impl Into<LanguageIdentifier>) -> Self {
        let locale = language.into().to_string();
        Self {
            locale: rust_i18n::AtomicStr::from(locale.as_str()),
        }
    }

    pub fn locale(&self) -> impl Deref<Target = str> + '_ {
        self.locale.as_str()
    }

    pub fn select_language(
        &self,
        language: impl Into<LanguageIdentifier>,
    ) -> Result<(), LocalizationError> {
        let locale = language.into().to_string();
        set_shared_locale(&locale);
        self.locale.replace(locale);
        Ok(())
    }

    pub fn localize_message(&self, key: &str) -> String {
        let locale = self.locale.as_str();
        translate_with_fallback(key, &locale)
    }

    pub fn localize_label(&self, key: &str) -> String {
        self.localize_message(key)
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
        + Default
        + std::fmt::Debug
{
}

#[derive(Clone, Copy)]
pub struct CurrentLanguage<L: Language>(pub L);

impl<L: Language> gpui::Global for CurrentLanguage<L> {}

pub fn init(cx: &mut App) {
    if cx.try_global::<I18n>().is_none() {
        install(I18n::new(), cx);
    }
}

pub fn init_with_language(cx: &mut App, language: impl Into<LanguageIdentifier>) {
    if cx.try_global::<I18n>().is_none() {
        install(I18n::new_with_language(language), cx);
    }
}

pub fn replace_with_language(cx: &mut App, language: impl Into<LanguageIdentifier>) {
    install(I18n::new_with_language(language), cx);
}

pub fn change_locale(
    cx: &mut App,
    language: impl Into<LanguageIdentifier>,
) -> Result<(), LocalizationError> {
    cx.global::<I18n>().select_language(language)
}

pub fn locale(cx: &impl Borrow<App>) -> String {
    cx.borrow()
        .try_global::<I18n>()
        .map(|i18n| i18n.locale().to_string())
        .unwrap_or_else(|| rust_i18n::locale().to_string())
}

pub fn try_localize_message(cx: &impl Borrow<App>, key: &str) -> Option<String> {
    Some(cx.borrow().try_global::<I18n>()?.localize_message(key))
}

pub fn localize_message(cx: &impl Borrow<App>, key: &str) -> String {
    cx.borrow()
        .try_global::<I18n>()
        .map(|i18n| i18n.localize_message(key))
        .unwrap_or_else(|| fallback_message(key))
}

pub fn localize_label(cx: &impl Borrow<App>, key: &str) -> String {
    localize_message(cx, key)
}

pub fn fallback_message(key: &str) -> String {
    humanize_key(key)
}

pub fn fallback_label(key: &str) -> String {
    humanize_key(key)
}

fn install(i18n: I18n, cx: &mut App) {
    let locale = i18n.locale().to_string();
    set_shared_locale(&locale);
    cx.set_global(i18n);
}

fn set_shared_locale(locale: &str) {
    #[cfg(feature = "component")]
    gpui_kit::component::set_locale(locale);
    #[cfg(not(feature = "component"))]
    rust_i18n::set_locale(locale);
}

fn translate_with_fallback(key: &str, locale: &str) -> String {
    let translated = rust_i18n::t!(key, locale = locale);
    if &*translated == key {
        humanize_key(key)
    } else {
        translated.to_string()
    }
}

pub fn humanize_key(key: &str) -> String {
    let key = key.strip_suffix("_label").unwrap_or(key);
    key.split(['.', '_', '-'])
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
    gpui_kit::component::locale()
        .parse::<LanguageIdentifier>()
        .or_else(|_| fallback.parse::<LanguageIdentifier>())
        .expect("fallback language must be a valid language identifier")
}

#[cfg(feature = "component")]
pub fn init_from_component_locale(cx: &mut App, fallback: &str) {
    init_with_language(cx, component_language(fallback));
}

#[cfg(feature = "component")]
pub fn set_component_locale(
    cx: &mut App,
    locale: impl AsRef<str>,
    fallback: &str,
) -> Result<LanguageIdentifier, LocalizationError> {
    let language = locale
        .as_ref()
        .parse::<LanguageIdentifier>()
        .unwrap_or_else(|_| {
            fallback
                .parse()
                .expect("fallback language must be a valid language identifier")
        });

    gpui_kit::component::set_locale(&language.to_string());
    replace_with_language(cx, language.clone());
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

#[cfg(test)]
mod tests {
    use super::*;
    fn langid(tag: &str) -> LanguageIdentifier {
        tag.parse().unwrap()
    }

    const KEYS: &[&str] = &[
        "date_picker.select_date",
        "file_picker.browse",
        "file_picker.dialog_dropped",
        "file_picker.paths_selected",
        "file_picker.select_a_directory",
        "file_picker.select_a_file",
        "file_picker.select_a_file_or_directory",
        "file_picker.select_directory",
        "file_picker.select_file",
        "file_picker.select_file_or_directory",
    ];

    const LOCALES: &[&str] = &["en", "fr-FR", "zh-CN"];

    #[test]
    fn every_key_resolves_in_every_locale() {
        for key in KEYS.iter().copied() {
            for locale in LOCALES.iter().copied() {
                let translated = rust_i18n::t!(key, locale = locale);
                assert_ne!(
                    &*translated, key,
                    "key {key} unresolved for locale {locale}"
                );
            }
        }
    }

    #[test]
    fn en_locale_file_matches_key_inventory() {
        let yml = include_str!("../locales/en.yml");
        let declared = yml
            .lines()
            .filter(|line| line.starts_with("  ") && line.contains(": "))
            .count();
        assert_eq!(declared, KEYS.len());
        for key in KEYS {
            let leaf = key.split('.').next_back().unwrap();
            assert!(yml.contains(&format!("  {leaf}:")), "missing key {leaf}");
        }
    }

    #[test]
    fn interpolates_count_arguments() {
        assert_eq!(
            rust_i18n::t!("file_picker.paths_selected", locale = "en", count = 2),
            "2 paths selected"
        );
        assert_eq!(
            rust_i18n::t!("file_picker.paths_selected", locale = "fr-FR", count = 2),
            "2 chemins sélectionnés"
        );
        assert_eq!(
            rust_i18n::t!("file_picker.paths_selected", locale = "zh-CN", count = 2),
            "已选择 2 个路径"
        );
    }

    #[test]
    fn humanizes_missing_keys() {
        assert_eq!(
            translate_with_fallback("definitely_missing_key", "fr-FR"),
            "Definitely Missing Key"
        );
        assert_eq!(
            humanize_key("file_picker.browse_label"),
            "File Picker Browse"
        );
        assert_eq!(fallback_label("some_label"), "Some");
    }

    #[test]
    fn localize_message_uses_instance_locale() {
        let i18n = I18n::new_with_language(langid("zh-CN"));
        assert_eq!(i18n.localize_message("date_picker.select_date"), "选择日期");
        assert_eq!(i18n.localize_message("missing_key"), "Missing Key");
    }

    #[test]
    fn select_language_switches_shared_locale() {
        let i18n = I18n::new_with_language(langid("en"));
        i18n.select_language(langid("fr-FR")).unwrap();
        assert_eq!(&*i18n.locale(), "fr-FR");
        assert_eq!(&*rust_i18n::locale(), "fr-FR");
        assert_eq!(rust_i18n::t!("file_picker.browse"), "Parcourir");

        i18n.select_language(langid("zh-CN")).unwrap();
        assert_eq!(rust_i18n::t!("date_picker.select_date"), "选择日期");

        i18n.select_language(langid("en")).unwrap();
        assert_eq!(rust_i18n::t!("file_picker.browse"), "Browse");
    }
}
