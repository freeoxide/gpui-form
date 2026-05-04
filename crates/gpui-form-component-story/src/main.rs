use es_fluent::{EsFluent, unic_langid::LanguageIdentifier};
use es_fluent_lang::es_fluent_language;
use gpui::BorrowAppContext as _;
use gpui_storybook::{Assets, Gallery};
use gpui_storybook_core::{
    language::Language,
    locale::{LocaleManager, LocaleStore},
};
use strum::EnumIter;

// Bring the library target into scope so story inventory registrations are linked.
#[allow(unused_imports, clippy::single_component_path_imports)]
use gpui_form_component_story;

#[es_fluent_language]
#[derive(Clone, Copy, Debug, EnumIter, Eq, EsFluent, PartialEq)]
pub enum Languages {}

struct ComponentLocaleStore<L: Language> {
    inner: LocaleManager<L>,
}

impl<L: Language> ComponentLocaleStore<L> {
    fn new() -> Self {
        Self {
            inner: LocaleManager::new(),
        }
    }
}

impl<L: Language> LocaleStore for ComponentLocaleStore<L> {
    fn available_locales(&self) -> anyhow::Result<Vec<(String, LanguageIdentifier)>> {
        self.inner.available_locales()
    }

    fn current_locale(&self, cx: &gpui::App) -> anyhow::Result<LanguageIdentifier> {
        self.inner.current_locale(cx)
    }

    fn set_current_locale(
        &self,
        locale: LanguageIdentifier,
        cx: &mut gpui::App,
    ) -> anyhow::Result<()> {
        self.inner.set_current_locale(locale.clone(), cx)?;
        gpui_form_component_story::i18n::change_locale(locale.clone()).map_err(|err| {
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
        gpui_form_component_story::i18n::init();
        gpui_storybook::init(Languages::default(), app_cx);
        app_cx
            .set_global(Box::new(ComponentLocaleStore::<Languages>::new()) as Box<dyn LocaleStore>);
        app_cx
            .update_global::<Box<dyn LocaleStore>, _>(|locale_store, cx| {
                locale_store.set_current_locale(Languages::default().into(), cx)
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
