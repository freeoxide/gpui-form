use es_fluent::ToFluentString as _;
use gpui::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ParentElement as _, Render, Styled, Subscription, Window, div, prelude::FluentBuilder as _,
};
use gpui_component::{
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
            eprintln!("Master select event: {:?}", selected.variant_name());
            // The event contains the value directly (Country), not TupleSelectItem<Country>
            if let Some(index) = this.read(cx).selected_index(cx) {
                eprintln!("Master selected index: {:?}", index);
                self.fields.location_path.set(0, index.row);
                // Clear deeper levels when master changes
                self.fields.location_path.truncate(1);
            }
            self.current_data.location = selected.clone();

            // Update child selects based on new master selection
            self.rebuild_child_selects(window, cx);
            cx.notify();
        }
    }

    fn rebuild_child_selects(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        eprintln!("rebuild_child_selects called");
        // Clear existing child selects
        self.fields.location_child_selects.clear();

        // Get current value and check if it has inner types
        let current = &self.current_data.location;
        eprintln!(
            "Current location: {:?}, has_inner: {}",
            current.variant_name(),
            current.has_inner()
        );
        if !current.has_inner() {
            eprintln!("No inner, returning early");
            return;
        }

        // Build child select for the next level
        let child_names = current.child_variant_names();
        eprintln!("Child names: {:?}", child_names);
        if child_names.is_empty() {
            return;
        }

        // Create items for child select
        let items: Vec<gpui_form_component::TupleSelectItem<Country>> = child_names
            .iter()
            .enumerate()
            .filter_map(|(idx, name)| {
                current.set_child_by_index(idx).map(|variant| {
                    gpui_form_component::TupleSelectItem::new(variant, name.to_string())
                })
            })
            .collect();

        eprintln!("Created {} items for child select", items.len());
        if items.is_empty() {
            return;
        }

        let child_select = cx.new(|cx| SelectState::new(items, None, window, cx));

        // Subscribe to child select events
        let subscription =
            cx.subscribe_in(&child_select, window, Self::on_location_child_select_event);
        self._subscriptions.push(subscription);

        self.fields.location_child_selects.push(child_select);
        eprintln!(
            "Child selects count now: {}",
            self.fields.location_child_selects.len()
        );
    }

    fn on_location_child_select_event(
        &mut self,
        this: &Entity<SelectState<Vec<gpui_form_component::TupleSelectItem<Country>>>>,
        event: &SelectEvent<Vec<gpui_form_component::TupleSelectItem<Country>>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        eprintln!("Child select event received");
        if let SelectEvent::Confirm(Some(selected)) = event {
            eprintln!("Child select confirmed: {:?}", selected.variant_name());
            if let Some(index) = this.read(cx).selected_index(cx) {
                eprintln!("Child selected index: {:?}", index);
                // Find which level this child select is at
                let level = self.fields.location_child_selects
                    .iter()
                    .position(|s| s == this)
                    .map(|pos| pos + 1)  // +1 because master is level 0
                    .unwrap_or(1);

                self.fields.location_path.set(level, index.row);
                // Truncate deeper levels when this level changes
                self.fields.location_path.truncate(level + 1);
            }
            self.current_data.location = selected.clone();

            // Only rebuild if the selected value has children (to add next level)
            // Don't rebuild if it's a leaf - that would clear the current select
            if selected.has_inner() {
                // TODO: Add next level of child selects without clearing current
                eprintln!("Selected has inner, would add another level");
            }
            cx.notify();
        }
    }

    fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
        original_data: some_lib::structs::location::LocationForm,
    ) -> Self {
        let name_input = cx.new(|cx| LocationFormFormComponents::name_input(window, cx));
        let location_master_select =
            cx.new(|cx| LocationFormFormComponents::location_master_select(window, cx));

        let _subscriptions = vec![
            cx.subscribe_in(&name_input, window, Self::on_name_input_event),
            cx.subscribe_in(
                &location_master_select,
                window,
                Self::on_location_master_select_event,
            ),
        ];

        Self {
            original_data: Arc::new(original_data.clone()),
            current_data: original_data.into(),
            errors: LocationFormFormErrors::default(),
            fields: LocationFormFields {
                name_input,
                location_master_select,
                location_child_selects: Vec::new(),
                location_path: gpui_form_component::TupleSelectPath::new(),
            },
            focus_handle: cx.focus_handle(),
            _subscriptions,
        }
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
