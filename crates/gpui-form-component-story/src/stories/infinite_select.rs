use es_fluent::{EsFluentThis, EsFluentVariants, ToFluentString as _};
use gpui::{
    App, AppContext as _, Context, Entity, Focusable, IntoElement, ParentElement as _, Render,
    Styled as _, Subscription, Window, div,
};
use gpui_component::IndexPath;
use gpui_component::form::v_form;

use gpui_form_component::InfiniteSelect;
use gpui_form_component::infinite_select::{
    InfiniteSelect as _, InfiniteSelectEvent, InfiniteSelectItem, InfiniteSelectState,
    build_from_key_path, build_from_path, to_select_items,
};

use super::common::story_panel;

type DeploymentSelectState = InfiniteSelectState<DeploymentTarget>;

fn selected_index(row: usize) -> Option<IndexPath> {
    Some(IndexPath {
        section: 0,
        row,
        column: 0,
    })
}

fn refresh_select_labels(
    state: &Entity<DeploymentSelectState>,
    window: &mut Window,
    cx: &mut Context<InfiniteSelectStory>,
) {
    let (value, path, master_select, child_selects) = {
        let state = state.read(cx);
        (
            state.value().clone(),
            state.path().clone(),
            state.master_select(),
            state.child_selects(),
        )
    };

    master_select.update(cx, |select, cx| {
        select.set_items(to_select_items::<DeploymentTarget>(), window, cx);
        select.set_selected_index(path.get(0).and_then(selected_index), window, cx);
    });

    let mut current_value = value;
    for (level, child_select) in child_selects.into_iter().enumerate() {
        let items = child_items_for_level(&current_value, level);
        if items.is_empty() {
            break;
        }

        let selected_row = path.get(level + 1).unwrap_or(0).min(items.len() - 1);
        child_select.update(cx, |select, cx| {
            select.set_items(items.clone(), window, cx);
            select.set_selected_index(selected_index(selected_row), window, cx);
        });
        current_value = items[selected_row].get_value().clone();
    }
}

fn child_items_for_level(
    current_value: &DeploymentTarget,
    level: usize,
) -> Vec<InfiniteSelectItem<DeploymentTarget>> {
    let (has_more, child_labels) = if level == 0 {
        (
            current_value.has_inner(),
            current_value.child_variant_labels(),
        )
    } else {
        (
            current_value.inner_has_inner(),
            current_value.inner_child_variant_labels(),
        )
    };

    if !has_more || child_labels.is_empty() {
        return Vec::new();
    }

    child_labels
        .into_iter()
        .enumerate()
        .filter_map(|(index, title)| {
            let value = if level == 0 {
                current_value.set_child_by_index(index)
            } else {
                current_value.inner_set_child_by_index(index)
            };
            value.map(|value| InfiniteSelectItem::new(value, title))
        })
        .collect()
}

#[gpui_storybook::story]
pub struct InfiniteSelectStory {
    select_state: Entity<DeploymentSelectState>,
    last_changed_depth: Option<usize>,
    last_previous_key_path: Option<String>,
    _subscription: Subscription,
}

impl gpui_storybook::Story for InfiniteSelectStory {
    fn title() -> String {
        "Infinite Select".into()
    }

    fn description() -> String {
        "Cascading select demo backed by the runtime infinite-select state helper.".into()
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        cx.new(|cx| Self::new(window, cx))
    }
}

impl Focusable for InfiniteSelectStory {
    fn focus_handle(&self, cx: &App) -> gpui::FocusHandle {
        self.select_state.focus_handle(cx)
    }
}

impl InfiniteSelectStory {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let selection = DeploymentTarget::default();
        let select_state = cx.new(|cx| InfiniteSelectState::new(selection.clone(), window, cx));
        let subscription = cx.subscribe_in(&select_state, window, Self::on_select_change);

