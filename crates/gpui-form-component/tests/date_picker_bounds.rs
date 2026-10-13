//! Behavioral bounds tests for the runtime date pickers: days outside the
//! configured `min_date`/`max_date` render disabled through the calendar's
//! existing disable machinery and reject clicks, while in-bounds days keep
//! selecting exactly as before.
//!
//! The tests render real frames (`Window::draw`, which includes the picker's
//! deferred popup), locate elements through their `debug_selector` hooks, and
//! dispatch real mouse clicks — no gpui-kit test-module dependency.

use gpui::{
    AppContext as _, Context, Entity, IntoElement, Modifiers, Render, TestAppContext, Window,
};
use gpui_form_component::date_picker::{
    DatePicker, DatePickerState, DateRangePicker, DateRangePickerState,
};
use jiff::civil::date;

struct DatePickerView {
    state: Entity<DatePickerState>,
}

impl Render for DatePickerView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        DatePicker::new(&self.state)
    }
}

struct DateRangePickerView {
    state: Entity<DateRangePickerState>,
}

impl Render for DateRangePickerView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        DateRangePicker::new(&self.state)
    }
}

/// One complete frame of the window's root view, including deferred popup
/// elements, so hitboxes and `debug_bounds` reflect the current state.
fn draw_frame(cx: &mut gpui::VisualTestContext) {
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
}

fn day_bounds(cx: &mut gpui::VisualTestContext, day: &'static str) -> gpui::Point<gpui::Pixels> {
    cx.debug_bounds(day)
        .unwrap_or_else(|| panic!("day cell {day} is missing from the rendered frame"))
        .center()
}

fn picker_date(
    cx: &gpui::VisualTestContext,
    view: &Entity<DatePickerView>,
) -> Option<jiff::civil::Date> {
    cx.read(|app| view.read(app).state.read(app).date())
}

fn picker_range(
    cx: &gpui::VisualTestContext,
    view: &Entity<DateRangePickerView>,
) -> (Option<jiff::civil::Date>, Option<jiff::civil::Date>) {
    cx.read(|app| view.read(app).state.read(app).range())
}

fn seeded_date_picker(window: &mut Window, cx: &mut Context<DatePickerView>) -> DatePickerView {
    let state = cx.new(|cx| {
        DatePickerState::new(window, cx)
            .min_date(date(2025, 1, 5))
            .max_date(date(2025, 1, 20))
    });
    state.update(cx, |state, cx| {
        state.set_date(date(2025, 1, 10), window, cx);
    });

    DatePickerView { state }
}

fn seeded_date_range_picker(
    window: &mut Window,
    cx: &mut Context<DateRangePickerView>,
) -> DateRangePickerView {
    let state = cx.new(|cx| {
        DateRangePickerState::new(window, cx)
            .min_date(date(2025, 1, 5))
            .max_date(date(2025, 1, 20))
    });
    state.update(cx, |state, cx| {
        state.set_range(date(2025, 1, 10), None, window, cx);
    });

    DateRangePickerView { state }
}

#[gpui::test]
async fn out_of_bounds_days_reject_clicks_and_in_bounds_days_still_select(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);

    let (view, cx) = cx.add_window_view(seeded_date_picker);
    draw_frame(cx);

    // Open the calendar popup; the grid anchors on the seeded January 2025
    // selection.
    let input = cx
        .debug_bounds("gpfd-date-picker-input")
        .expect("the date picker input must be in the frame")
        .center();
    cx.simulate_click(input, Modifiers::none());
    draw_frame(cx);
    day_bounds(cx, "gpfd-day-2025-01-25");

    // Beyond `max_date`: renders disabled, click is rejected.
    let past_max = day_bounds(cx, "gpfd-day-2025-01-25");
    cx.simulate_click(past_max, Modifiers::none());
    assert_eq!(
        picker_date(cx, &view),
        Some(date(2025, 1, 10)),
        "a day past max_date must reject selection"
    );

    // Below `min_date`: renders disabled, click is rejected.
    let before_min = day_bounds(cx, "gpfd-day-2025-01-01");
    cx.simulate_click(before_min, Modifiers::none());
    assert_eq!(
        picker_date(cx, &view),
        Some(date(2025, 1, 10)),
        "a day before min_date must reject selection"
    );

    // An in-bounds day keeps selecting through the existing machinery, and
    // the completed selection closes the popup.
    let inside = day_bounds(cx, "gpfd-day-2025-01-15");
    cx.simulate_click(inside, Modifiers::none());
    assert_eq!(
        picker_date(cx, &view),
        Some(date(2025, 1, 15)),
        "an in-bounds day must keep selecting"
    );
    draw_frame(cx);
    assert!(
        cx.debug_bounds("gpfd-day-2025-01-20").is_none(),
        "the popup must close after a completed selection"
    );
}

#[gpui::test]
async fn range_picker_rejects_out_of_bounds_ends_and_completes_in_bounds_ranges(
    cx: &mut TestAppContext,
) {
    cx.update(gpui_kit::init);

    let (view, cx) = cx.add_window_view(seeded_date_range_picker);
    draw_frame(cx);

    // Open the popup; the grid anchors on the seeded range start.
    let input = cx
        .debug_bounds("gpfd-date-range-picker-input")
        .expect("the date range picker input must be in the frame")
        .center();
    cx.simulate_click(input, Modifiers::none());
    draw_frame(cx);
    day_bounds(cx, "gpfd-day-2025-01-25");

    // The range start (Jan 10) was seeded programmatically; the picker only
    // re-commits once the calendar emits a complete range.
    assert_eq!(
        picker_range(cx, &view),
        (Some(date(2025, 1, 10)), None),
        "the seeded partial range is the picker's value"
    );

    // Start an in-bounds range from the grid; an incomplete range does not
    // commit.
    let start = day_bounds(cx, "gpfd-day-2025-01-06");
    cx.simulate_click(start, Modifiers::none());
    assert_eq!(
        picker_range(cx, &view),
        (Some(date(2025, 1, 10)), None),
        "the range stays uncommitted until complete"
    );

    // A day past max_date cannot become the range end.
    let past_max = day_bounds(cx, "gpfd-day-2025-01-25");
    cx.simulate_click(past_max, Modifiers::none());
    assert_eq!(
        picker_range(cx, &view),
        (Some(date(2025, 1, 10)), None),
        "an out-of-bounds day must not complete the range"
    );

    // An in-bounds day completes the range and commits it.
    let end = day_bounds(cx, "gpfd-day-2025-01-18");
    cx.simulate_click(end, Modifiers::none());
    assert_eq!(
        picker_range(cx, &view),
        (Some(date(2025, 1, 6)), Some(date(2025, 1, 18))),
        "in-bounds endpoints must complete and commit the range"
    );
}
