use es_fluent::{EsFluentThis, EsFluentVariants, ToFluentString as _};
use gpui::{
    App, AppContext as _, Context, Entity, Focusable, IntoElement, ParentElement as _, Render,
    Styled as _, Subscription, Window, div,
};
use gpui_component::form::v_form;

use crate::i18n::InfiniteSelectStoryText;
use gpui_form_component::InfiniteSelect;
use gpui_form_component::infinite_select::{
    InfiniteSelectEvent, InfiniteSelectState, build_from_key_path, build_from_path,
};

use super::common::story_panel;

type DeploymentSelectState = InfiniteSelectState<DeploymentTarget>;

#[gpui_storybook::story]
pub struct InfiniteSelectStory {
    select_state: Entity<DeploymentSelectState>,
    last_changed_depth: Option<usize>,
    last_previous_key_path: Option<String>,
    _subscription: Subscription,
}

impl gpui_storybook::Story for InfiniteSelectStory {
    fn title() -> String {
        InfiniteSelectStoryText::Title.to_fluent_string()
    }

    fn description() -> String {
        InfiniteSelectStoryText::Description.to_fluent_string()
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
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let snapshot = self.select_state.read(cx).snapshot();
        let path = snapshot.path().clone();
        let key_path = snapshot.key_path().clone();

        let form = snapshot
            .form_fields()
            .into_iter()
            .fold(v_form(), |form, field| form.child(field));

        let rebuilt = build_from_path::<DeploymentTarget>(&path)
            .map(|value| value.summary())
            .unwrap_or_else(|_| InfiniteSelectStoryText::None.to_fluent_string());
        let rebuilt_from_keys = build_from_key_path::<DeploymentTarget>(&key_path)
            .map(|value| value.summary())
            .unwrap_or_else(|_| InfiniteSelectStoryText::None.to_fluent_string());

        story_panel(
            InfiniteSelectStoryText::PanelTitle.to_fluent_string(),
            InfiniteSelectStoryText::PanelDescription.to_fluent_string(),
            div().flex().flex_col().gap_4().child(form).child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .mt_2()
                    .text_sm()
                    .child(
                        InfiniteSelectStoryText::CurrentSelection {
                            value: snapshot.value().summary(),
                        }
                        .to_fluent_string(),
                    )
                    .child(
                        InfiniteSelectStoryText::PathIndices {
                            value: format!("{:?}", path.indices()),
                        }
                        .to_fluent_string(),
                    )
                    .child(
                        InfiniteSelectStoryText::PathKeys {
                            value: format!("{:?}", key_path.keys()),
                        }
                        .to_fluent_string(),
                    )
                    .child(
                        InfiniteSelectStoryText::RebuiltFromPath { value: rebuilt }
                            .to_fluent_string(),
                    )
                    .child(
                        InfiniteSelectStoryText::RebuiltFromKeys {
                            value: rebuilt_from_keys,
                        }
                        .to_fluent_string(),
                    )
                    .child(
                        InfiniteSelectStoryText::PreviousKeyPath {
                            value: self.last_previous_key_path.clone().unwrap_or_else(|| {
                                InfiniteSelectStoryText::None.to_fluent_string()
                            }),
                        }
                        .to_fluent_string(),
                    )
                    .child(
                        InfiniteSelectStoryText::LastChangedDepth {
                            value: self
                                .last_changed_depth
                                .map(|depth| depth.to_string())
                                .unwrap_or_else(|| {
                                    InfiniteSelectStoryText::None.to_fluent_string()
                                }),
                        }
                        .to_fluent_string(),
                    ),
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
            Self::Web(region) => InfiniteSelectStoryText::DeploymentWeb {
                region: region.summary(),
            }
            .to_fluent_string(),
            Self::Desktop(platform) => InfiniteSelectStoryText::DeploymentDesktop {
                platform: platform.name(),
            }
            .to_fluent_string(),
            Self::Docs => InfiniteSelectStoryText::DeploymentDocs.to_fluent_string(),
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
            Self::UsEast(zone) | Self::Europe(zone) => InfiniteSelectStoryText::WebRegion {
                region: self.name(),
                zone: zone.name(),
            }
            .to_fluent_string(),
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