        Self {
            select_state,
            last_changed_depth: None,
            last_previous_key_path: None,
            _subscription: subscription,
        }
    }

    fn on_select_change(
        &mut self,
        _this: &Entity<DeploymentSelectState>,
        event: &InfiniteSelectEvent<DeploymentTarget>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        self.last_changed_depth = Some(event.changed_depth());
        self.last_previous_key_path = Some(event.previous_key_path().to_string());
    }
}

impl Render for InfiniteSelectStory {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        refresh_select_labels(&self.select_state, window, cx);

        let snapshot = self.select_state.read(cx).snapshot();
        let path = snapshot.path().clone();
        let key_path = snapshot.key_path().clone();

        let form = snapshot
            .form_fields()
            .into_iter()
            .fold(v_form(), |form, field| form.child(field));

        let rebuilt = build_from_path::<DeploymentTarget>(&path)
            .map(|value| value.summary())
            .unwrap_or_else(|_| "None".to_string());
        let rebuilt_from_keys = build_from_key_path::<DeploymentTarget>(&key_path)
            .map(|value| value.summary())
            .unwrap_or_else(|_| "None".to_string());

        story_panel(
            "Cascading selection",
            "This mirrors the runtime helper flow used by generated forms: one state entity owns the master select, derived child selects, and the selection path.",
            div().flex().flex_col().gap_4().child(form).child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .mt_2()
                    .text_sm()
                    .child(format!("Current selection: {}", snapshot.value().summary()))
                    .child(format!("Path indices: {:?}", path.indices()))
                    .child(format!("Path keys: {:?}", key_path.keys()))
                    .child(format!("Rebuilt from path: {rebuilt}"))
                    .child(format!("Rebuilt from keys: {rebuilt_from_keys}"))
                    .child(format!(
                        "Previous key path: {}",
                        self.last_previous_key_path
                            .clone()
                            .unwrap_or_else(|| "None".to_string())
                    ))
                    .child(format!(
                        "Last changed depth: {}",
                        self.last_changed_depth
                            .map(|depth| depth.to_string())
                            .unwrap_or_else(|| "None".to_string())
                    )),
            ),
        )
    }
}

#[derive(Clone, Debug, EsFluentThis, EsFluentVariants, InfiniteSelect)]
#[fluent(namespace = "infinite_select")]
#[fluent_this(origin, variants)]
#[fluent_variants(keys = ["description", "label"])]
#[fluent_kv(keys = ["description", "label"], keys_this)]
enum DeploymentTarget {
    Web(WebRegion),
    Desktop(DesktopPlatform),
    Docs,
}

impl DeploymentTarget {
    fn summary(&self) -> String {
        match self {
            Self::Web(region) => format!(
                "{} / {}",
                DeploymentTargetLabelVariants::Web.to_fluent_string(),
                region.summary()
            ),
            Self::Desktop(platform) => format!(
                "{} / {}",
                DeploymentTargetLabelVariants::Desktop.to_fluent_string(),
                platform.name()
            ),
            Self::Docs => DeploymentTargetLabelVariants::Docs.to_fluent_string(),
        }
    }
}

impl Default for DeploymentTarget {
    fn default() -> Self {
        Self::Web(WebRegion::default())
    }
}

#[derive(Clone, Debug, EsFluentThis, EsFluentVariants, InfiniteSelect)]
#[fluent(namespace = "infinite_select")]
#[fluent_this(origin, variants)]
#[fluent_variants(keys = ["description", "label"])]
#[fluent_kv(keys = ["description", "label"], keys_this)]
enum WebRegion {
    UsEast(AvailabilityZone),
    Europe(AvailabilityZone),
}

impl WebRegion {
    fn name(&self) -> String {
        match self {
            Self::UsEast(_) => WebRegionLabelVariants::UsEast.to_fluent_string(),
            Self::Europe(_) => WebRegionLabelVariants::Europe.to_fluent_string(),
        }
    }

