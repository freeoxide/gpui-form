use std::borrow::Borrow;

use gpui::App;
use rust_i18n::t;

pub use gpui_form_i18n::{
    CurrentLanguage, I18n, Language, LocalizationError, change_locale, fallback_label,
    fallback_message, humanize_key, init, init_with_language, locale, localize_label,
    localize_message, replace_with_language, try_localize_message,
};

pub(crate) trait StoryText {
    fn key(&self) -> &'static str;
}

pub(crate) fn localize(cx: &impl Borrow<App>, text: &impl StoryText) -> String {
    localize_message(cx, text.key())
}

#[derive(Clone, Debug)]
pub(crate) enum DatePickerComponentText {
    LaunchPlaceholder,
}

impl StoryText for DatePickerComponentText {
    fn key(&self) -> &'static str {
        match self {
            Self::LaunchPlaceholder => "date_picker.launch_placeholder",
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum FilePickerComponentText {
    SourcePlaceholder,
    OutputPlaceholder,
    ChooseFiles,
}

impl StoryText for FilePickerComponentText {
    fn key(&self) -> &'static str {
        match self {
            Self::SourcePlaceholder => "file_picker.source_placeholder",
            Self::OutputPlaceholder => "file_picker.output_placeholder",
            Self::ChooseFiles => "file_picker.choose_files",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StoryText;

    const KEYS: &[&str] = &[
        "date_picker.launch_placeholder",
        "file_picker.choose_files",
        "file_picker.output_placeholder",
        "file_picker.source_placeholder",
        "infinite_select.availability_zone_description_variants_disaster_recovery",
        "infinite_select.availability_zone_description_variants_label",
        "infinite_select.availability_zone_description_variants_primary",
        "infinite_select.availability_zone_label",
        "infinite_select.availability_zone_label_variants_disaster_recovery",
        "infinite_select.availability_zone_label_variants_label",
        "infinite_select.availability_zone_label_variants_primary",
        "infinite_select.deployment_target_description_variants_desktop",
        "infinite_select.deployment_target_description_variants_docs",
        "infinite_select.deployment_target_description_variants_label",
        "infinite_select.deployment_target_description_variants_web",
        "infinite_select.deployment_target_label",
        "infinite_select.deployment_target_label_variants_desktop",
        "infinite_select.deployment_target_label_variants_docs",
        "infinite_select.deployment_target_label_variants_label",
        "infinite_select.deployment_target_label_variants_web",
        "infinite_select.desktop_platform_description_variants_label",
        "infinite_select.desktop_platform_description_variants_linux",
        "infinite_select.desktop_platform_description_variants_mac_os",
        "infinite_select.desktop_platform_description_variants_windows",
        "infinite_select.desktop_platform_label",
        "infinite_select.desktop_platform_label_variants_label",
        "infinite_select.desktop_platform_label_variants_linux",
        "infinite_select.desktop_platform_label_variants_mac_os",
        "infinite_select.desktop_platform_label_variants_windows",
        "infinite_select.web_region_description_variants_europe",
        "infinite_select.web_region_description_variants_label",
        "infinite_select.web_region_description_variants_us_east",
        "infinite_select.web_region_label",
        "infinite_select.web_region_label_variants_europe",
        "infinite_select.web_region_label_variants_label",
        "infinite_select.web_region_label_variants_us_east",
    ];

    const LOCALES: &[&str] = &["en", "fr-FR", "zh-CN"];

    fn localized(locale: &str, text: &impl StoryText) -> String {
        t!(text.key(), locale = locale).to_string()
    }

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
    fn en_locale_file_matches_key_inventory() {
        let yml = include_str!("../locales/en.yml");
        let declared = yml
            .lines()
            .filter(|line| line.starts_with("  ") && line.contains(": "))
            .count();
        assert_eq!(declared, KEYS.len());
        for key in KEYS {
            let namespace = key.split('.').next().unwrap();
            let leaf = key.split('.').next_back().unwrap();
            assert!(
                yml.contains(&format!("{namespace}:")) && yml.contains(&format!("  {leaf}:")),
                "missing key {key}"
            );
        }
    }

    #[test]
    fn resolves_demo_strings_in_every_locale() {
        use super::{DatePickerComponentText, FilePickerComponentText};

        assert_eq!(
            localized("en", &DatePickerComponentText::LaunchPlaceholder),
            "Select a launch date"
        );
        assert_eq!(
            localized("en", &FilePickerComponentText::SourcePlaceholder),
            "Choose a source file"
        );
        assert_eq!(
            localized("fr-FR", &DatePickerComponentText::LaunchPlaceholder),
            "Sélectionner une date de lancement"
        );
        assert_eq!(
            localized("fr-FR", &FilePickerComponentText::SourcePlaceholder),
            "Sélectionner un fichier source"
        );
        assert_eq!(
            localized("zh-CN", &DatePickerComponentText::LaunchPlaceholder),
            "选择发布日期"
        );
        assert_eq!(
            localized("zh-CN", &FilePickerComponentText::SourcePlaceholder),
            "选择源文件"
        );
    }

    #[test]
    fn resolves_infinite_select_labels_in_every_locale() {
        use super::super::stories::infinite_select::{
            AvailabilityZoneLabelVariants, DeploymentTargetLabelVariants,
            DesktopPlatformLabelVariants, WebRegionLabelVariants,
        };

        assert_eq!(localized("en", &DeploymentTargetLabelVariants::Web), "Web");
        assert_eq!(localized("en", &WebRegionLabelVariants::UsEast), "US East");
        assert_eq!(
            localized("en", &AvailabilityZoneLabelVariants::Primary),
            "Primary"
        );
        assert_eq!(
            localized("en", &DesktopPlatformLabelVariants::MacOs),
            "macOS"
        );
        assert_eq!(
            localized("fr-FR", &DeploymentTargetLabelVariants::Desktop),
            "Bureau"
        );
        assert_eq!(
            localized("fr-FR", &WebRegionLabelVariants::UsEast),
            "Est des États-Unis"
        );
        assert_eq!(
            localized("fr-FR", &AvailabilityZoneLabelVariants::DisasterRecovery),
            "Reprise après sinistre"
        );
        assert_eq!(
            localized("fr-FR", &DesktopPlatformLabelVariants::Windows),
            "Windows"
        );
        assert_eq!(
            localized("zh-CN", &DeploymentTargetLabelVariants::Docs),
            "文档"
        );
        assert_eq!(
            localized("zh-CN", &WebRegionLabelVariants::UsEast),
            "美国东部"
        );
        assert_eq!(
            localized("zh-CN", &AvailabilityZoneLabelVariants::DisasterRecovery),
            "灾难恢复"
        );
        assert_eq!(
            localized("zh-CN", &DesktopPlatformLabelVariants::MacOs),
            "macOS"
        );
    }
}
