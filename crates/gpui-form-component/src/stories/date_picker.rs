use gpui::{
    App, AppContext as _, Context, Entity, Focusable, IntoElement, ParentElement as _, Render,
    SharedString, Styled as _, Subscription, Window, div,
};
use gpui_component::form::v_form;
use icu_locale_core::locale;
use jiff::civil::{Date as JiffDate, date};

use crate::date_picker::{DateDisplayStyle, DatePicker, DatePickerEvent, DatePickerState};

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
        "Date Picker".into()
    }

    fn description() -> String {
        "Localized runtime date-picker demo covering empty, prefilled, and styled variants.".into()
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
            last_change: "Interact with a picker to inspect DatePickerEvent::Change output.".into(),
            _subscriptions: subscriptions,
        }
    }

    fn picker_label(&self, picker: &Entity<DatePickerState>) -> &'static str {
        if picker == &self.default_picker {
            "Default"
        } else if picker == &self.long_picker {
            "Long"
        } else if picker == &self.localized_picker {
            "French"
        } else if picker == &self.compact_picker {
            "Compact"
        } else {
            "Unknown"
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
        self.last_change = format!(
            "{} picker changed to {}",
            self.picker_label(picker),
            describe_date(*date)
        )
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
                "Default picker",
                "Starts empty, uses the active locale, and shows the default two-month calendar.",
                DatePicker::new(&self.default_picker)
                    .placeholder("Select a launch date")
                    .cleanable(true),
            ))
            .child(story_field(
                "Long display",
                "Prefilled with DateDisplayStyle::Long so the input renders a fully localized date string.",
                DatePicker::new(&self.long_picker)
                    .cleanable(true)
                    .number_of_months(1),
            ))
            .child(story_field(
                "French locale",
                "Overrides the display locale to French while still emitting the same Jiff date value.",
                DatePicker::new(&self.localized_picker).cleanable(true),
            ))
            .child(story_field(
                "Compact appearance",
                "Uses a short format, a single calendar month, and the borderless input presentation.",
                DatePicker::new(&self.compact_picker)
                    .cleanable(true)
                    .number_of_months(1)
                    .appearance(false),
            ));

        story_panel(
            "Localized date selection",
            "This exercises the runtime wrapper around the calendar component: locale-aware display text, selection state, and emitted change events.",
            div().flex().flex_col().gap_4().child(form).child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .mt_2()
                    .text_sm()
                    .child(format!("Default value: {default_value}"))
                    .child(format!("Long value: {long_value}"))
                    .child(format!("French value: {localized_value}"))
                    .child(format!("Compact value: {compact_value}"))
                    .child(format!("Last change event: {}", self.last_change)),
            ),
        )
    }
}

fn describe_date(date: Option<JiffDate>) -> String {
    match date {
        Some(date) => date.to_string(),
        None => "None".to_string(),
    }
}