    fn summary(&self) -> String {
        match self {
            Self::UsEast(zone) | Self::Europe(zone) => format!("{} / {}", self.name(), zone.name()),
        }
    }
}

impl Default for WebRegion {
    fn default() -> Self {
        Self::UsEast(AvailabilityZone::default())
    }
}

#[derive(Clone, Debug, Default, EsFluentThis, EsFluentVariants, InfiniteSelect)]
#[fluent(namespace = "infinite_select")]
#[fluent_this(origin, variants)]
#[fluent_variants(keys = ["description", "label"])]
#[fluent_kv(keys = ["description", "label"], keys_this)]
enum AvailabilityZone {
    #[default]
    Primary,
    DisasterRecovery,
}

impl AvailabilityZone {
    fn name(&self) -> String {
        match self {
            Self::Primary => AvailabilityZoneLabelVariants::Primary.to_fluent_string(),
            Self::DisasterRecovery => {
                AvailabilityZoneLabelVariants::DisasterRecovery.to_fluent_string()
            },
        }
    }
}

#[derive(Clone, Debug, Default, EsFluentThis, EsFluentVariants, InfiniteSelect)]
#[fluent(namespace = "infinite_select")]
#[fluent_this(origin, variants)]
#[fluent_variants(keys = ["description", "label"])]
#[fluent_kv(keys = ["description", "label"], keys_this)]
enum DesktopPlatform {
    #[default]
    MacOs,
    Linux,
    Windows,
}

impl DesktopPlatform {
    fn name(&self) -> String {
        match self {
            Self::MacOs => DesktopPlatformLabelVariants::MacOs.to_fluent_string(),
            Self::Linux => DesktopPlatformLabelVariants::Linux.to_fluent_string(),
            Self::Windows => DesktopPlatformLabelVariants::Windows.to_fluent_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use es_fluent::ToFluentString as _;

    use crate::i18n::{DatePickerComponentText, FilePickerComponentText};

    use super::{DeploymentTargetLabelVariants, WebRegionLabelVariants};

    #[test]
    fn resolves_infinite_select_demo_metadata() {
        es_fluent_manager_embedded::init_with_language(unic_langid::langid!("en"));

        assert_eq!(DeploymentTargetLabelVariants::Web.to_fluent_string(), "Web");
        assert_eq!(WebRegionLabelVariants::UsEast.to_fluent_string(), "US East");
        assert_eq!(
            FilePickerComponentText::SourcePlaceholder.to_fluent_string(),
            "Choose a source file"
        );
        assert_eq!(
            DatePickerComponentText::LaunchPlaceholder.to_fluent_string(),
            "Select a launch date"
        );

        es_fluent_manager_embedded::select_language(unic_langid::langid!("fr-FR")).unwrap();
        assert_eq!(
            DeploymentTargetLabelVariants::Desktop.to_fluent_string(),
            "Bureau"
        );
        assert_eq!(
            WebRegionLabelVariants::UsEast.to_fluent_string(),
            "Est des États-Unis"
        );
        assert_eq!(
            FilePickerComponentText::SourcePlaceholder.to_fluent_string(),
            "Sélectionner un fichier source"
        );
        assert_eq!(
            DatePickerComponentText::LaunchPlaceholder.to_fluent_string(),
            "Sélectionner une date de lancement"
        );

        es_fluent_manager_embedded::select_language(unic_langid::langid!("zh-CN")).unwrap();
        assert_eq!(
            DeploymentTargetLabelVariants::Docs.to_fluent_string(),
            "文档"
        );
        assert_eq!(
            WebRegionLabelVariants::UsEast.to_fluent_string(),
            "美国东部"
        );
        assert_eq!(
            FilePickerComponentText::SourcePlaceholder.to_fluent_string(),
            "选择源文件"
        );
        assert_eq!(
            DatePickerComponentText::LaunchPlaceholder.to_fluent_string(),
            "选择发布日期"
        );
    }
}
