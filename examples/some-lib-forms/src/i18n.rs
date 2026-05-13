use es_fluent::{FluentLabel, FluentLocalizer, FluentLocalizerExt as _, FluentMessage};
use es_fluent_manager_embedded::{EmbeddedI18n, EmbeddedInitError, LocalizationError};

es_fluent_manager_embedded::define_i18n_module!();

#[derive(Clone)]
pub struct I18n {
    manager: EmbeddedI18n,
}

impl I18n {
    pub fn new(
        language: impl Into<es_fluent::unic_langid::LanguageIdentifier>,
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
        language: impl Into<es_fluent::unic_langid::LanguageIdentifier>,
    ) -> Result<(), LocalizationError> {
        self.manager.select_language(language)
    }
}

impl gpui::Global for I18n {}

struct FallbackLocalizer;

impl FluentLocalizer for FallbackLocalizer {
    fn localize<'a>(
        &self,
        id: &str,
        _args: Option<&std::collections::HashMap<&str, es_fluent::FluentValue<'a>>>,
    ) -> Option<String> {
        Some(humanize_key(id))
    }

    fn localize_in_domain<'a>(
        &self,
        _domain: &str,
        id: &str,
        _args: Option<&std::collections::HashMap<&str, es_fluent::FluentValue<'a>>>,
    ) -> Option<String> {
        Some(humanize_key(id))
    }
}

fn humanize_key(id: &str) -> String {
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

pub fn init(
    cx: &mut gpui::App,
    language: impl Into<es_fluent::unic_langid::LanguageIdentifier>,
) -> Result<(), EmbeddedInitError> {
    cx.set_global(I18n::new(language)?);
    Ok(())
}

pub fn localize_message<T>(cx: &impl std::borrow::Borrow<gpui::App>, message: &T) -> String
where
    T: FluentMessage + ?Sized,
{
    match cx.borrow().try_global::<I18n>() {
        Some(i18n) => i18n.manager().localize_message(message),
        None => FallbackLocalizer.localize_message(message),
    }
}

pub fn localize_label<T>(cx: &impl std::borrow::Borrow<gpui::App>) -> String
where
    T: FluentLabel,
{
    match cx.borrow().try_global::<I18n>() {
        Some(i18n) => T::localize_label(i18n.manager()),
        None => T::localize_label(&FallbackLocalizer),
    }
}

pub fn fallback_label<T>() -> String
where
    T: FluentLabel,
{
    T::localize_label(&FallbackLocalizer)
}

pub fn change_locale(
    cx: &mut gpui::App,
    language: impl Into<es_fluent::unic_langid::LanguageIdentifier>,
) -> Result<(), LocalizationError> {
    cx.global::<I18n>().select_language(language)
}

#[cfg(test)]
mod tests {
    use es_fluent::FluentLabel as _;
    use some_lib::structs::{empty::Empty, user::User};

    #[test]
    fn resolves_form_labels() {
        let i18n = es_fluent_manager_embedded::EmbeddedI18n::try_new_with_language(
            es_fluent::unic_langid::langid!("en"),
        )
        .unwrap();
        assert_eq!(User::localize_label(&i18n), "User");
        assert_eq!(Empty::localize_label(&i18n), "Empty");

        i18n.select_language(es_fluent::unic_langid::langid!("fr-FR"))
            .unwrap();
        assert_eq!(User::localize_label(&i18n), "Utilisateur");
        assert_eq!(Empty::localize_label(&i18n), "Vide");

        i18n.select_language(es_fluent::unic_langid::langid!("zh-CN"))
            .unwrap();
        assert_eq!(User::localize_label(&i18n), "用户");
        assert_eq!(Empty::localize_label(&i18n), "空");
    }
}
