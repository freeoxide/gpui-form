use es_fluent::ToFluentString as _;
use gpui::{
    App, AppContext as _, Context, Entity, Focusable, IntoElement, ParentElement as _, Render,
    SharedString, Styled as _, Subscription, Window, div,
};
use gpui_component::form::v_form;
use icu_locale_core::locale;
use jiff::civil::{Date as JiffDate, date};

use crate::i18n::DatePickerStoryText;
use gpui_form_component::date_picker::{
    DateDisplayStyle, DatePicker, DatePickerEvent, DatePickerState,
};

use super::common::{story_field, story_panel};

#[gpui_storybook::story]
pub struct DatePickerStory {
    default_picker: Entity<DatePickerState>,
    long_picker: Entity<DatePickerState>,
    localized_picker: Entity<DatePickerState>,
    compact_picker: Entity<DatePickerState>,
    last_change: SharedString,
    _subscriptions: Vec<Subscription>,
}

impl gpui_storybook::Story for DatePickerStory {
    fn title() -> String {
        DatePickerStoryText::Title.to_fluent_string()
    }

    fn description() -> String {
        DatePickerStoryText::Description.to_fluent_string()
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        cx.new(|cx| Self::new(window, cx))
    }
}

impl Focusable for DatePickerStory {
    fn focus_handle(&self, cx: &App) -> gpui::FocusHandle {
        self.default_picker.read(cx).focus_handle(cx)
    }
}

impl DatePickerStory {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let default_picker = cx.new(|cx| DatePickerState::new(window, cx));
        let long_picker = cx.new(|cx| {
            let mut state = DatePickerState::new(window, cx)
                .display_locale(locale!("en-US"))
                .display_style(DateDisplayStyle::Long);
            state.set_date(Some(date(2026, 4, 20)), window, cx);
            state
        });
        let localized_picker = cx.new(|cx| {
            let mut state = DatePickerState::new(window, cx)
                .display_locale(locale!("fr-FR"))
                .display_style(DateDisplayStyle::Long);
            state.set_date(Some(date(2026, 7, 14)), window, cx);
            state
        });
        let compact_picker = cx.new(|cx| {
            let mut state = DatePickerState::new(window, cx)
                .display_locale(locale!("en-GB"))
                .display_style(DateDisplayStyle::Short);
            state.set_date(Some(date(2026, 10, 5)), window, cx);
            state
        });

        let subscriptions = vec![
            cx.subscribe_in(&default_picker, window, Self::on_picker_change),
            cx.subscribe_in(&long_picker, window, Self::on_picker_change),
            cx.subscribe_in(&localized_picker, window, Self::on_picker_change),
            cx.subscribe_in(&compact_picker, window, Self::on_picker_change),
        ];

        Self {
            default_picker,
            long_picker,
            localized_picker,
            compact_picker,
            last_change: DatePickerStoryText::InitialEvent.to_fluent_string().into(),
            _subscriptions: subscriptions,
        }
    }

    fn picker_label(&self, picker: &Entity<DatePickerState>) -> String {
        if picker == &self.default_picker {
            DatePickerStoryText::DefaultLabel.to_fluent_string()
        } else if picker == &self.long_picker {
            DatePickerStoryText::LongLabel.to_fluent_string()
        } else if picker == &self.localized_picker {
            DatePickerStoryText::FrenchLabel.to_fluent_string()
        } else if picker == &self.compact_picker {
            DatePickerStoryText::CompactLabel.to_fluent_string()
        } else {
            DatePickerStoryText::UnknownLabel.to_fluent_string()
        }
    }

    fn on_picker_change(
        &mut self,
        picker: &Entity<DatePickerState>,
        event: &DatePickerEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let DatePickerEvent::Change(date) = event;
        self.last_change = DatePickerStoryText::Changed {
            picker: self.picker_label(picker),
            date: describe_date(*date),
        }
        .to_fluent_string()
        .into();
        cx.notify();
    }
}

impl Render for DatePickerStory {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let default_value = describe_date(self.default_picker.read(cx).date());
        let long_value = describe_date(self.long_picker.read(cx).date());
        let localized_value = describe_date(self.localized_picker.read(cx).date());
        let compact_value = describe_date(self.compact_picker.read(cx).date());

        let form = v_form()
            .child(story_field(
                DatePickerStoryText::DefaultFieldLabel.to_fluent_string(),
                DatePickerStoryText::DefaultFieldDescription.to_fluent_string(),
                DatePicker::new(&self.default_picker)
                    .placeholder(DatePickerStoryText::LaunchPlaceholder.to_fluent_string())
                    .cleanable(true),
            ))
            .child(story_field(
                DatePickerStoryText::LongFieldLabel.to_fluent_string(),
                DatePickerStoryText::LongFieldDescription.to_fluent_string(),
                DatePicker::new(&self.long_picker)
                    .cleanable(true)
                    .number_of_months(1),
            ))
            .child(story_field(
                DatePickerStoryText::FrenchFieldLabel.to_fluent_string(),
                DatePickerStoryText::FrenchFieldDescription.to_fluent_string(),
                DatePicker::new(&self.localized_picker).cleanable(true),
            ))
            .child(story_field(
                DatePickerStoryText::CompactFieldLabel.to_fluent_string(),
                DatePickerStoryText::CompactFieldDescription.to_fluent_string(),
                DatePicker::new(&self.compact_picker)
                    .cleanable(true)
                    .number_of_months(1)
                    .appearance(false),
            ));

        story_panel(
            DatePickerStoryText::PanelTitle.to_fluent_string(),
            DatePickerStoryText::PanelDescription.to_fluent_string(),
            div().flex().flex_col().gap_4().child(form).child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .mt_2()
                    .text_sm()
                    .child(
                        DatePickerStoryText::DefaultValue {
                            value: default_value,
                        }
                        .to_fluent_string(),
                    )
                    .child(DatePickerStoryText::LongValue { value: long_value }.to_fluent_string())
                    .child(
                        DatePickerStoryText::FrenchValue {
                            value: localized_value,
                        }
                        .to_fluent_string(),
                    )
                    .child(
                        DatePickerStoryText::CompactValue {
                            value: compact_value,
                        }
                        .to_fluent_string(),
                    )
                    .child(
                        DatePickerStoryText::LastChange {
                            value: self.last_change.to_string(),
                        }
                        .to_fluent_string(),
                    ),
            ),
        )
    }
}

fn describe_date(date: Option<JiffDate>) -> String {
    match date {
        Some(date) => date.to_string(),
        None => DatePickerStoryText::None.to_fluent_string(),
    }
}
