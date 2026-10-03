pub use gpui_form_i18n::{
    CurrentLanguage, I18n, Language, LocalizationError, change_locale, fallback_label,
    fallback_message, humanize_key, init, init_with_language, locale, localize_label,
    localize_message, replace_with_language, try_localize_message,
};

#[cfg(test)]
mod tests {
    use rust_i18n::t;

    const LOCALES: &[&str] = &["en", "fr-FR", "zh-CN"];

    fn declared_keys() -> Vec<String> {
        let yml = include_str!("../locales/en.yml");
        let mut keys = Vec::new();
        let mut namespace: Option<&str> = None;
        for line in yml.lines() {
            if line.starts_with("  ") {
                let ns = namespace.expect("leaf key outside a namespace");
                let leaf = line
                    .trim_start()
                    .split(':')
                    .next()
                    .expect("leaf line always has a colon")
                    .trim();
                keys.push(format!("{ns}.{leaf}"));
            } else if let Some(rest) = line.strip_suffix(':') {
                namespace = Some(rest);
            } else if let Some((key, value)) = line.split_once(": ") {
                if !key.starts_with('#') && !value.is_empty() {
                    keys.push(key.to_string());
                }
            }
        }
        keys
    }

    #[test]
    fn every_key_resolves_in_every_locale() {
        let keys = declared_keys();
        assert!(!keys.is_empty(), "locale inventory must not be empty");
        for key in &keys {
            let key: &str = key;
            for locale in LOCALES.iter().copied() {
                let translated = t!(key, locale = locale);
                assert_ne!(
                    &*translated, key,
                    "key {key} unresolved for locale {locale}"
                );
            }
        }
    }

    #[test]
    fn resolves_ported_form_labels() {
        assert_eq!(t!("user_label", locale = "en"), "User");
        assert_eq!(t!("empty_label", locale = "en"), "Empty");
        assert_eq!(t!("user_label", locale = "fr-FR"), "Utilisateur");
        assert_eq!(t!("empty_label", locale = "fr-FR"), "Vide");
        assert_eq!(t!("user_label", locale = "zh-CN"), "用户");
        assert_eq!(t!("empty_label", locale = "zh-CN"), "空");
    }

    #[test]
    fn resolves_ported_field_and_action_strings() {
        assert_eq!(t!("user.username_label", locale = "en"), "Username");
        assert_eq!(
            t!("user.username_label", locale = "fr-FR"),
            "Nom d'utilisateur"
        );
        assert_eq!(t!("user.username_label", locale = "zh-CN"), "用户名");
        assert_eq!(t!("form_action.submit", locale = "en"), "Submit");
        assert_eq!(t!("form_action.submit", locale = "fr-FR"), "Envoyer");
        assert_eq!(t!("form_action.reset", locale = "zh-CN"), "重置");
        assert_eq!(
            t!("user.username_description", locale = "fr-FR"),
            "Nom d'utilisateur"
        );
        assert_eq!(t!("item.index_label", locale = "zh-CN"), "索引");
        assert_eq!(
            t!("location_form.location_label", locale = "fr-FR"),
            "Emplacement"
        );
    }
}
