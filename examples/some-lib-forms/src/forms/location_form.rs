use es_fluent::FluentMessage as _;
use gpui::prelude::FluentBuilder as _;
use gpui::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ParentElement as _, Render, Styled, Window,
};
use gpui::{Subscription, div};
use gpui_component::ActiveTheme as _;
use gpui_component::Disableable as _;
use gpui_component::form::field;
use gpui_component::form::v_form;
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::separator::Separator;
use gpui_component::v_flex;
use gpui_form::infinite_select::{InfiniteSelectEvent, InfiniteSelectState};
use some_lib::structs::form_action::FormAction;
use some_lib::structs::location::*;
const CONTEXT: &str = "LocationFormForm";
fn localize(cx: &impl std::borrow::Borrow<App>, message: &impl es_fluent::FluentMessage) -> String {
    crate::i18n::localize_message(cx, message)
}
#[gpui_storybook::story_init]
pub fn init(_cx: &mut App) {}
#[gpui_storybook::story]
pub struct LocationFormForm {
    current_data: LocationFormFormValueHolder,
    fields: LocationFormFormFields,
    focus_handle: FocusHandle,
    _subscriptions: Vec<Subscription>,
}
impl Focusable for LocationFormForm {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
impl gpui_storybook::Story for LocationFormForm {
    fn title(cx: &gpui::App) -> String {
        crate::i18n::localize_label::<LocationForm>(cx)
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
                    text.parse::<String>().ok()
                };
            },
            _ => {},
        }
    }
    fn on_location_infinite_select_event(
        &mut self,
        _this: &Entity<InfiniteSelectState<Country>>,
        event: &InfiniteSelectEvent<Country>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        self.current_data.location = event.value().clone();
    }
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let current_data = LocationFormFormValueHolder::default();
        let name_input = cx.new(|cx| LocationFormFormComponents::name_input(window, cx));
        let location_infinite_select = cx.new(|cx| {
            InfiniteSelectState::<Country>::new_with_options(
                current_data.location.clone(),
                gpui_form::infinite_select::InfiniteSelectStateOptions::default().searchable(false),
                window,
                cx,
            )
        });
        let mut _subscriptions = vec![
            cx.subscribe_in(&name_input, window, Self::on_name_input_event),
            cx.subscribe_in(
                &location_infinite_select,
                window,
                Self::on_location_infinite_select_event,
            ),
        ];
        if let Some(value) = current_data.name.as_ref() {
            name_input.update(cx, |state, cx| {
                state.set_value(value.to_string(), window, cx);
            });
        }
        Self {
            current_data,
            fields: LocationFormFormFields {
                name_input,
                location_infinite_select,
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
        gpui_component::button::Button::new(format!("{}-submit-button", "location_form-form"))
            .label(label)
            .disabled(false)
            .on_click(cx.listener(move |this, _, window, cx| {
                on_submit(this.submit_payload(), window, cx);
            }))
    }
    fn reset_button(
        &self,
        cx: &mut Context<Self>,
        label: impl Into<gpui::SharedString>,
    ) -> gpui_component::button::Button {
        gpui_component::button::Button::new(format!("{}-reset-button", "location_form-form"))
            .label(label)
            .on_click(cx.listener(|this, _, window, cx| {
                this.reset_form(window, cx);
            }))
    }
    fn action_buttons(
        &self,
        cx: &mut Context<Self>,
        on_submit: impl Fn(LocationForm, &mut Window, &mut Context<Self>) + 'static,
    ) -> impl IntoElement {
        div()
            .flex()
            .gap_2()
            .child(self.submit_button(cx, localize(cx, &FormAction::Submit), on_submit))
            .child(self.reset_button(cx, localize(cx, &FormAction::Reset)))
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
            .child(Separator::horizontal())
            .child(
                v_form()
                    .child(
                        field()
                            .label({
                                let message = LocationFormLabelVariants::Name;
                                localize(cx, &message)
                            })
                            .description_fn({
                                let description = {
                                    let message = LocationFormDescriptionVariants::Name;
                                    localize(cx, &message)
                                };
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
                    .children(self.fields.location_infinite_select.read(cx).form_fields())
                    .child(field().label_indent(false).child(self.action_buttons(
                        cx,
                        |payload, _, _| {
                            let _ = payload;
                        },
                    ))),
            )
            .child(Separator::horizontal())
            .child({
                let mut form_state =
                    ::gpui_form::FormState::new(LocationFormFormValueHolder::default());
                form_state.replace_current(self.current_data.clone());
                format!("form_state.is_dirty: {}", form_state.is_dirty())
            })
            .child(format!(
                "field_paths: {}",
                vec![
                    LocationFormFormPath::name().to_string(),
                    LocationFormFormPath::location().to_string()
                ]
                .join(", ")
            ))
            .child(format!("value_holder: {:?}", self.current_data))
            .child(format!(
                "into_original: {:?}",
                LocationFormFormValueHolder::try_from(self.current_data.clone())
            ))
    }
}
