use some_lib::structs::custom_vec_string_external::*;
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
use gpui_form_component::infinite_select::InfiniteSelect;
use es_fluent::{ThisFtl as _, ToFluentString as _};
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
                                div()
                                    .child(
                                        format!(
                                            "Custom component `{}` is initialized; add manual render/subscriptions.",
                                            "tags"
                                        ),
                                    ),
                            ),
                    ),
            )
            .child(Divider::horizontal())
            .child(format!("{:?}", self.current_data))
    }
}
