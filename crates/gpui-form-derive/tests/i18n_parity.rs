rust_i18n::i18n!("locales", fallback = "en");

const LOCALES: &[&str] = &["en", "fr-FR", "zh-CN"];

const KEYS: &[&str] = &[
    "validation.required",
    "validation.non_empty",
    "validation.len",
    "validation.credit_card",
    "validation.email",
    "validation.ip",
    "validation.phone_number",
    "validation.url",
    "validation.negative",
    "validation.non_negative",
    "validation.non_positive",
    "validation.positive",
    "validation.range",
    "validation.alphanumeric",
    "validation.ascii",
    "validation.contains",
    "validation.matches",
    "validation.pattern",
    "validation.prefix",
    "validation.suffix",
    "validation.invalid",
];

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
fn required_message_is_ported_from_koruma_collection_ftl() {
    assert_eq!(
        &*rust_i18n::t!("validation.required", locale = "en"),
        "This field is required."
    );
    assert_eq!(
        &*rust_i18n::t!("validation.required", locale = "fr-FR"),
        "Ce champ est obligatoire."
    );
    assert_eq!(
        &*rust_i18n::t!("validation.required", locale = "zh-CN"),
        "此字段为必填。"
    );
}

#[test]
fn email_message_is_ported_from_koruma_collection_ftl() {
    assert_eq!(
        &*rust_i18n::t!("validation.email", locale = "en"),
        "Not a valid email address."
    );
    assert_eq!(
        &*rust_i18n::t!("validation.email", locale = "fr-FR"),
        "N'est pas une adresse e-mail valide."
    );
    assert_eq!(
        &*rust_i18n::t!("validation.email", locale = "zh-CN"),
        "不是有效的电子邮件地址。"
    );
}

#[test]
fn len_message_interpolates_bounds_in_every_locale() {
    for locale in LOCALES.iter().copied() {
        let translated = rust_i18n::t!("validation.len", min = 3, max = 8, locale = locale);
        let rendered = translated.to_string();
        assert!(
            rendered.contains('3') && rendered.contains('8'),
            "len message for {locale} must interpolate bounds: {rendered}"
        );
        assert!(
            !rendered.contains("%{min}") && !rendered.contains("%{max}"),
            "len message for {locale} must not leak placeholders: {rendered}"
        );
    }
}
