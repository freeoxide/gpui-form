//! Parity oracle (campaign round 0): characterization tests pinning the
//! CURRENT ICU4X date formatting behavior of the date picker's display path.
//!
//! These tests exist to make every later optimization a visible diff: they
//! must keep passing UNCHANGED across the whole campaign (e.g. when the
//! per-call `FixedCalendarDateTimeFormatter` construction in
//! `format_display_date` gets cached). A differing value after an
//! optimization is a behavior change and must be a deliberate oracle update.
//!
//! Every expected string below was pinned by running the exact
//! `format_display_date` code path (icu_datetime 2.2 / icu_calendar 2.2,
//! compiled CLDR data) against these fixed dates before any optimization
//! work started.

use gpui_form_component::date_picker::{DateDisplayStyle, format_display_date};
use icu_locale_core::{Locale, locale};
use jiff::civil::{Date, date};

/// One locale's pinned outputs: (date, Short, Medium, Long).
type Row = (Date, &'static str, &'static str, &'static str);

fn assert_rows_match_current_icu_output(locale: &Locale, rows: &[Row]) {
    for (d, want_short, want_medium, want_long) in rows {
        let short = format_display_date(*d, locale, DateDisplayStyle::Short)
            .expect("short format must succeed");
        let medium = format_display_date(*d, locale, DateDisplayStyle::Medium)
            .expect("medium format must succeed");
        let long = format_display_date(*d, locale, DateDisplayStyle::Long)
            .expect("long format must succeed");

        assert_eq!(
            short.as_ref(),
            *want_short,
            "Short changed for {locale:?} {d}"
        );
        assert_eq!(
            medium.as_ref(),
            *want_medium,
            "Medium changed for {locale:?} {d}"
        );
        assert_eq!(long.as_ref(), *want_long, "Long changed for {locale:?} {d}");
    }
}

#[test]
fn en_outputs_are_pinned() {
    let en = locale!("en");
    let rows: &[Row] = &[
        (
            date(2025, 1, 15),
            "1/15/25",
            "Jan 15, 2025",
            "January 15, 2025",
        ),
        (date(1970, 1, 1), "1/1/70", "Jan 1, 1970", "January 1, 1970"),
        (
            date(2000, 2, 29),
            "2/29/00",
            "Feb 29, 2000",
            "February 29, 2000",
        ),
        (
            date(2024, 12, 31),
            "12/31/24",
            "Dec 31, 2024",
            "December 31, 2024",
        ),
    ];
    assert_rows_match_current_icu_output(&en, rows);
}

#[test]
fn fr_fr_outputs_are_pinned() {
    let fr_fr = locale!("fr-FR");
    let rows: &[Row] = &[
        (
            date(2025, 1, 15),
            "15/01/2025",
            "15 janv. 2025",
            "15 janvier 2025",
        ),
        (
            date(1970, 1, 1),
            "01/01/1970",
            "1 janv. 1970",
            "1 janvier 1970",
        ),
        (
            date(2000, 2, 29),
            "29/02/2000",
            "29 févr. 2000",
            "29 février 2000",
        ),
        (
            date(2024, 12, 31),
            "31/12/2024",
            "31 déc. 2024",
            "31 décembre 2024",
        ),
    ];
    assert_rows_match_current_icu_output(&fr_fr, rows);
}

#[test]
fn zh_cn_outputs_are_pinned() {
    let zh_cn = locale!("zh-CN");
    let rows: &[Row] = &[
        (
            date(2025, 1, 15),
            "2025/1/15",
            "2025年1月15日",
            "2025年1月15日",
        ),
        (date(1970, 1, 1), "1970/1/1", "1970年1月1日", "1970年1月1日"),
        (
            date(2000, 2, 29),
            "2000/2/29",
            "2000年2月29日",
            "2000年2月29日",
        ),
        (
            date(2024, 12, 31),
            "2024/12/31",
            "2024年12月31日",
            "2024年12月31日",
        ),
    ];
    assert_rows_match_current_icu_output(&zh_cn, rows);
}

#[test]
fn locale_is_explicit_not_global() {
    // The pure path takes the locale as an argument: the same date through
    // two locales differs in one call, with no global state involved.
    let d = date(2025, 1, 15);
    let en = format_display_date(d, &locale!("en"), DateDisplayStyle::Medium).expect("en");
    let fr = format_display_date(d, &locale!("fr-FR"), DateDisplayStyle::Medium).expect("fr");
    assert_ne!(en.as_ref(), fr.as_ref());
    assert_eq!(en.as_ref(), "Jan 15, 2025");
    assert_eq!(fr.as_ref(), "15 janv. 2025");
}
