use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use gpui_form_component::date_picker::{DateDisplayStyle, format_display_date};
use icu_locale_core::{Locale, locale};
use jiff::civil::{Date, date};

const LOCALES: [Locale; 3] = [locale!("en"), locale!("fr-FR"), locale!("zh-CN")];

const DATES: [Date; 3] = [date(2025, 1, 15), date(2000, 2, 29), date(2024, 12, 31)];

const STYLES: [DateDisplayStyle; 3] = [
    DateDisplayStyle::Short,
    DateDisplayStyle::Medium,
    DateDisplayStyle::Long,
];

fn bench_single_call(c: &mut Criterion) {
    let d = date(2025, 1, 15);
    let loc = locale!("en");
    c.bench_function("datefmt/format_display_date/en_medium", |b| {
        b.iter(|| {
            format_display_date(
                black_box(d),
                black_box(&loc),
                black_box(DateDisplayStyle::Medium),
            )
        })
    });
}

fn bench_locale_style_sweep(c: &mut Criterion) {
    c.bench_function("datefmt/format_display_date/sweep_3x3x3", |b| {
        b.iter(|| {
            let mut total_len = 0usize;
            for &d in &DATES {
                for loc in &LOCALES {
                    for &style in &STYLES {
                        if let Some(formatted) = format_display_date(d, loc, style) {
                            total_len = total_len.wrapping_add(formatted.len());
                        }
                    }
                }
            }
            total_len
        })
    });
}

criterion_group!(benches, bench_single_call, bench_locale_style_sweep);
criterion_main!(benches);
