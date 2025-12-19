use es_fluent::ToFluentString as _;
use gpui::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ParentElement as _, Render, Styled, Subscription, Window, div, prelude::FluentBuilder as _,
};
use gpui_component::{
    IndexPath,
    divider::Divider,
    form::{field, v_form},
    input::{Input, InputEvent, InputState},
    select::{Select, SelectEvent, SelectState},
    v_flex,
};
use gpui_form_component::TupleEnumInner;
use some_lib::structs::location::*;
use std::sync::Arc;

#[derive(Clone, Debug, es_fluent::EsFluent)]
pub enum LocationFormErrorsFtl {
    Name { value: String },
    Location { value: String },
}

const CONTEXT: &str = "LocationForm";

#[gpui_storybook::story_init]
pub fn init(_cx: &mut App) {}

#[gpui_storybook::story]
pub struct LocationForm {
    original_data: Arc<some_lib::structs::location::LocationForm>,
    current_data: LocationFormFormValueHolder,
    errors: LocationFormFormErrors,
    fields: LocationFormFields,
    focus_handle: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

struct LocationFormFields {
    name_input: Entity<InputState>,
    location_master_select: Entity<SelectState<Vec<gpui_form_component::TupleSelectItem<Country>>>>,
    location_child_selects:
        Vec<Entity<SelectState<Vec<gpui_form_component::TupleSelectItem<Country>>>>>,
    location_path: gpui_form_component::TupleSelectPath,
}

impl Focusable for LocationForm {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl gpui_storybook::Story for LocationForm {
    fn title() -> String {
        some_lib::structs::location::LocationForm::this_ftl()
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(
            window,
            cx,
            some_lib::structs::location::LocationForm::default(),
        )
    }
}

impl LocationForm {
    pub fn view(
        window: &mut Window,
        cx: &mut App,
        original_data: some_lib::structs::location::LocationForm,
    ) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx, original_data))
    }

    fn on_name_input_event(
        &mut self,
        state: &Entity<InputState>,
        event: &InputEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let InputEvent::Change = event {
            let text = state.read(cx).value();
            self.current_data.name = text.to_owned().into();
        }
    }

    fn on_location_master_select_event(
        &mut self,
        this: &Entity<SelectState<Vec<gpui_form_component::TupleSelectItem<Country>>>>,
        event: &SelectEvent<Vec<gpui_form_component::TupleSelectItem<Country>>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let SelectEvent::Confirm(Some(selected)) = event {
            if let Some(index) = this.read(cx).selected_index(cx) {
                self.fields.location_path.set(0, index.row);
            }
            self.current_data.location = selected.clone();

            // Clear and rebuild all child selects from level 0
            self.fields.location_child_selects.clear();
            self.build_all_child_selects(&selected.clone(), window, cx);
            cx.notify();
        }
    }

    fn on_location_child_select_event(
        &mut self,
        this: &Entity<SelectState<Vec<gpui_form_component::TupleSelectItem<Country>>>>,
        event: &SelectEvent<Vec<gpui_form_component::TupleSelectItem<Country>>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let SelectEvent::Confirm(Some(selected)) = event {
            // Find which level this child select is at
            let level = self.fields.location_child_selects
                .iter()
                .position(|s| s == this)
                .map(|pos| pos + 1)  // +1 because master is level 0
                .unwrap_or(1);

            if let Some(index) = this.read(cx).selected_index(cx) {
                self.fields.location_path.set(level, index.row);
            }
            self.current_data.location = selected.clone();

            // Remove child selects after this level and rebuild
            self.fields.location_child_selects.truncate(level);

            // Add child selects for remaining levels if the selected value has children
            if selected.has_inner() {
                self.build_child_selects_for_remaining_levels(&selected.clone(), level, window, cx);
            }
            cx.notify();
        }
    }

    /// Build all child selects for a given parent value (used after master selection)
    fn build_all_child_selects(
        &mut self,
        parent: &Country,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.build_child_selects_for_remaining_levels(parent, 0, window, cx);
    }

    /// Build child selects for remaining levels starting from a given level
    fn build_child_selects_for_remaining_levels(
        &mut self,
        parent: &Country,
        start_level: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let max_depth = Country::depth();
        let mut current_value = parent.clone();

        for level in start_level..(max_depth - 1) {
            // For level 0 (first child select), use child_variant_names (children of the root variant)
            // For level 1+ (second+ child select), use inner_child_variant_names (children of the inner value)
            let (child_names, has_more) = if level == 0 {
                (
                    current_value.child_variant_names(),
                    current_value.has_inner(),
                )
            } else {
                (
                    current_value.inner_child_variant_names(),
                    current_value.inner_has_inner(),
                )
            };

            if !has_more || child_names.is_empty() {
                break;
            }

            // Create items for this level
            let items: Vec<gpui_form_component::TupleSelectItem<Country>> = child_names
                .iter()
                .enumerate()
                .filter_map(|(idx, name)| {
                    let variant = if level == 0 {
                        current_value.set_child_by_index(idx)
                    } else {
                        current_value.inner_set_child_by_index(idx)
                    };
                    variant.map(|v| gpui_form_component::TupleSelectItem::new(v, name.to_string()))
                })
                .collect();

            if items.is_empty() {
                break;
            }

            // Default to first item selected
            let selected_index = Some(IndexPath {
                section: 0,
                row: 0,
                column: 0,
            });
            self.fields.location_path.set(level + 1, 0);

            let child_select =
                cx.new(|cx| SelectState::new(items.clone(), selected_index, window, cx));

            let subscription =
                cx.subscribe_in(&child_select, window, Self::on_location_child_select_event);
            self._subscriptions.push(subscription);

            self.fields.location_child_selects.push(child_select);

            // Move to the first child for the next iteration
            if let Some(first_item) = items.first() {
                current_value = first_item.get_value().clone();
            } else {
                break;
            }
        }
    }

    fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
        original_data: some_lib::structs::location::LocationForm,
    ) -> Self {
        let name_input = cx.new(|cx| LocationFormFormComponents::name_input(window, cx));

        // Create master select with the correct initial selection
        let initial_location = &original_data.location;
        let master_variants = Country::variants();
        let initial_country_name = initial_location.variant_name();
        let initial_country_idx = master_variants
            .iter()
            .position(|v| v.variant_name() == initial_country_name)
            .unwrap_or(0);

        let master_selected_index = Some(IndexPath {
            section: 0,
            row: initial_country_idx,
            column: 0,
        });

        let location_master_select = cx.new(|cx| {
            let items: Vec<gpui_form_component::TupleSelectItem<Country>> =
                gpui_form_component::tuple_enum_to_select_items::<Country>();
            SelectState::new(items, master_selected_index, window, cx)
        });

        let mut _subscriptions = vec![
            cx.subscribe_in(&name_input, window, Self::on_name_input_event),
            cx.subscribe_in(
                &location_master_select,
                window,
                Self::on_location_master_select_event,
            ),
        ];

        // Build initial path
        let mut initial_path = gpui_form_component::TupleSelectPath::new();
        initial_path.set(0, initial_country_idx);

        let mut form = Self {
            original_data: Arc::new(original_data.clone()),
            current_data: original_data.into(),
            errors: LocationFormFormErrors::default(),
            fields: LocationFormFields {
                name_input,
                location_master_select,
                location_child_selects: Vec::new(),
                location_path: initial_path,
            },
            focus_handle: cx.focus_handle(),
            _subscriptions,
        };

        // Build initial child selects based on default value
        let initial_value = form.current_data.location.clone();
        form.build_all_child_selects(&initial_value, window, cx);

        form
    }
}

