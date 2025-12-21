use es_fluent::{ThisFtl as _, ToFluentString as _};
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

            let new_children =
                LocationFormFormComponents::location_child_selects(&selected, 0, window, cx);

            for child in &new_children {
                let sub = cx.subscribe_in(child, window, Self::on_location_child_select_event);
                self._subscriptions.push(sub);
            }

            self.fields.location_child_selects = new_children;
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
                let new_children = LocationFormFormComponents::location_child_selects(
                    &selected, level, window, cx,
                );

                for child in &new_children {
                    let sub = cx.subscribe_in(child, window, Self::on_location_child_select_event);
                    self._subscriptions.push(sub);
                }

                self.fields.location_child_selects.extend(new_children);
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

        // Build initial child selects
        // Note: Using the helper resets deeper selections to default (0),
        // matching the behavior of the previous manual implementation.
        let location_child_selects = LocationFormFormComponents::location_child_selects(
            &original_data.location, // Using original data which is correct for starting level 0
            0,
            window,
            cx,
        );

        // Subscribe to initial children
        // We can't use Self::on_location_child_select_event directly here easily because we are in 'new'
        // and subscriptions usually require a handler on the view.
        // Wait, cx.subscribe_in works fine with Self::method.

        for child in &location_child_selects {
            let sub = cx.subscribe_in(child, window, Self::on_location_child_select_event);
            _subscriptions.push(sub);
        }

        Self {
            original_data: Arc::new(original_data.clone()),
            current_data: original_data.into(),
            errors: LocationFormFormErrors::default(),
            fields: LocationFormFields {
                name_input,
                location_master_select,
                location_child_selects,
                location_path: initial_path,
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
