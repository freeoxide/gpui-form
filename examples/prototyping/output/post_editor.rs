use ::some_lib::structs::post::*;
use ::gpui::{Subscription, div};
use ::gpui::prelude::FluentBuilder as _;
use ::gpui_kit::component::{ActiveTheme as _, h_flex};
use ::gpui_kit::component::button::Button;
use ::gpui_kit::component::form::field;
use ::gpui_kit::component::input::{Input, InputEvent, InputState};
use crate::address::AddressForm;
/// i18n keys this scaffold resolves via `::rust_i18n::t!`; declare
/// them in the consumer crate's `locales/` for en, fr-FR and zh-CN.
/// - post_editor.title_label
/// - post_editor.title_description
/// - post_editor.tags_label
/// - post_editor.tags_item_label
/// - post_editor.tags_remove
/// - post_editor.address_label
#[allow(dead_code)]
pub const POST_EDITOR_I18N_KEYS: &[&str] = &[
    "post_editor.title_label",
    "post_editor.title_description",
    "post_editor.tags_label",
    "post_editor.tags_item_label",
    "post_editor.tags_remove",
    "post_editor.address_label",
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
const CONTEXT: &str = "PostEditorForm";
fn localize(cx: &impl ::std::borrow::Borrow<App>, key: &str) -> String {
    crate::i18n::localize_message(cx, key)
}
pub struct PostEditorForm {
    current_data: PostEditorFormValueHolder,
    fields: PostEditorFormFields,
    address_nested_form: Entity<AddressForm>,
    focus_handle: FocusHandle,
    _subscriptions: Vec<Subscription>,
}
impl Focusable for PostEditorForm {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
impl PostEditorForm {
    fn on_title_input_event(
        &mut self,
        state: &Entity<InputState>,
        event: &InputEvent,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        match event {
            InputEvent::Change => {
                let text = state.read(_cx).value();
                self.current_data.title = if text.is_empty() {
                    None
                } else {
                    text.parse::<String>().ok()
                };
            }
            _ => {}
        }
    }
    /// Repeatable-row control for this list field: one row per current
    /// item (Remove button each) plus an Add button appending a default
    /// item. Generated scaffold; edit freely.
    fn tags_list_control(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut rows = ::std::vec::Vec::new();
        for (index, item) in self.current_data.tags_items().iter().enumerate() {
            let remove_id = format!("{}-{}", "tags-remove", index);
            rows.push(
                h_flex()
                    .justify_between()
                    .child(
                        field()
                            .label(
                                format!(
                                    "{} {}", { let key : & str = "post_editor.tags_item_label";
                                    let translated = ::rust_i18n::t!(key); if & * translated ==
                                    key { "Tags".to_string() } else { translated.into_owned() }
                                    }, index + 1
                                ),
                            )
                            .child(format!("{:?}", item)),
                    )
                    .child(
                        Button::new(remove_id)
                            .label(
                                {
                                    let key: &str = "post_editor.tags_remove";
                                    let translated = ::rust_i18n::t!(key);
                                    if &*translated == key {
                                        "Remove".to_string()
                                    } else {
                                        translated.into_owned()
                                    }
                                }
                                    .clone(),
                            )
                            .on_click(
                                cx
                                    .listener(move |this, _, _, cx| {
                                        this.current_data.remove_tags(index);
                                        cx.notify();
                                    }),
                            ),
                    ),
            );
        }
        v_flex()
            .gap_2()
            .child(
                field()
                    .label(
                        {
                            let key: &str = "post_editor.tags_label";
                            let translated = ::rust_i18n::t!(key);
                            if &*translated == key {
                                "Tags".to_string()
                            } else {
                                translated.into_owned()
                            }
                        }
                            .clone(),
                    ),
            )
            .children(rows)
            .child(
                Button::new("tags-add")
                    .label("Add another tag".to_string().clone())
                    .on_click(
                        cx
                            .listener(|this, _, _, cx| {
                                this.current_data
                                    .add_tags(<String as ::core::default::Default>::default());
                                cx.notify();
                            }),
                    ),
            )
    }
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let current_data = PostEditorFormValueHolder::default();
        let title_input = cx.new(|cx| PostEditorFormComponents::title_input(window, cx));
        let address_nested_form = cx.new(|cx| AddressForm::new(window, cx));
        let mut _subscriptions = vec![
            cx.subscribe_in(& title_input, window, Self::on_title_input_event)
        ];
        if let Some(value) = current_data.title.as_ref() {
            title_input
                .update(
                    cx,
                    |state, cx| {
                        state.set_value(value.to_string(), window, cx);
                    },
                );
        }
        Self {
            current_data,
            fields: PostEditorFormFields {
                title_input,
            },
            address_nested_form,
            focus_handle: cx.focus_handle(),
            _subscriptions,
        }
    }
    fn reset_form(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        *self = Self::new(window, cx);
        cx.notify();
    }
    fn submit_payload(&self) -> Result<Option<PostEditor>, String> {
        match self.current_data.validate() {
            Ok(_) => {
                Ok(PostEditorFormValueHolder::try_from(self.current_data.clone()).ok())
            }
            Err(error) => Err(format!("{error:?}")),
        }
    }
    fn submit_button(
        &self,
        cx: &mut Context<Self>,
        label: impl Into<::gpui::SharedString>,
        on_submit: impl Fn(
            Result<Option<PostEditor>, String>,
            &mut Window,
            &mut Context<Self>,
        ) + 'static,
    ) -> ::gpui_kit::component::button::Button {
        ::gpui_kit::component::button::Button::new(
                format!("{}-submit-button", "post_editor-form"),
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
                format!("{}-reset-button", "post_editor-form"),
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
            Result<Option<PostEditor>, String>,
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
impl Render for PostEditorForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let validation_errors = self.current_data.validate().err();
        v_flex()
            .key_context(CONTEXT)
            .id("post_editor-form")
            .size_full()
            .p_4()
            .justify_start()
            .gap_3()
            .child(
                v_flex()
                    .text_lg()
                    .font_semibold()
                    .child(localize(cx, "post_editor_label")),
            )
            .child(Separator::horizontal())
            .child(
                v_form()
                    .child(
                        field()
                            .label({
                                let key: &str = "post_editor.title_label";
                                let translated = ::rust_i18n::t!(key);
                                if &*translated == key {
                                    "Title".to_string()
                                } else {
                                    translated.into_owned()
                                }
                            })
                            .description_fn({
                                let description = {
                                    let key: &str = "post_editor.title_description";
                                    let translated = ::rust_i18n::t!(key);
                                    if &*translated == key {
                                        "Title".to_string()
                                    } else {
                                        translated.into_owned()
                                    }
                                };
                                let error = {
                                    validation_errors
                                        .as_ref()
                                        .and_then(|e| {
                                            let errs = e.title().all();
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
                            .child(Input::new(&self.fields.title_input)),
                    )
                    .child(self.tags_list_control(cx))
                    .child(
                        div()
                            .border_1()
                            .rounded_md()
                            .p_2()
                            .child(
                                v_flex()
                                    .gap_2()
                                    .child(
                                        field()
                                            .label({
                                                let key: &str = "post_editor.address_label";
                                                let translated = ::rust_i18n::t!(key);
                                                if &*translated == key {
                                                    "Address".to_string()
                                                } else {
                                                    translated.into_owned()
                                                }
                                            }),
                                    )
                                    .child(address_nested_form.clone()),
                            ),
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
                    PostEditorFormValueHolder::default(),
                );
                form_state.replace_current(self.current_data.clone());
                format!("form_state.is_dirty: {}", form_state.is_dirty())
            })
            .child(
                format!(
                    "field_paths: {}", vec![PostEditorFormPath::title().to_string(),
                    PostEditorFormPath::tags().to_string(), PostEditorFormPath::address()
                    .to_string()] .join(", ")
                ),
            )
            .child(format!("value_holder: {:?}", self.current_data))
            .child(
                format!(
                    "into_original: {:?}", PostEditorFormValueHolder::try_from(self
                    .current_data.clone())
                ),
            )
    }
}
