//! Consumer-facing prototyping utilities for gpui-form.
//!
//! Generated scaffolds localize labels and descriptions through
//! `::rust_i18n::t!`, which resolves against the CONSUMER crate's
//! `rust_i18n::i18n!` backend and `locales/` directory. The `locales/` shipped
//! here are the reference template for the keys emitted for the example
//! `User` shape; consumer forms declare their own copies per crate, since
//! `t!` never resolves across crates.

rust_i18n::i18n!("locales", fallback = "en");

pub mod code_gen;
pub mod error;
pub mod implementations;
pub mod imports;

pub use code_gen::{FormLayout, FormParts, FormShapeAdapter};
pub use error::{PrototypingError, PrototypingResult};

#[cfg(test)]
mod tests {
    use rust_i18n::t;

    /// Every key this crate ships, in field order for the `User` example
    /// shape the scaffolds document.
    const KEYS: &[&str] = &[
        "user.age_label",
        "user.age_description",
        "user.balance_label",
        "user.balance_description",
        "user.birth_date_label",
        "user.birth_date_description",
        "user.country_label",
        "user.country_description",
        "user.debt_label",
        "user.debt_description",
        "user.email_label",
        "user.email_description",
        "user.enable_notifications_label",
        "user.enable_notifications_description",
        "user.preferred_label",
        "user.preferred_description",
        "user.subscribe_newsletter_label",
        "user.subscribe_newsletter_description",
        "user.username_label",
        "user.username_description",
    ];

    const EN: &[(&str, &str)] = &[
        ("user.age_label", "Age"),
        ("user.age_description", "Age"),
        ("user.balance_label", "Balance"),
        ("user.balance_description", "Balance"),
        ("user.birth_date_label", "Birth Date"),
        ("user.birth_date_description", "Birth Date"),
        ("user.country_label", "Country"),
        ("user.country_description", "Country"),
        ("user.debt_label", "Debt"),
        ("user.debt_description", "Debt"),
        ("user.email_label", "Email"),
        ("user.email_description", "Email"),
        ("user.enable_notifications_label", "Enable Notifications"),
        (
            "user.enable_notifications_description",
            "Enable Notifications",
        ),
        ("user.preferred_label", "Preferred"),
        ("user.preferred_description", "Preferred"),
        ("user.subscribe_newsletter_label", "Subscribe Newsletter"),
        (
            "user.subscribe_newsletter_description",
            "Subscribe Newsletter",
        ),
        ("user.username_label", "Username"),
        ("user.username_description", "Username"),
    ];

    const FR_FR: &[(&str, &str)] = &[
        ("user.age_label", "Âge"),
        ("user.age_description", "Âge"),
        ("user.balance_label", "Solde"),
        ("user.balance_description", "Solde"),
        ("user.birth_date_label", "Date de naissance"),
        ("user.birth_date_description", "Date de naissance"),
        ("user.country_label", "Pays"),
        ("user.country_description", "Pays"),
        ("user.debt_label", "Dette"),
        ("user.debt_description", "Dette"),
        ("user.email_label", "Courriel"),
        ("user.email_description", "Courriel"),
        (
            "user.enable_notifications_label",
            "Activer les notifications",
        ),
        (
            "user.enable_notifications_description",
            "Activer les notifications",
        ),
        ("user.preferred_label", "Préféré"),
        ("user.preferred_description", "Préféré"),
        (
            "user.subscribe_newsletter_label",
            "S'abonner à la newsletter",
        ),
        (
            "user.subscribe_newsletter_description",
            "S'abonner à la newsletter",
        ),
        ("user.username_label", "Nom d'utilisateur"),
        ("user.username_description", "Nom d'utilisateur"),
    ];

    const ZH_CN: &[(&str, &str)] = &[
        ("user.age_label", "年龄"),
        ("user.age_description", "年龄"),
        ("user.balance_label", "余额"),
        ("user.balance_description", "余额"),
        ("user.birth_date_label", "出生日期"),
        ("user.birth_date_description", "出生日期"),
        ("user.country_label", "国家"),
        ("user.country_description", "国家"),
        ("user.debt_label", "债务"),
        ("user.debt_description", "债务"),
        ("user.email_label", "电子邮件"),
        ("user.email_description", "电子邮件"),
        ("user.enable_notifications_label", "启用通知"),
        ("user.enable_notifications_description", "启用通知"),
        ("user.preferred_label", "首选"),
        ("user.preferred_description", "首选"),
        ("user.subscribe_newsletter_label", "订阅简报"),
        ("user.subscribe_newsletter_description", "订阅简报"),
        ("user.username_label", "用户名"),
        ("user.username_description", "用户名"),
    ];

    fn table_for(locale: &str) -> &'static [(&'static str, &'static str)] {
        match locale {
            "en" => EN,
            "fr-FR" => FR_FR,
            _ => ZH_CN,
        }
    }

    #[test]
    fn every_key_resolves_in_every_locale() {
        for key in KEYS.iter().copied() {
            for locale in ["en", "fr-FR", "zh-CN"] {
                let translated = t!(key, locale = locale);
                assert_ne!(
                    &*translated, key,
                    "key {key} must resolve in locale {locale}"
                );
            }
        }
    }

    #[test]
    fn shipped_values_match_the_ported_fluent_strings() {
        for locale in ["en", "fr-FR", "zh-CN"] {
            for (key, expected) in table_for(locale).iter().copied() {
                assert_eq!(
                    &*t!(key, locale = locale),
                    expected,
                    "locale {locale} diverges from the ported Fluent string for {key}"
                );
            }
        }
    }

    #[test]
    fn key_inventory_matches_en_table() {
        let table_keys: Vec<&str> = EN.iter().map(|(key, _)| *key).collect();
        assert_eq!(table_keys, KEYS);
    }
}
