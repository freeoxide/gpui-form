use std::cell::RefCell;
use std::collections::HashMap;

use icu_calendar::{Date as IcuDate, Gregorian, types::Weekday};
use icu_datetime::{FixedCalendarDateTimeFormatter, fieldsets};
use icu_locale_core::Locale;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum DateFormatKind {
    DateShort,
    DateMedium,
    DateLong,
    WeekdayShort,
    MonthLong,
    YearMedium,
    DayShort,
}

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

const CACHE_CAPACITY: usize = 64;

thread_local! {
    static FORMATTER_CACHE: RefCell<HashMap<(DateFormatKind, Locale), CachedFormatter>> =
        RefCell::new(HashMap::new());
}

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
