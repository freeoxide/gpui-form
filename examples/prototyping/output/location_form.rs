use some_lib::structs::location::*;
use es_fluent::{ThisFtl as _, ToFluentString as _};
use gpui::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement,
    IntoElement, ParentElement as _, Render, Styled, Subscription, Window, div,
};
use gpui::prelude::FluentBuilder as _;
use gpui_component::{ActiveTheme as _, Disableable as _, IndexPath, v_flex};
use gpui_component::divider::Divider;
use gpui_component::form::{field, v_form};
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::select::{Select, SelectEvent, SelectState};
use gpui_form::infinite_select::InfiniteSelect;
use rust_decimal::Decimal;
use some_lib::structs::form_action::FormAction;
const CONTEXT: &str = "LocationFormForm";
#[gpui_storybook::story_init]
pub fn init(cx: &mut App) {}
#[gpui_storybook::story]
pub struct LocationFormForm {
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
        cx.new(|cx| Self::new(window, cx))
    }
}
impl LocationFormForm {
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
            SelectState<Vec<gpui_form::infinite_select::InfiniteSelectItem<Country>>>,
        >,
        event: &SelectEvent<
            Vec<gpui_form::infinite_select::InfiniteSelectItem<Country>>,
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
            SelectState<Vec<gpui_form::infinite_select::InfiniteSelectItem<Country>>>,
        >,
        event: &SelectEvent<
            Vec<gpui_form::infinite_select::InfiniteSelectItem<Country>>,
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
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let current_data = LocationFormFormValueHolder::default();
        let name_input = cx.new(|cx| LocationFormFormComponents::name_input(window, cx));
        let initial_location = &current_data.location;
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
                    gpui_form::infinite_select::InfiniteSelectItem<Country>,
                > = gpui_form::infinite_select::to_select_items::<Country>();
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
        if let Some(value) = current_data.name.as_ref() {
            name_input
                .update(
                    cx,
                    |state, cx| {
                        state.set_value(value.to_string(), window, cx);
                    },
                );
        }
        let mut location_path = gpui_form::infinite_select::InfiniteSelectPath::new();
        location_path.set(0, initial_variant_idx_location);
        let location_child_selects = LocationFormFormComponents::location_child_selects(
            &current_data.location,
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
            current_data,
            fields: LocationFormFormFields {
                name_input,
                location_master_select,
                location_child_selects,
                location_path: gpui_form::infinite_select::InfiniteSelectPath::new(),
            },
            focus_handle: cx.focus_handle(),
            _subscriptions,
        }
    }
    fn reset_form(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        *self = Self::new(window, cx);
        cx.notify();
    }
    fn submit_payload(&self) -> LocationForm {
        self.current_data.clone().into()
    }
    fn submit_button(
        &self,
        cx: &mut Context<Self>,
        label: impl Into<gpui::SharedString>,
        on_submit: impl Fn(LocationForm, &mut Window, &mut Context<Self>) + 'static,
    ) -> gpui_component::button::Button {
        gpui_component::button::Button::new(
                format!("{}-submit-button", "location_form-form"),
            )
            .label(label)
            .disabled(false)
            .on_click(
                cx
                    .listener(move |this, _, window, cx| {
                        on_submit(this.submit_payload(), window, cx);
                    }),
            )
    }
    fn reset_button(
        &self,
        cx: &mut Context<Self>,
        label: impl Into<gpui::SharedString>,
    ) -> gpui_component::button::Button {
        gpui_component::button::Button::new(
                format!("{}-reset-button", "location_form-form"),
            )
            .label(label)
            .on_click(
                cx
                    .listener(|this, _, window, cx| {
                        this.reset_form(window, cx);
                    }),
            )
    }
    fn action_buttons(
        &self,
        cx: &mut Context<Self>,
        on_submit: impl Fn(LocationForm, &mut Window, &mut Context<Self>) + 'static,
    ) -> impl IntoElement {
        div()
            .flex()
            .gap_2()
            .child(
                self.submit_button(cx, FormAction::Submit.to_fluent_string(), on_submit),
            )
            .child(self.reset_button(cx, FormAction::Reset.to_fluent_string()))
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
                            .label(LocationFormLabelVariants::Name.to_fluent_string())
                            .description_fn({
                                let description = LocationFormDescriptionVariants::Name
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
                    })
                    .child(
                        field()
                            .label_indent(false)
                            .child(
                                self
                                    .action_buttons(
                                        cx,
                                        |payload, _, _| {
                                            let _ = payload;
                                        },
                                    ),
                            ),
                    ),
            )
            .child(Divider::horizontal())
            .child(format!("value_holder: {:?}", self.current_data))
            .child(
                format!(
                    "into_original: {:?}", LocationFormFormValueHolder::try_from(self
                    .current_data.clone())
                ),
            )
    }
}
