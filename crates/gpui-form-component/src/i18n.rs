use rust_i18n::t;

pub use gpui_form_i18n::{
    CurrentLanguage, I18n, Language, LocalizationError, change_locale, fallback_label,
    fallback_message, humanize_key, init, init_with_language, locale, localize_label,
    localize_message, replace_with_language, try_localize_message,
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
    use std::sync::Mutex;
    use std::sync::MutexGuard;

    use super::*;

    fn langid(tag: &str) -> unic_langid::LanguageIdentifier {
        tag.parse().unwrap()
    }

    /// The process-global locale is shared by every locale test in this
    /// binary; tests that write it (or assert against it) hold this lock so
    /// they cannot interleave.
    static LOCALE_LOCK: Mutex<()> = Mutex::new(());

    fn lock_locale() -> MutexGuard<'static, ()> {
        LOCALE_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
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
        for &key in KEYS {
            for &locale in LOCALES {
                let translated = t!(key, locale = locale);
                assert_ne!(
                    &*translated, key,
                    "key {key} unresolved for locale {locale}"
                );
            }
        }
    }

    #[test]
    fn resolves_runtime_component_messages() {
        let _guard = lock_locale();
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
        let _guard = lock_locale();
        let i18n = I18n::new_with_language(langid("zh-CN"));
        assert_eq!(i18n.localize_message("date_picker.select_date"), "选择日期");
        assert_eq!(i18n.localize_message("missing_key"), "Missing Key");

        // The wrapper keeps no locale copy: a direct global write is
        // immediately visible through the bridge.
        rust_i18n::set_locale("en");
        assert_eq!(&*i18n.locale(), "en");
        assert_eq!(i18n.localize_message("file_picker.browse"), "Browse");
        assert_eq!(t!("file_picker.browse"), "Browse");

        rust_i18n::set_locale("zh-CN");
    }

    #[gpui::test]
    async fn change_locale_agrees_across_bridge_and_t(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            let _guard = lock_locale();
            init(cx);

            change_locale(cx, langid("fr-FR")).expect("fr-FR is a supported language");
            assert_eq!(&*rust_i18n::locale(), "fr-FR");
            assert_eq!(
                localize_message(cx, "date_picker.select_date"),
                "Sélectionner une date"
            );
            assert_eq!(t!("date_picker.select_date"), "Sélectionner une date");

            change_locale(cx, langid("zh-CN")).expect("zh-CN is a supported language");
            assert_eq!(localize_message(cx, "file_picker.browse"), "浏览");
            assert_eq!(t!("file_picker.browse"), "浏览");

            // The split is closed in both directions: a direct
            // `rust_i18n::set_locale` shows up through the GPUI-global
            // bridge too, and `change_locale` drives `t!()`.
            rust_i18n::set_locale("en");
            assert_eq!(localize_message(cx, "file_picker.browse"), "Browse");
            assert_eq!(t!("file_picker.browse"), "Browse");
        });
    }
}
