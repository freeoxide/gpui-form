use some_lib::structs::custom_vec_string_external::*;
use es_fluent::{ThisFtl as _, ToFluentString as _};
use gpui::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement,
    IntoElement, ParentElement as _, Render, Styled, Subscription, Window, div,
};
use gpui::prelude::FluentBuilder as _;
use gpui_component::{ActiveTheme as _, v_flex};
use gpui_component::divider::Divider;
use gpui_component::form::{field, v_form};
use some_lib_custom_components::ExternalTagsInput;
use rust_decimal::Decimal;
const CONTEXT: &str = "ExternalShapeVecStringInputListForm";
#[gpui_storybook::story_init]
pub fn init(cx: &mut App) {}
#[gpui_storybook::story]
pub struct ExternalShapeVecStringInputListForm {
    current_data: ExternalShapeVecStringInputListFormValueHolder,
    fields: ExternalShapeVecStringInputListFormFields,
    focus_handle: FocusHandle,
}
impl Focusable for ExternalShapeVecStringInputListForm {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
impl gpui_storybook::Story for ExternalShapeVecStringInputListForm {
    fn title() -> String {
        ExternalShapeVecStringInputList::this_ftl()
    }
    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        cx.new(|cx| Self::new(window, cx))
    }
}
impl ExternalShapeVecStringInputListForm {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let current_data = ExternalShapeVecStringInputListFormValueHolder::default();
        let tags_custom = cx
            .new(|cx| ExternalShapeVecStringInputListFormComponents::tags_custom(
                window,
                cx,
            ));
        Self {
            current_data,
            fields: ExternalShapeVecStringInputListFormFields {
                tags_custom,
            },
            focus_handle: cx.focus_handle(),
        }
    }
    fn reset_form(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        *self = Self::new(window, cx);
        cx.notify();
    }
    fn submit_payload(&self) -> ExternalShapeVecStringInputList {
        self.current_data.clone().into()
    }
    fn submit_button(
        &self,
        cx: &mut Context<Self>,
        label: impl Into<gpui::SharedString>,
        on_submit: impl Fn(
            ExternalShapeVecStringInputList,
            &mut Window,
            &mut Context<Self>,
        ) + 'static,
    ) -> gpui_component::button::Button {
        gpui_component::button::Button::new(
                format!("{}-submit-button", "external_shape_vec_string_input_list-form"),
            )
            .label(label)
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
                format!("{}-reset-button", "external_shape_vec_string_input_list-form"),
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
            ExternalShapeVecStringInputList,
            &mut Window,
            &mut Context<Self>,
        ) + 'static,
    ) -> impl IntoElement {
        div()
            .flex()
            .gap_2()
            .child(self.submit_button(cx, "Submit", on_submit))
            .child(self.reset_button(cx, "Reset"))
    }
}
impl Render for ExternalShapeVecStringInputListForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .key_context(CONTEXT)
            .id("external_shape_vec_string_input_list-form")
            .size_full()
            .p_4()
            .justify_start()
            .gap_3()
            .child(Divider::horizontal())
            .child(
                v_form()
                    .child(
                        field()
                            .label(
                                ExternalShapeVecStringInputListLabelVariants::Tags
                                    .to_fluent_string(),
                            )
                            .description_fn({
                                let description = ExternalShapeVecStringInputListDescriptionVariants::Tags
                                    .to_fluent_string();
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                }
                            })
                            .child(
                                some_lib_custom_components::ExternalTagsInput::new(
                                    &self.fields.tags_custom,
                                ),
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
            .child(Divider::horizontal())
            .child(format!("value_holder: {:?}", self.current_data))
            .child(
                format!(
                    "into_original: {:?}",
                    ExternalShapeVecStringInputListFormValueHolder::try_from(self
                    .current_data.clone())
                ),
            )
    }
}
