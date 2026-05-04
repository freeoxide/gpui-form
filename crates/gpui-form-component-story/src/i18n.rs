use std::sync::OnceLock;

use es_fluent::{EsFluent, FluentLabel, FluentMessage, unic_langid::LanguageIdentifier};
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

#[derive(Clone, Debug, EsFluent)]
#[fluent(namespace = "date_picker")]
pub(crate) enum DatePickerComponentText {
    LaunchPlaceholder,
}

#[derive(Clone, Debug, EsFluent)]
#[fluent(namespace = "file_picker")]
pub(crate) enum FilePickerComponentText {
    SourcePlaceholder,
    OutputPlaceholder,
    ChooseFiles,
}
