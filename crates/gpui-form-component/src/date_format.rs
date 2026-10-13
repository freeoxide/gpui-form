//! Shared ICU4X formatter cache for the runtime date surfaces.
//!
//! `FixedCalendarDateTimeFormatter::try_new` is expensive enough that an open
//! calendar grid rebuilt it ~94x per frame. Formatters are pure functions of
//! `(fieldset kind, locale)`, so built formatters live in a thread-local map
//! keyed by exactly that pair and are only rebuilt on a key miss — that is, on
//! a locale (or fieldset) first use, never while the locale stays stable.
//!
//! The cache is a keyed structure rather than a field on
//! `DatePickerState`/`DateRangePickerState`/`CalendarState` because the same
//! cached path must serve the pickers' `RenderOnce` elements (which hold only
//! shared reads of their state), the calendar grid helpers, and the pure
//! `format_display_date` that benches and the parity oracle call directly.

use std::cell::RefCell;
use std::collections::HashMap;

use icu_calendar::{Date as IcuDate, Gregorian, types::Weekday};
use icu_datetime::{FixedCalendarDateTimeFormatter, fieldsets};
use icu_locale_core::Locale;

/// The ICU4X fieldsets the runtime date surfaces format through.
///
/// Each variant names one call-site fieldset, so the kind plus the locale is
/// the full cache key for the formatter that call site needs.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum DateFormatKind {
    /// `fieldsets::YMD` short — `DateDisplayStyle::Short` picker labels.
    DateShort,
    /// `fieldsets::YMD` medium — `DateDisplayStyle::Medium` picker labels.
    DateMedium,
    /// `fieldsets::YMD` long — `DateDisplayStyle::Long` picker labels.
    DateLong,
    /// `fieldsets::E` short — weekday header cells.
    WeekdayShort,
    /// `fieldsets::M` long — month header labels.
    MonthLong,
    /// `fieldsets::Y` medium — year header and year grid labels.
    YearMedium,
    /// `fieldsets::D` short — day-of-month cells.
    DayShort,
}

/// A built formatter for one cache key.
///
/// The concrete fieldset type differs per kind, so each variant carries the
/// formatter its kind formats through.
enum CachedFormatter {
    Date(FixedCalendarDateTimeFormatter<Gregorian, fieldsets::YMD>),
    Weekday(FixedCalendarDateTimeFormatter<Gregorian, fieldsets::E>),
    Month(FixedCalendarDateTimeFormatter<Gregorian, fieldsets::M>),
    Year(FixedCalendarDateTimeFormatter<Gregorian, fieldsets::Y>),
    Day(FixedCalendarDateTimeFormatter<Gregorian, fieldsets::D>),
}

impl CachedFormatter {
    fn try_new(kind: DateFormatKind, locale: &Locale) -> Option<Self> {
        let prefs = locale.clone().into();
        Some(match kind {
            DateFormatKind::DateShort => Self::Date(
                FixedCalendarDateTimeFormatter::try_new(prefs, fieldsets::YMD::short()).ok()?,
            ),
            DateFormatKind::DateMedium => Self::Date(
                FixedCalendarDateTimeFormatter::try_new(prefs, fieldsets::YMD::medium()).ok()?,
            ),
            DateFormatKind::DateLong => Self::Date(
                FixedCalendarDateTimeFormatter::try_new(prefs, fieldsets::YMD::long()).ok()?,
            ),
            DateFormatKind::WeekdayShort => Self::Weekday(
                FixedCalendarDateTimeFormatter::try_new(prefs, fieldsets::E::short()).ok()?,
            ),
            DateFormatKind::MonthLong => Self::Month(
                FixedCalendarDateTimeFormatter::try_new(prefs, fieldsets::M::long()).ok()?,
            ),
            DateFormatKind::YearMedium => Self::Year(
                FixedCalendarDateTimeFormatter::try_new(prefs, fieldsets::Y::medium()).ok()?,
            ),
            DateFormatKind::DayShort => Self::Day(
                FixedCalendarDateTimeFormatter::try_new(prefs, fieldsets::D::short()).ok()?,
            ),
        })
    }

    fn format_date(&self, date: &IcuDate<Gregorian>) -> Option<String> {
        match self {
            Self::Date(formatter) => Some(formatter.format(date).to_string()),
            Self::Month(formatter) => Some(formatter.format(date).to_string()),
            Self::Year(formatter) => Some(formatter.format(date).to_string()),
            Self::Day(formatter) => Some(formatter.format(date).to_string()),
            Self::Weekday(_) => None,
        }
    }

    fn format_weekday(&self, weekday: Weekday) -> Option<String> {
        match self {
            Self::Weekday(formatter) => Some(formatter.format(&weekday).to_string()),
            _ => None,
        }
    }
}

/// Upper bound on cached `(kind, locale)` entries.
///
/// Seven kinds exist per locale, so this covers the oracle locales plus a
/// handful of runtime locale switches; churn beyond that clears the map to
/// keep long-running locale toggling bounded.
const CACHE_CAPACITY: usize = 64;

thread_local! {
    static FORMATTER_CACHE: RefCell<HashMap<(DateFormatKind, Locale), CachedFormatter>> =
        RefCell::new(HashMap::new());
}

/// Formats a Gregorian date through the cached formatter for `(kind, locale)`.
///
/// Returns `None` when no formatter can be built for the locale; failures are
/// not cached so the fallback paths behave exactly as before the cache.
pub(crate) fn format_date(
    kind: DateFormatKind,
    locale: &Locale,
    date: &IcuDate<Gregorian>,
) -> Option<String> {
    FORMATTER_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        cached_formatter(&mut cache, kind, locale)?.format_date(date)
    })
}

/// Formats a weekday through the cached `fieldsets::E` short formatter.
pub(crate) fn format_weekday(locale: &Locale, weekday: Weekday) -> Option<String> {
    FORMATTER_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        cached_formatter(&mut cache, DateFormatKind::WeekdayShort, locale)?.format_weekday(weekday)
    })
}

fn cached_formatter<'a>(
    cache: &'a mut HashMap<(DateFormatKind, Locale), CachedFormatter>,
    kind: DateFormatKind,
    locale: &Locale,
) -> Option<&'a CachedFormatter> {
    let key = (kind, locale.clone());
    if !cache.contains_key(&key) {
        let formatter = CachedFormatter::try_new(kind, locale)?;
        if cache.len() >= CACHE_CAPACITY {
            cache.clear();
        }
        cache.insert(key, formatter);
    }
    cache.get(&(kind, locale.clone()))
}
