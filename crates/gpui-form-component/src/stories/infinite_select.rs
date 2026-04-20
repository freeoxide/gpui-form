use gpui::{
    App, AppContext as _, Context, Entity, Focusable, IntoElement, ParentElement as _, Render,
    SharedString, Styled as _, Subscription, Window, div,
};
use gpui_component::{
    IndexPath,
    form::v_form,
    select::{Select, SelectEvent, SelectState},
};

use crate::InfiniteSelect;
use crate::infinite_select::{
    InfiniteSelect as _, InfiniteSelectItem, InfiniteSelectPath, build_from_path, to_select_items,
};

use super::common::{story_field, story_panel};

type DeploymentSelectState = SelectState<Vec<InfiniteSelectItem<DeploymentTarget>>>;

#[gpui_storybook::story]
pub struct InfiniteSelectStory {
    selection: DeploymentTarget,
    path: InfiniteSelectPath,
    master_select: Entity<DeploymentSelectState>,
    child_selects: Vec<Entity<DeploymentSelectState>>,
    _master_subscription: Subscription,
    _child_subscriptions: Vec<Subscription>,
}

impl gpui_storybook::Story for InfiniteSelectStory {
    fn title() -> String {
        "Infinite Select".into()
    }

    fn description() -> String {
        "Cascading select demo backed by the runtime infinite-select trait and helper types.".into()
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        cx.new(|cx| Self::new(window, cx))
    }
}

impl Focusable for InfiniteSelectStory {
    fn focus_handle(&self, cx: &App) -> gpui::FocusHandle {
        self.master_select.read(cx).focus_handle(cx)
    }
}

impl InfiniteSelectStory {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let selection = DeploymentTarget::default();
        let master_select = cx.new(|cx| {
            SelectState::new(
                to_select_items::<DeploymentTarget>(),
                selected_row(0),
                window,
                cx,
            )
        });
        let child_selects = Self::build_child_selects(&selection, 0, window, cx);

        Self {
            selection,
            path: path_with_defaults(0, child_selects.len()),
            _master_subscription: cx.subscribe_in(&master_select, window, Self::on_master_select),
            _child_subscriptions: Self::subscribe_child_selects(&child_selects, window, cx),
            master_select,
            child_selects,
        }
    }

    fn build_child_selects(
        parent: &DeploymentTarget,
        start_level: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<Entity<DeploymentSelectState>> {
        let mut current_value = parent.clone();
        let mut selects = Vec::new();

        for level in start_level..(DeploymentTarget::depth() - 1) {
            let child_count = if level == 0 {
                current_value.child_variant_names().len()
            } else {
                current_value.inner_child_variant_names().len()
            };

            if child_count == 0 {
                break;
            }

            let items: Vec<InfiniteSelectItem<DeploymentTarget>> = (0..child_count)
                .filter_map(|index| {
                    let variant = if level == 0 {
                        current_value.set_child_by_index(index)
                    } else {
                        current_value.inner_set_child_by_index(index)
                    };
                    variant.map(|value| {
                        let title = value.story_option_title(level);
                        InfiniteSelectItem::new(value, title)
                    })
                })
                .collect();

            if items.is_empty() {
                break;
            }

            let child_select =
                cx.new(|cx| SelectState::new(items.clone(), selected_row(0), window, cx));
            selects.push(child_select);

            if let Some(first_item) = items.first() {
                current_value = first_item.get_value().clone();
            } else {
                break;
            }
        }

        selects
    }

    fn subscribe_child_selects(
        child_selects: &[Entity<DeploymentSelectState>],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<Subscription> {
        child_selects
            .iter()
            .map(|child| cx.subscribe_in(child, window, Self::on_child_select))
            .collect()
    }

    fn on_master_select(
        &mut self,
        this: &Entity<DeploymentSelectState>,
        event: &SelectEvent<Vec<InfiniteSelectItem<DeploymentTarget>>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let SelectEvent::Confirm(Some(selected)) = event else {
            return;
        };

        let root_index = this
            .read(cx)
            .selected_index(cx)
            .map_or(0, |index| index.row);
        self.selection = selected.clone();
        self.child_selects = Self::build_child_selects(&self.selection, 0, window, cx);
        self._child_subscriptions = Self::subscribe_child_selects(&self.child_selects, window, cx);
        self.path = path_with_defaults(root_index, self.child_selects.len());
        cx.notify();
    }

    fn on_child_select(
        &mut self,
        this: &Entity<DeploymentSelectState>,
        event: &SelectEvent<Vec<InfiniteSelectItem<DeploymentTarget>>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let SelectEvent::Confirm(Some(selected)) = event else {
            return;
        };

        let Some(level) = self
            .child_selects
            .iter()
            .position(|child| child == this)
            .map(|position| position + 1)
        else {
            return;
        };

        let selected_index = this
            .read(cx)
            .selected_index(cx)
            .map_or(0, |index| index.row);

        self.selection = selected.clone();
        self.child_selects.truncate(level);

        let new_children = Self::build_child_selects(&self.selection, level, window, cx);
        self.child_selects.extend(new_children);
        self._child_subscriptions = Self::subscribe_child_selects(&self.child_selects, window, cx);

        self.path.set(level, selected_index);
        self.path.truncate(level + 1);
        for default_level in 0..(self.child_selects.len().saturating_sub(level)) {
            self.path.set(level + default_level + 1, 0);
        }

        cx.notify();
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
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let form = self.child_selects.iter().enumerate().fold(
            v_form().child(story_field(
                self.selection.story_type_label(),
                self.selection.story_type_description(),
                Select::new(&self.master_select),
            )),
            |form, (depth, child)| {
                form.child(story_field(
                    self.child_label(depth),
                    self.child_description(depth),
                    Select::new(child),
                ))
            },
        );

        let rebuilt = build_from_path::<DeploymentTarget>(&self.path)
            .map(|value| value.summary())
            .unwrap_or_else(|| "None".to_string());

        story_panel(
            "Cascading selection",
            "This mirrors the runtime helper flow used by generated forms: a master select, derived child selects, and an index path that can rebuild the nested value.",
            div().flex().flex_col().gap_4().child(form).child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .mt_2()
                    .text_sm()
                    .child(format!("Current selection: {}", self.selection.summary()))
                    .child(format!("Path indices: {:?}", self.path.indices()))
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

    fn story_option_title(&self, depth: usize) -> SharedString {
        match (depth, self) {
            (0, Self::Web(WebRegion::UsEast(_))) => "US East".into(),
            (0, Self::Web(WebRegion::Europe(_))) => "Europe".into(),
            (0, Self::Desktop(DesktopPlatform::MacOs)) => "macOS".into(),
            (0, Self::Desktop(DesktopPlatform::Linux)) => "Linux".into(),
            (0, Self::Desktop(DesktopPlatform::Windows)) => "Windows".into(),
            (1, Self::Web(WebRegion::UsEast(zone) | WebRegion::Europe(zone))) => zone.name().into(),
            _ => self.variant_name().into(),
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

fn path_with_defaults(root_index: usize, child_count: usize) -> InfiniteSelectPath {
    let mut path = InfiniteSelectPath::new();
    path.set(0, root_index);

    for depth in 0..child_count {
        path.set(depth + 1, 0);
    }

    path
}

fn selected_row(row: usize) -> Option<IndexPath> {
    Some(IndexPath {
        section: 0,
        row,
        column: 0,
    })
}
