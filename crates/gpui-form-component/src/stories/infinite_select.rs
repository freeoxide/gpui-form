use gpui::{
    App, AppContext as _, Context, Entity, Focusable, IntoElement, ParentElement as _, Render,
    SharedString, Styled as _, Subscription, Window, div,
};
use gpui_component::form::v_form;
use gpui_component::select::Select;

use crate::InfiniteSelect;
use crate::infinite_select::{InfiniteSelectEvent, InfiniteSelectState, build_from_path};

use super::common::{story_field, story_panel};

type DeploymentSelectState = InfiniteSelectState<DeploymentTarget>;

#[gpui_storybook::story]
pub struct InfiniteSelectStory {
    selection: DeploymentTarget,
    select_state: Entity<DeploymentSelectState>,
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
            selection,
            select_state,
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
        match event {
            InfiniteSelectEvent::Change(value) => {
                self.selection = value.clone();
            },
        }
    }

    fn child_label(&self, depth: usize) -> SharedString {
        self.selection
            .story_child_label_at_depth(depth)
            .unwrap_or_else(|| format!("Level {}", depth + 2).into())
    }

    fn child_description(&self, depth: usize) -> SharedString {
        self.selection
            .story_child_description_at_depth(depth)
            .unwrap_or_else(|| "Choose the next nested option.".into())
    }
}

impl Render for InfiniteSelectStory {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let select_state = self.select_state.read(cx);
        let master_select = select_state.master_select();
        let child_selects = select_state.child_selects();
        let path = select_state.path().clone();

        let form = child_selects.iter().enumerate().fold(
            v_form().child(story_field(
                self.selection.story_type_label(),
                self.selection.story_type_description(),
                Select::new(&master_select),
            )),
            |form, (depth, child)| {
                form.child(story_field(
                    self.child_label(depth),
                    self.child_description(depth),
                    Select::new(child),
                ))
            },
        );

        let rebuilt = build_from_path::<DeploymentTarget>(&path)
            .map(|value| value.summary())
            .unwrap_or_else(|| "None".to_string());

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
                    .child(format!("Current selection: {}", self.selection.summary()))
                    .child(format!("Path indices: {:?}", path.indices()))
                    .child(format!("Rebuilt from path: {rebuilt}")),
            ),
        )
    }
}

#[derive(Clone, Debug, InfiniteSelect)]
enum DeploymentTarget {
    Web(WebRegion),
    Desktop(DesktopPlatform),
    Docs,
}

impl DeploymentTarget {
    fn summary(&self) -> String {
        match self {
            Self::Web(region) => format!("Web / {}", region.summary()),
            Self::Desktop(platform) => format!("Desktop / {}", platform.name()),
            Self::Docs => "Docs".to_string(),
        }
    }

    fn story_type_label(&self) -> SharedString {
        "Target".into()
    }

    fn story_type_description(&self) -> SharedString {
        "Choose the surface that the generated form should target.".into()
    }

    fn story_child_label_at_depth(&self, depth: usize) -> Option<SharedString> {
        match depth {
            0 => match self {
                Self::Web(_) => Some("Region".into()),
                Self::Desktop(_) => Some("Platform".into()),
                Self::Docs => None,
            },
            1 => self.story_inner_child_label_at_depth(depth - 1),
            _ => None,
        }
    }

    fn story_child_description_at_depth(&self, depth: usize) -> Option<SharedString> {
        match depth {
            0 => match self {
                Self::Web(_) => Some("Pick a deployment region for the web surface.".into()),
                Self::Desktop(_) => Some("Pick the desktop platform to generate for.".into()),
                Self::Docs => None,
            },
            1 => self.story_inner_child_description_at_depth(depth - 1),
            _ => None,
        }
    }

    fn story_inner_child_label_at_depth(&self, depth: usize) -> Option<SharedString> {
        match (self, depth) {
            (Self::Web(_), 0) => Some("Availability zone".into()),
            _ => None,
        }
    }

    fn story_inner_child_description_at_depth(&self, depth: usize) -> Option<SharedString> {
        match (self, depth) {
            (Self::Web(_), 0) => Some("Select the fallback zone within the chosen region.".into()),
            _ => None,
        }
    }
}

impl Default for DeploymentTarget {
    fn default() -> Self {
        Self::Web(WebRegion::default())
    }
}

#[derive(Clone, Debug, InfiniteSelect)]
enum WebRegion {
    UsEast(AvailabilityZone),
    Europe(AvailabilityZone),
}

impl WebRegion {
    fn name(&self) -> &'static str {
        match self {
            Self::UsEast(_) => "US East",
            Self::Europe(_) => "Europe",
        }
    }

    fn summary(&self) -> String {
        match self {
            Self::UsEast(zone) | Self::Europe(zone) => {
                format!("{} / {}", self.name(), zone.name())
            },
        }
    }
}

impl Default for WebRegion {
    fn default() -> Self {
        Self::UsEast(AvailabilityZone::default())
    }
}

#[derive(Clone, Debug, Default, InfiniteSelect)]
enum AvailabilityZone {
    #[default]
    Primary,
    DisasterRecovery,
}

impl AvailabilityZone {
    fn name(&self) -> &'static str {
        match self {
            Self::Primary => "Primary",
            Self::DisasterRecovery => "Disaster recovery",
        }
    }
}

#[derive(Clone, Debug, Default, InfiniteSelect)]
enum DesktopPlatform {
    #[default]
    MacOs,
    Linux,
    Windows,
}

impl DesktopPlatform {
    fn name(&self) -> &'static str {
        match self {
            Self::MacOs => "macOS",
            Self::Linux => "Linux",
            Self::Windows => "Windows",
        }
    }
}
