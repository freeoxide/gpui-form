use std::sync::OnceLock;

use es_fluent::{FluentLabel, FluentMessage, unic_langid::LanguageIdentifier};
use es_fluent_manager_embedded as i18n_manager;

es_fluent_manager_embedded::define_i18n_module!();

static I18N: OnceLock<i18n_manager::EmbeddedI18n> = OnceLock::new();

pub fn manager() -> &'static i18n_manager::EmbeddedI18n {
    I18N.get_or_init(|| {
        i18n_manager::EmbeddedI18n::try_new()
            .expect("failed to initialize embedded es-fluent manager")
    })
}

pub fn init() -> &'static i18n_manager::EmbeddedI18n {
    manager()
}

pub fn localize<T: FluentMessage + ?Sized>(message: &T) -> String {
    manager().localize_message(message)
}

pub fn localize_label<T: FluentLabel>() -> String {
    T::localize_label(manager())
}

pub fn change_locale<L: Into<LanguageIdentifier>>(
    language: L,
) -> Result<(), i18n_manager::LocalizationError> {
    manager().select_language(language)
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