impl Render for LocationForm {
    fn render(&mut self, _: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let mut location_selects = v_flex()
            .gap_2()
            .child(Select::new(&self.fields.location_master_select));

        // Add child selects dynamically
        for child_select in &self.fields.location_child_selects {
            location_selects = location_selects.child(Select::new(child_select));
        }

        v_flex()
            .key_context(CONTEXT)
            .id("location-form")
            .size_full()
            .p_4()
            .justify_start()
            .gap_3()
            .child(Divider::horizontal())
            .child(
                v_form()
                    .child(
                        field()
                            .label(LocationFormLabelKvFtl::Name.to_fluent_string())
                            .description_fn({
                                let error = self.errors.name.clone();
                                let description =
                                    LocationFormDescriptionKvFtl::Name.to_fluent_string();
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                        .when(!error.is_empty(), |this| {
                                            this.child(
                                                div().text_color(gpui::red()).child(error.clone()),
                                            )
                                        })
                                }
                            })
                            .child(Input::new(&self.fields.name_input)),
                    )
                    .child(
                        field()
                            .label(LocationFormLabelKvFtl::Location.to_fluent_string())
                            .description_fn({
                                let error = self.errors.location.clone();
                                let description =
                                    LocationFormDescriptionKvFtl::Location.to_fluent_string();
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                        .when(!error.is_empty(), |this| {
                                            this.child(
                                                div().text_color(gpui::red()).child(error.clone()),
                                            )
                                        })
                                }
                            })
                            .child(location_selects),
                    ),
            )
            .child(Divider::horizontal())
            .child(format!("Path: {:?}", self.fields.location_path))
            .child(format!("Data: {:?}", self.current_data))
            .child(format!(
                "Child selects count: {}",
                self.fields.location_child_selects.len()
            ))
    }
}
