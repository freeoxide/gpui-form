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

use crate::i18n::{StoryText, localize};

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
        let items = child_items_for_level(&current_value, level, cx);
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
    _cx: &impl std::borrow::Borrow<App>,
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
    fn title(_: &gpui::App) -> String {
        "Infinite Select".into()
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
            .map(|value| value.summary(cx))
            .unwrap_or_else(|_| "None".to_string());
        let rebuilt_from_keys = build_from_key_path::<DeploymentTarget>(&key_path)
            .map(|value| value.summary(cx))
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
                    .child(format!(
                        "Current selection: {}",
                        snapshot.value().summary(cx)
                    ))
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

#[derive(Clone, Copy, Debug)]
pub(crate) enum DeploymentTargetLabelVariants {
    Web,
    Desktop,
    Docs,
}

impl StoryText for DeploymentTargetLabelVariants {
    fn key(&self) -> &'static str {
        match self {
            Self::Web => "infinite_select.deployment_target_label_variants_web",
            Self::Desktop => "infinite_select.deployment_target_label_variants_desktop",
            Self::Docs => "infinite_select.deployment_target_label_variants_docs",
        }
    }
}

#[derive(Clone, Debug, InfiniteSelect, PartialEq)]
#[fluent_kv(keys = ["description", "label"], keys_this)]
enum DeploymentTarget {
    Web(WebRegion),
    Desktop(DesktopPlatform),
    Docs,
}

impl DeploymentTarget {
    fn summary(&self, cx: &impl std::borrow::Borrow<App>) -> String {
        match self {
            Self::Web(region) => format!(
                "{} / {}",
                localize(cx, &DeploymentTargetLabelVariants::Web),
                region.summary(cx)
            ),
            Self::Desktop(platform) => format!(
                "{} / {}",
                localize(cx, &DeploymentTargetLabelVariants::Desktop),
                platform.name(cx)
            ),
            Self::Docs => localize(cx, &DeploymentTargetLabelVariants::Docs),
        }
    }
}

impl Default for DeploymentTarget {
    fn default() -> Self {
        Self::Web(WebRegion::default())
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum WebRegionLabelVariants {
    UsEast,
    Europe,
}

impl StoryText for WebRegionLabelVariants {
    fn key(&self) -> &'static str {
        match self {
            Self::UsEast => "infinite_select.web_region_label_variants_us_east",
            Self::Europe => "infinite_select.web_region_label_variants_europe",
        }
    }
}

#[derive(Clone, Debug, InfiniteSelect, PartialEq)]
#[fluent_kv(keys = ["description", "label"], keys_this)]
enum WebRegion {
    UsEast(AvailabilityZone),
    Europe(AvailabilityZone),
}

impl WebRegion {
    fn name(&self, cx: &impl std::borrow::Borrow<App>) -> String {
        match self {
            Self::UsEast(_) => localize(cx, &WebRegionLabelVariants::UsEast),
            Self::Europe(_) => localize(cx, &WebRegionLabelVariants::Europe),
        }
    }

    fn summary(&self, cx: &impl std::borrow::Borrow<App>) -> String {
        match self {
            Self::UsEast(zone) | Self::Europe(zone) => {
                format!("{} / {}", self.name(cx), zone.name(cx))
            },
        }
    }
}

impl Default for WebRegion {
    fn default() -> Self {
        Self::UsEast(AvailabilityZone::default())
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum AvailabilityZoneLabelVariants {
    Primary,
    DisasterRecovery,
}

impl StoryText for AvailabilityZoneLabelVariants {
    fn key(&self) -> &'static str {
        match self {
            Self::Primary => "infinite_select.availability_zone_label_variants_primary",
            Self::DisasterRecovery => {
                "infinite_select.availability_zone_label_variants_disaster_recovery"
            },
        }
    }
}

#[derive(Clone, Debug, Default, InfiniteSelect, PartialEq)]
#[fluent_kv(keys = ["description", "label"], keys_this)]
enum AvailabilityZone {
    #[default]
    Primary,
    DisasterRecovery,
}

impl AvailabilityZone {
    fn name(&self, cx: &impl std::borrow::Borrow<App>) -> String {
        match self {
            Self::Primary => localize(cx, &AvailabilityZoneLabelVariants::Primary),
            Self::DisasterRecovery => {
                localize(cx, &AvailabilityZoneLabelVariants::DisasterRecovery)
            },
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum DesktopPlatformLabelVariants {
    MacOs,
    Linux,
    Windows,
}

impl StoryText for DesktopPlatformLabelVariants {
    fn key(&self) -> &'static str {
        match self {
            Self::MacOs => "infinite_select.desktop_platform_label_variants_mac_os",
            Self::Linux => "infinite_select.desktop_platform_label_variants_linux",
            Self::Windows => "infinite_select.desktop_platform_label_variants_windows",
        }
    }
}

#[derive(Clone, Debug, Default, InfiniteSelect, PartialEq)]
#[fluent_kv(keys = ["description", "label"], keys_this)]
enum DesktopPlatform {
    #[default]
    MacOs,
    Linux,
    Windows,
}

impl DesktopPlatform {
    fn name(&self, cx: &impl std::borrow::Borrow<App>) -> String {
        match self {
            Self::MacOs => localize(cx, &DesktopPlatformLabelVariants::MacOs),
            Self::Linux => localize(cx, &DesktopPlatformLabelVariants::Linux),
            Self::Windows => localize(cx, &DesktopPlatformLabelVariants::Windows),
        }
    }
}
