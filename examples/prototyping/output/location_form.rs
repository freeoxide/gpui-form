use some_lib::structs::location_form::*;
use gpui::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement,
    IntoElement, ParentElement as _, Render, Styled, Subscription, Window, div,
    prelude::FluentBuilder as _,
};
use gpui_component::{
    checkbox::Checkbox, date_picker::{DatePicker, DatePickerEvent, DatePickerState},
    divider::Divider, select::{Select, SelectEvent, SelectState, SearchableVec},
    form::{field, v_form},
    input::{InputEvent, InputState, NumberInput, NumberInputEvent, StepAction, Input},
    switch::Switch, v_flex,
};
use rust_decimal::Decimal;
use std::sync::Arc;
use es_fluent::{ThisFtl as _, ToFluentString as _};
#[derive(Clone, Debug, es_fluent::EsFluent)]
pub enum LocationFormFormErrorsFtl {
    Name { value: String },
    Location { value: String },
}
const CONTEXT: &str = "LocationFormForm";
#[gpui_storybook::story_init]
pub fn init(cx: &mut App) {}
#[gpui_storybook::story]
pub struct LocationFormForm {
    original_data: Arc<LocationForm>,
    current_data: LocationFormFormValueHolder,
    errors: LocationFormFormErrors,
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
                self.current_data.name = text.to_owned().into();
            }
            _ => {}
        }
    }
    fn on_location_master_select_event(
        &mut self,
        this: &Entity<SelectState<Vec<gpui_form_component::TupleSelectItem<Country>>>>,
        event: &SelectEvent<Vec<gpui_form_component::TupleSelectItem<Country>>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            SelectEvent::Confirm(Some(item)) => {
                let selected = item.clone();
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
                        .subscribe_in(
                            child,
                            window,
                            Self::on_location_child_select_event,
                        );
                    self._subscriptions.push(sub);
                }
                self.fields.location_child_selects = new_children;
                cx.notify();
            }
            _ => {}
        }
    }
    fn on_location_child_select_event(
        &mut self,
        this: &Entity<SelectState<Vec<gpui_form_component::TupleSelectItem<Country>>>>,
        event: &SelectEvent<Vec<gpui_form_component::TupleSelectItem<Country>>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            SelectEvent::Confirm(Some(item)) => {
                let selected = item.clone();
                let level = self
                    .fields
                    .location_child_selects
                    .iter()
                    .position(|s| s == this)
                    .map(|p| p + 1)
                    .unwrap_or(1);
                if let Some(index) = this.read(cx).selected_index(cx) {
                    self.fields.location_path.set(level, index.row);
                }
                self.current_data.location = selected.clone();
                self.fields.location_child_selects.truncate(level);
                use gpui_form_component::TupleEnumInner as _;
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
            _ => {}
        }
    }
    fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
        original_data: LocationForm,
    ) -> Self {
        let name_input = cx.new(|cx| LocationFormFormComponents::name_input(window, cx));
        let location_master_select = cx
            .new(|cx| LocationFormFormComponents::location_master_select(window, cx));
        let _subscriptions = vec![
            cx.subscribe_in(& name_input, window, Self::on_name_input_event), cx
            .subscribe_in(& location_master_select, window,
            Self::on_location_master_select_event)
        ];
        Self {
            original_data: Arc::new(original_data.clone()),
            current_data: original_data.into(),
            errors: LocationFormFormErrors::default(),
            fields: LocationFormFormFields {
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
                                let error = self.errors.name.clone();
                                let description = LocationFormDescriptionKvFtl::Name
                                    .to_fluent_string();
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                        .when(
                                            !error.is_empty(),
                                            |this| {
                                                this.child(
                                                    div().text_color(gpui::red()).child(error.clone()),
                                                )
                                            },
                                        )
                                }
                            })
                            .child(Input::new(&self.fields.name_input)),
                    )
                    .child({
                        use gpui_form_component::TupleEnumInner as _;
                        field()
                            .label(self.current_data.location.type_label())
                            .description(self.current_data.location.type_description())
                            .child(Select::new(&self.fields.location_master_select))
                    })
                    .children({
                        use gpui_form_component::TupleEnumInner as _;
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
                                    .description(
                                        self
                                            .current_data
                                            .location
                                            .child_description_at_depth(i)
                                            .unwrap_or("".into()),
                                    )
                                    .child(Select::new(child))
                            })
                    })
                    .when(
                        !self.errors.location.is_empty(),
                        |form| {
                            form.child(
                                field()
                                    .child(
                                        div()
                                            .text_color(gpui::red())
                                            .child(self.errors.location.clone()),
                                    ),
                            )
                        },
                    ),
            )
            .child(Divider::horizontal())
            .child(format!("{:?}", self.current_data))
    }
}
