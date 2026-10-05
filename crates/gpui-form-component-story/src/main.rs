use anyhow::anyhow;
use gpui::BorrowAppContext as _;
use unic_langid::LanguageIdentifier;

use gpui_storybook::{Assets, Gallery};
use gpui_storybook_core::locale::LocaleStore;

// Library target must be linked or inventory story registrations are dropped.
#[allow(unused_imports, clippy::single_component_path_imports)]
use gpui_form_component_story;

const DEFAULT_LOCALE: &str = "en";

const AVAILABLE_LOCALES: &[(&str, &str)] = &[
    ("English", "en"),
    ("Français", "fr-FR"),
    ("简体中文", "zh-CN"),
];

struct ComponentLocaleStore;

impl ComponentLocaleStore {
    fn new() -> Self {
        Self
    }
}

impl LocaleStore for ComponentLocaleStore {
    fn available_locales(
        &self,
        _cx: &gpui::App,
    ) -> anyhow::Result<Vec<(String, LanguageIdentifier)>> {
        AVAILABLE_LOCALES
            .iter()
            .map(|(label, id)| {
                let locale = id
                    .parse()
                    .map_err(|err| anyhow!("invalid locale '{id}': {err}"))?;
                Ok(((*label).to_string(), locale))
            })
            .collect()
    }

    fn current_locale(&self, cx: &gpui::App) -> anyhow::Result<LanguageIdentifier> {
        let locale = gpui_form_component_story::i18n::locale(cx);
        locale
            .parse()
            .map_err(|err| anyhow!("invalid current locale '{locale}': {err}"))
    }

    fn set_current_locale(
        &self,
        locale: LanguageIdentifier,
        cx: &mut gpui::App,
    ) -> anyhow::Result<()> {
        gpui_form_component_story::i18n::change_locale(cx, locale.clone()).map_err(|err| {
            anyhow::anyhow!(
                "failed to sync gpui-form-component-story locale to '{}': {err}",
                locale
            )
        })?;
        Ok(())
    }
}

fn main() {
    let app = gpui_platform::application().with_assets(Assets);
    let name_arg = std::env::args().nth(1);

    app.run(move |app_cx| {
        gpui_component::init(app_cx);
        gpui_form_component_story::i18n::init(app_cx);
        gpui_storybook_core::story::init(app_cx);
        app_cx.set_global(Box::new(ComponentLocaleStore::new()) as Box<dyn LocaleStore>);
        app_cx
            .update_global::<Box<dyn LocaleStore>, _>(|locale_store, cx| {
                locale_store.set_current_locale(DEFAULT_LOCALE.parse().unwrap(), cx)
            })
            .unwrap();
        app_cx.activate(true);

        gpui_storybook::create_new_window(
            &format!("{} - Stories", env!("CARGO_PKG_NAME")),
            move |window, cx| {
                let all_stories = gpui_storybook::generate_stories(window, cx);

                Gallery::view(all_stories, name_arg.as_deref(), window, cx)
            },
            app_cx,
        );
    });
}
