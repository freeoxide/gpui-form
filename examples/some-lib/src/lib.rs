rust_i18n::i18n!("locales", fallback = "en");

pub mod structs;

#[cfg(test)]
mod i18n_tests {
    use rust_i18n::t;

    const KEYS: &[&str] = &[
        "enum_country.china",
        "enum_country.france",
        "enum_country.united_states",
        "item.index_label",
        "prefered_language.chinese",
        "prefered_language.english",
        "prefered_language.french",
        "preferred_language.chinese",
        "preferred_language.english",
        "preferred_language.french",
        "user.age_label",
        "user.balance_label",
        "user.birth_date_label",
        "user.country_label",
        "user.debt_label",
        "user.email_label",
        "user.enable_notifications_label",
        "user.preferred_label",
        "user.subscribe_newsletter_label",
        "user.username_label",
        "validation.alphanumeric",
        "validation.ascii",
        "validation.contains",
        "validation.credit_card",
        "validation.email",
        "validation.invalid",
        "validation.ip",
        "validation.len",
        "validation.matches",
        "validation.negative",
        "validation.non_empty",
        "validation.non_negative",
        "validation.non_positive",
        "validation.pattern",
        "validation.phone_number",
        "validation.positive",
        "validation.prefix",
        "validation.range",
        "validation.required",
        "validation.suffix",
        "validation.url",
    ];

    const LOCALES: &[&str] = &["en", "fr-FR", "zh-CN"];

    #[test]
    fn every_key_resolves_in_every_locale() {
        for key in KEYS.iter().copied() {
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
    fn select_item_titles_resolve_per_locale() {
        assert_eq!(
            t!("preferred_language.english", locale = "en").as_ref(),
            "English"
        );
        assert_eq!(
            t!("enum_country.united_states", locale = "fr-FR").as_ref(),
            "États-Unis"
        );
        assert_eq!(t!("enum_country.china", locale = "zh-CN").as_ref(), "中国");
        assert_eq!(
            t!("user.username_label", locale = "zh-CN").as_ref(),
            "用户名"
        );
    }
}
