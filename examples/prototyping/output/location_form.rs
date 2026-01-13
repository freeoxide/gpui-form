use some_lib::structs::location::*;
use gpui::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement,
    IntoElement, ParentElement as _, Render, Styled, Subscription, Window, div,
    prelude::FluentBuilder as _,
};
use gpui_component::{
    ActiveTheme as _, IndexPath, checkbox::Checkbox,
    date_picker::{DatePicker, DatePickerEvent, DatePickerState},
    divider::Divider, form::{field, v_form},
    input::{Input, InputEvent, InputState, NumberInput, NumberInputEvent, StepAction},
    select::{SearchableVec, Select, SelectEvent, SelectState},
    switch::Switch, v_flex,
};
use gpui_form::component::infinite_select::InfiniteSelect;
use std::sync::Arc;
use es_fluent::{ThisFtl as _, ToFluentString as _};
use rust_decimal::Decimal;
const CONTEXT: &str = "LocationFormForm";
#[gpui_storybook::story_init]
pub fn init(cx: &mut App) {}
#[gpui_storybook::story]
pub struct LocationFormForm {
    original_data: Arc<LocationForm>,
    current_data: LocationFormFormValueHolder,
    fields: LocationFormFormFields,
    focus_handle: FocusHandle,
    _subscriptions: Vec<Subscription>,
}
impl Focusable for LocationFormForm {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
impl gpui_storybook::Story for LocationFormForm {
    fn title() -> String {
        LocationForm::this_ftl()
    }
    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx, LocationForm::default())
    }
}
impl LocationFormForm {
    pub fn view(
        window: &mut Window,
        cx: &mut App,
        original_data: LocationForm,
    ) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx, original_data))
    }
    fn on_name_input_event(
        &mut self,
        state: &Entity<InputState>,
        event: &InputEvent,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        match event {
            InputEvent::Change => {
                let text = state.read(_cx).value();
                self.current_data.name = if text.is_empty() {
                    None
                } else {
                    Some(text.to_string())
                };
            }
            _ => {}
        }
    }
    fn on_location_master_select_event(
        &mut self,
        this: &Entity<
            SelectState<
                Vec<gpui_form::component::infinite_select::InfiniteSelectItem<Country>>,
            >,
        >,
        event: &SelectEvent<
            Vec<gpui_form::component::infinite_select::InfiniteSelectItem<Country>>,
        >,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let SelectEvent::Confirm(Some(selected)) = event {
            if let Some(index) = this.read(cx).selected_index(cx) {
                self.fields.location_path.set(0, index.row);
            }
            self.current_data.location = selected.clone();
            self.fields.location_child_selects.clear();
            let new_children = LocationFormFormComponents::location_child_selects(
                &selected,
                0,
                window,
                cx,
            );
            for child in &new_children {
                let sub = cx
                    .subscribe_in(child, window, Self::on_location_child_select_event);
                self._subscriptions.push(sub);
            }
            self.fields.location_child_selects = new_children;
            cx.notify();
        }
    }
    fn on_location_child_select_event(
        &mut self,
        this: &Entity<
            SelectState<
                Vec<gpui_form::component::infinite_select::InfiniteSelectItem<Country>>,
            >,
        >,
        event: &SelectEvent<
            Vec<gpui_form::component::infinite_select::InfiniteSelectItem<Country>>,
        >,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let SelectEvent::Confirm(Some(selected)) = event {
            let level = self
                .fields
                .location_child_selects
                .iter()
                .position(|s| s == this)
                .map(|pos| pos + 1)
                .unwrap_or(1);
            if let Some(index) = this.read(cx).selected_index(cx) {
                self.fields.location_path.set(level, index.row);
            }
            self.current_data.location = selected.clone();
            self.fields.location_child_selects.truncate(level);
            if selected.has_inner() {
                let new_children = LocationFormFormComponents::location_child_selects(
                    &selected,
                    level,
                    window,
                    cx,
                );
                for child in &new_children {
                    let sub = cx
                        .subscribe_in(
                            child,
                            window,
                            Self::on_location_child_select_event,
                        );
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
        original_data: LocationForm,
    ) -> Self {
        let name_input = cx.new(|cx| LocationFormFormComponents::name_input(window, cx));
        let initial_location = &original_data.location;
        let master_variants_location = Country::variants();
        let initial_variant_name_location = initial_location.variant_name();
        let initial_variant_idx_location = master_variants_location
            .iter()
            .position(|v| v.variant_name() == initial_variant_name_location)
            .unwrap_or(0);
        let master_selected_index_location = Some(gpui_component::IndexPath {
            section: 0,
            row: initial_variant_idx_location,
            column: 0,
        });
        let location_master_select = cx
            .new(|cx| {
                let items: Vec<
                    gpui_form::component::infinite_select::InfiniteSelectItem<Country>,
                > = gpui_form::component::infinite_select::to_select_items::<Country>();
                gpui_component::select::SelectState::new(
                    items,
                    master_selected_index_location,
                    window,
                    cx,
                )
            });
        let mut _subscriptions = vec![
            cx.subscribe_in(& name_input, window, Self::on_name_input_event), cx
            .subscribe_in(& location_master_select, window,
            Self::on_location_master_select_event)
        ];
        let mut location_path = gpui_form::component::infinite_select::InfiniteSelectPath::new();
        location_path.set(0, initial_variant_idx_location);
        let location_child_selects = LocationFormFormComponents::location_child_selects(
            &original_data.location,
            0,
            window,
            cx,
        );
        for child in &location_child_selects {
            let sub = cx
                .subscribe_in(child, window, Self::on_location_child_select_event);
            _subscriptions.push(sub);
        }
        Self {
            original_data: Arc::new(original_data.clone()),
            current_data: original_data.into(),
            fields: LocationFormFormFields {
                name_input,
                location_master_select,
                location_child_selects,
                location_path: gpui_form::component::infinite_select::InfiniteSelectPath::new(),
            },
            focus_handle: cx.focus_handle(),
            _subscriptions,
        }
    }
}
impl Render for LocationFormForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .key_context(CONTEXT)
            .id("location_form-form")
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
                                let description = LocationFormDescriptionKvFtl::Name
                                    .to_fluent_string();
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                }
                            })
                            .child(Input::new(&self.fields.name_input)),
                    )
                    .child({
                        field()
                            .label(self.current_data.location.type_label())
                            .description_fn({
                                let description = self
                                    .current_data
                                    .location
                                    .type_description();
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                }
                            })
                            .child(Select::new(&self.fields.location_master_select))
                    })
                    .children({
                        self.fields
                            .location_child_selects
                            .iter()
                            .enumerate()
                            .map(|(i, child)| {
                                field()
                                    .label(
                                        self
                                            .current_data
                                            .location
                                            .child_label_at_depth(i)
                                            .unwrap_or("".into()),
                                    )
                                    .description_fn({
                                        let description = self
                                            .current_data
                                            .location
                                            .child_description_at_depth(i)
                                            .unwrap_or("".into());
                                        move |_, _| {
                                            div()
                                                .flex()
                                                .flex_col()
                                                .gap_1()
                                                .child(div().child(description.clone()))
                                        }
                                    })
                                    .child(Select::new(child))
                            })
                    }),
            )
            .child(Divider::horizontal())
            .child(format!("{:?}", self.current_data))
    }
}
