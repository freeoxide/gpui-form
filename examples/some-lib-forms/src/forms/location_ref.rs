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
    fields: LocationFormFormFields,
    focus_handle: FocusHandle,
    _subscriptions: Vec<Subscription>,
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
        // Use generated component initialization logic?
        // The generated LocationFormFormComponents has methods for creating components.
        // LocationFormFormComponents::location_master_select(window, cx) creates default select.
        // We need to set initial selection.

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

        // We can manually create it or use the helper and then set selection.
        // The helper `location_master_select` sets default to 0 index.
        // Let's manually create to control initial selection, matching previous logic.
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

        let mut initial_path = gpui_form_component::TupleSelectPath::new();
        initial_path.set(0, initial_country_idx);

        let location_child_selects = LocationFormFormComponents::location_child_selects(
            &original_data.location,
            0,
            window,
            cx,
        );

        for child in &location_child_selects {
            let sub = cx.subscribe_in(child, window, Self::on_location_child_select_event);
            _subscriptions.push(sub);
        }

        Self {
            original_data: Arc::new(original_data.clone()),
            current_data: original_data.into(),
            errors: LocationFormFormErrors::default(),
            fields: LocationFormFormFields {
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
        let mut form = v_form().child(
            field()
                .label(LocationFormLabelKvFtl::Name.to_fluent_string())
                .description_fn({
                    let error = self.errors.name.clone();
                    let description = LocationFormDescriptionKvFtl::Name.to_fluent_string();
                    move |_, _| {
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(div().child(description.clone()))
                            .when(!error.is_empty(), |this| {
                                this.child(div().text_color(gpui::red()).child(error.clone()))
                            })
                    }
                })
                .child(Input::new(&self.fields.name_input)),
        );

        // ... rest of render ...

        // Master select field
        form = form.child(
            field()
                .label(self.current_data.location.type_label())
                .description(self.current_data.location.type_description())
                .child(Select::new(&self.fields.location_master_select)),
        );

        // Dynamic child select fields
        for (i, child_select) in self.fields.location_child_selects.iter().enumerate() {
            form = form.child(
                field()
                    .label(
                        self.current_data
                            .location
                            .child_label_at_depth(i)
                            .unwrap_or("".into()),
                    )
                    .description(
                        self.current_data
                            .location
                            .child_description_at_depth(i)
                            .unwrap_or("".into()),
                    )
                    .child(Select::new(child_select)),
            );
        }

        v_flex()
            .key_context(CONTEXT)
            .id("location-form")
            .size_full()
            .p_4()
            .justify_start()
            .gap_3()
            .child(Divider::horizontal())
            .child(form)
            .child(Divider::horizontal())
            .child(format!("Path: {:?}", self.fields.location_path))
            .child(format!("Data: {:?}", self.current_data))
            .child(format!(
                "Child selects count: {}",
                self.fields.location_child_selects.len()
            ))
            // Display location error at the bottom if present
            .when(!self.errors.location.is_empty(), |this| {
                 this.child(div().text_color(gpui::red()).child(self.errors.location.clone()))
            })
    }
}
