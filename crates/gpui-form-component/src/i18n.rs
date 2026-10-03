use rust_i18n::t;

pub use gpui_form_i18n::{
    CurrentLanguage, I18n, Language, LocalizationError, change_locale, fallback_label,
    fallback_message, humanize_key, init, init_with_language, localize_label, localize_message,
    locale, replace_with_language, try_localize_message,
};

#[derive(Clone, Debug)]
pub(crate) enum DatePickerText {
    SelectDate,
}

impl DatePickerText {
    fn key(&self) -> &'static str {
        match self {
            Self::SelectDate => "date_picker.select_date",
        }
    }

    pub(crate) fn default_text(&self) -> String {
        t!(self.key()).to_string()
    }
}

#[derive(Clone, Debug)]
pub(crate) enum FilePickerText {
    SelectAFile,
    SelectADirectory,
    SelectAFileOrDirectory,
    SelectFile,
    SelectDirectory,
    SelectFileOrDirectory,
    Browse,
    DialogDropped,
    PathsSelected { count: usize },
}

impl FilePickerText {
    fn key(&self) -> &'static str {
        match self {
            Self::SelectAFile => "file_picker.select_a_file",
            Self::SelectADirectory => "file_picker.select_a_directory",
            Self::SelectAFileOrDirectory => "file_picker.select_a_file_or_directory",
            Self::SelectFile => "file_picker.select_file",
            Self::SelectDirectory => "file_picker.select_directory",
            Self::SelectFileOrDirectory => "file_picker.select_file_or_directory",
            Self::Browse => "file_picker.browse",
            Self::DialogDropped => "file_picker.dialog_dropped",
            Self::PathsSelected { count: 1 } => "file_picker.paths_selected_one",
            Self::PathsSelected { .. } => "file_picker.paths_selected_other",
        }
    }

    pub(crate) fn default_text(&self) -> String {
        match self {
            Self::PathsSelected { count } => t!(self.key(), count = count).to_string(),
            _ => t!(self.key()).to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn langid(tag: &str) -> unic_langid::LanguageIdentifier {
        tag.parse().unwrap()
    }

    const KEYS: &[&str] = &[
        "date_picker.select_date",
        "file_picker.browse",
        "file_picker.dialog_dropped",
        "file_picker.paths_selected_one",
        "file_picker.paths_selected_other",
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
                let translated = t!(key, locale = locale);
                assert_ne!(
                    &*translated,
                    key,
                    "key {key} unresolved for locale {locale}"
                );
            }
        }
    }

    #[test]
    fn resolves_runtime_component_messages() {
        rust_i18n::set_locale("en");
        assert_eq!(DatePickerText::SelectDate.default_text(), "Select date");
        assert_eq!(FilePickerText::Browse.default_text(), "Browse");
        assert_eq!(
            FilePickerText::SelectAFileOrDirectory.default_text(),
            "Select a file or directory"
        );
        assert_eq!(
            FilePickerText::PathsSelected { count: 2 }.default_text(),
            "2 paths selected"
        );

        rust_i18n::set_locale("fr-FR");
        assert_eq!(
            DatePickerText::SelectDate.default_text(),
            "Sélectionner une date"
        );
        assert_eq!(FilePickerText::Browse.default_text(), "Parcourir");
        assert_eq!(
            FilePickerText::PathsSelected { count: 1 }.default_text(),
            "1 chemin sélectionné"
        );
        assert_eq!(
            FilePickerText::PathsSelected { count: 2 }.default_text(),
            "2 chemins sélectionnés"
        );

        rust_i18n::set_locale("zh-CN");
        assert_eq!(DatePickerText::SelectDate.default_text(), "选择日期");
        assert_eq!(FilePickerText::Browse.default_text(), "浏览");
        assert_eq!(
            FilePickerText::PathsSelected { count: 1 }.default_text(),
            "已选择 1 个路径"
        );
        assert_eq!(
            FilePickerText::PathsSelected { count: 2 }.default_text(),
            "已选择 2 个路径"
        );

        rust_i18n::set_locale("en");
    }

    #[test]
    fn bridge_helpers_stay_reexported() {
        let i18n = I18n::new_with_language(langid("zh-CN"));
        assert_eq!(i18n.localize_message("date_picker.select_date"), "选择日期");
        assert_eq!(i18n.localize_message("missing_key"), "Missing Key");
    }
}
