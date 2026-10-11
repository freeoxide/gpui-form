use ::some_lib::structs::post::*;
use ::gpui::{Subscription, div};
use ::gpui::prelude::FluentBuilder as _;
use ::gpui_kit::component::ActiveTheme as _;
use ::gpui_kit::component::form::field;
use ::gpui_kit::component::input::{Input, InputEvent, InputState};
/// i18n keys this scaffold resolves via `::rust_i18n::t!`; declare
/// them in the consumer crate's `locales/` for en, fr-FR and zh-CN.
/// - address.city_label
/// - address.city_description
/// - address.zip_label
/// - address.zip_description
#[allow(dead_code)]
pub const ADDRESS_I18N_KEYS: &[&str] = &[
    "address.city_label",
    "address.city_description",
    "address.zip_label",
    "address.zip_description",
];
use ::gpui::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement,
    IntoElement, ParentElement as _, Render, Styled, Window,
};
use ::gpui_kit::component::Disableable as _;
use ::gpui_kit::component::separator::Separator;
use ::gpui_kit::component::form::v_form;
use ::gpui_kit::component::v_flex;
use ::some_lib::structs::form_action::FormAction;
const CONTEXT: &str = "AddressForm";
fn localize(cx: &impl ::std::borrow::Borrow<App>, key: &str) -> String {
    crate::i18n::localize_message(cx, key)
}
pub struct AddressForm {
    current_data: AddressFormValueHolder,
    fields: AddressFormFields,
    focus_handle: FocusHandle,
    _subscriptions: Vec<Subscription>,
}
impl Focusable for AddressForm {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
impl AddressForm {
    fn on_city_input_event(
        &mut self,
        state: &Entity<InputState>,
        event: &InputEvent,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        match event {
            InputEvent::Change => {
                let text = state.read(_cx).value();
                self.current_data.city = if text.is_empty() {
                    None
                } else {
                    text.parse::<String>().ok()
                };
            }
            _ => {}
        }
    }
    fn on_zip_input_event(
        &mut self,
        state: &Entity<InputState>,
        event: &InputEvent,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        match event {
            InputEvent::Change => {
                let text = state.read(_cx).value();
                self.current_data.zip = if text.is_empty() {
                    None
                } else {
                    text.parse::<String>().ok()
                };
            }
            _ => {}
        }
    }
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let current_data = AddressFormValueHolder::default();
        let city_input = cx.new(|cx| AddressFormComponents::city_input(window, cx));
        let zip_input = cx.new(|cx| AddressFormComponents::zip_input(window, cx));
        let mut _subscriptions = vec![
            cx.subscribe_in(& city_input, window, Self::on_city_input_event), cx
            .subscribe_in(& zip_input, window, Self::on_zip_input_event)
        ];
        if let Some(value) = current_data.city.as_ref() {
            city_input
                .update(
                    cx,
                    |state, cx| {
                        state.set_value(value.to_string(), window, cx);
                    },
                );
        }
        if let Some(value) = current_data.zip.as_ref() {
            zip_input
                .update(
                    cx,
                    |state, cx| {
                        state.set_value(value.to_string(), window, cx);
                    },
                );
        }
        Self {
            current_data,
            fields: AddressFormFields {
                city_input,
                zip_input,
            },
            focus_handle: cx.focus_handle(),
            _subscriptions,
        }
    }
    fn reset_form(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        *self = Self::new(window, cx);
        cx.notify();
    }
    fn submit_payload(&self) -> Result<Option<Address>, String> {
        match self.current_data.validate() {
            Ok(_) => Ok(AddressFormValueHolder::try_from(self.current_data.clone()).ok()),
            Err(error) => Err(format!("{error:?}")),
        }
    }
    fn submit_button(
        &self,
        cx: &mut Context<Self>,
        label: impl Into<::gpui::SharedString>,
        on_submit: impl Fn(
            Result<Option<Address>, String>,
            &mut Window,
            &mut Context<Self>,
        ) + 'static,
    ) -> ::gpui_kit::component::button::Button {
        ::gpui_kit::component::button::Button::new(
                format!("{}-submit-button", "address-form"),
            )
            .label(label)
            .disabled(self.current_data.validate().is_err())
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
        label: impl Into<::gpui::SharedString>,
    ) -> ::gpui_kit::component::button::Button {
        ::gpui_kit::component::button::Button::new(
                format!("{}-reset-button", "address-form"),
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
        on_submit: impl Fn(
            Result<Option<Address>, String>,
            &mut Window,
            &mut Context<Self>,
        ) + 'static,
    ) -> impl IntoElement {
        div()
            .flex()
            .gap_2()
            .child(
                self.submit_button(cx, localize(cx, FormAction::Submit.key()), on_submit),
            )
            .child(self.reset_button(cx, localize(cx, FormAction::Reset.key())))
    }
}
impl Render for AddressForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let validation_errors = self.current_data.validate().err();
        v_flex()
            .key_context(CONTEXT)
            .id("address-form")
            .size_full()
            .p_4()
            .justify_start()
            .gap_3()
            .child(
                v_flex().text_lg().font_semibold().child(localize(cx, "address_label")),
            )
            .child(Separator::horizontal())
            .child(
                v_form()
                    .child(
                        field()
                            .label({
                                let key: &str = "address.city_label";
                                let translated = ::rust_i18n::t!(key);
                                if &*translated == key {
                                    "City".to_string()
                                } else {
                                    translated.into_owned()
                                }
                            })
                            .description_fn({
                                let description = {
                                    let key: &str = "address.city_description";
                                    let translated = ::rust_i18n::t!(key);
                                    if &*translated == key {
                                        "City".to_string()
                                    } else {
                                        translated.into_owned()
                                    }
                                };
                                let error = {
                                    validation_errors
                                        .as_ref()
                                        .and_then(|e| {
                                            let errs = e.city().all();
                                            if errs.is_empty() {
                                                None
                                            } else {
                                                Some(
                                                    errs
                                                        .iter()
                                                        .map(|v| v.to_string())
                                                        .collect::<Vec<_>>()
                                                        .join("\n"),
                                                )
                                            }
                                        })
                                };
                                let error_color = cx.theme().danger;
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                        .when(
                                            error.is_some(),
                                            |this| {
                                                this.child(
                                                    div()
                                                        .text_color(error_color)
                                                        .child(error.clone().unwrap_or_default()),
                                                )
                                            },
                                        )
                                }
                            })
                            .child(Input::new(&self.fields.city_input)),
                    )
                    .child(
                        field()
                            .label({
                                let key: &str = "address.zip_label";
                                let translated = ::rust_i18n::t!(key);
                                if &*translated == key {
                                    "Zip".to_string()
                                } else {
                                    translated.into_owned()
                                }
                            })
                            .description_fn({
                                let description = {
                                    let key: &str = "address.zip_description";
                                    let translated = ::rust_i18n::t!(key);
                                    if &*translated == key {
                                        "Zip".to_string()
                                    } else {
                                        translated.into_owned()
                                    }
                                };
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                }
                            })
                            .child(Input::new(&self.fields.zip_input)),
                    )
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
            .child(Separator::horizontal())
            .child({
                let mut form_state = ::gpui_form::FormState::new(
                    AddressFormValueHolder::default(),
                );
                form_state.replace_current(self.current_data.clone());
                format!("form_state.is_dirty: {}", form_state.is_dirty())
            })
            .child(
                format!(
                    "field_paths: {}", vec![AddressFormPath::city().to_string(),
                    AddressFormPath::zip().to_string()] .join(", ")
                ),
            )
            .child(format!("value_holder: {:?}", self.current_data))
            .child(
                format!(
                    "into_original: {:?}", AddressFormValueHolder::try_from(self
                    .current_data.clone())
                ),
            )
    }
}
