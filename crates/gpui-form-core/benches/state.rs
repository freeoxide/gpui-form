use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use gpui_form_core::{
    FieldPath,
    numeric::{validate_signed_numeric, validate_unsigned_numeric},
};
use gpui_form_derive::GpuiForm;

#[derive(Clone, Debug, Default, GpuiForm, koruma::Koruma)]
#[gpui_form(koruma)]
struct StateBenchForm {
    #[gpui_form(component(input))]
    #[koruma(koruma_collection::collection::LenValidation::<_>::builder().min(2).max(64))]
    username: String,

    #[gpui_form(component(input))]
    #[koruma(koruma_collection::format::EmailValidation::<_>::builder())]
    email: Option<String>,

    #[gpui_form(component(number_input))]
    #[koruma(koruma_collection::numeric::RangeValidation::<_>::builder().min(1).max(130))]
    age: u32,

    #[gpui_form(component(number_input))]
    #[koruma(koruma_collection::numeric::RangeValidation::<_>::builder().min(0.0).max(1_000_000.0))]
    balance: f64,

    #[gpui_form(component(input))]
    #[koruma(koruma_collection::collection::LenValidation::<_>::builder().min(8).max(128))]
    passphrase: String,

    #[gpui_form(component(input))]
    display_name: Option<String>,

    #[gpui_form(component(checkbox))]
    newsletter: bool,

    #[gpui_form(component(switch))]
    notifications: bool,

    #[gpui_form(component(input))]
    #[koruma(koruma_collection::collection::NonEmptyValidation::<_>::builder())]
    backup_contact: String,

    #[gpui_form(component(number_input))]
    #[koruma(koruma_collection::numeric::RangeValidation::<_>::builder().min(10_000).max(99_999))]
    postal_code: u32,
}

fn valid_holder() -> StateBenchFormFormValueHolder {
    StateBenchFormFormValueHolder {
        username: Some("ada.lovelace".to_string()),
        email: Some("ada@example.com".to_string()),
        age: Some(36),
        balance: Some(1_024.5),
        passphrase: Some("correct horse battery staple".to_string()),
        display_name: Some("Ada".to_string()),
        newsletter: true,
        notifications: false,
        backup_contact: Some("ops@example.com".to_string()),
        postal_code: Some(94_110),
    }
}

fn all_field_paths() -> [StateBenchFormFormPath; 10] {
    [
        StateBenchFormFormPath::username(),
        StateBenchFormFormPath::email(),
        StateBenchFormFormPath::age(),
        StateBenchFormFormPath::balance(),
        StateBenchFormFormPath::passphrase(),
        StateBenchFormFormPath::display_name(),
        StateBenchFormFormPath::newsletter(),
        StateBenchFormFormPath::notifications(),
        StateBenchFormFormPath::backup_contact(),
        StateBenchFormFormPath::postal_code(),
    ]
}

fn bench_validate_fields(c: &mut Criterion) {
    let valid = valid_holder();
    let invalid = StateBenchFormFormValueHolder::default();

    c.bench_function("state/validate_fields/valid_10_field_koruma", |b| {
        b.iter(|| black_box(&valid).validate_fields())
    });
    c.bench_function("state/validate_fields/invalid_10_field_koruma", |b| {
        b.iter(|| black_box(&invalid).validate_fields())
    });
}

fn bench_error_for_round_trip(c: &mut Criterion) {
    let paths = all_field_paths();
    let valid = valid_holder();
    let invalid = StateBenchFormFormValueHolder::default();

    let round_trip = |holder: &StateBenchFormFormValueHolder| {
        let errors = black_box(holder).validate_fields();
        let mut hits = 0usize;
        for path in &paths {
            if errors.error_for(black_box(path)).is_some() {
                hits = hits.wrapping_add(1);
            }
        }
        hits
    };

    c.bench_function("state/validate_fields_error_for_round_trip/valid", |b| {
        b.iter(|| round_trip(&valid))
    });
    c.bench_function("state/validate_fields_error_for_round_trip/invalid", |b| {
        b.iter(|| round_trip(&invalid))
    });
}

fn bench_form_state_cached_error_for(c: &mut Criterion) {
    let invalid = StateBenchFormFormValueHolder::default();
    let paths = all_field_paths();

    c.bench_function("state/form_state_error_for_cold/invalid", |b| {
        let mut state = StateBenchFormFormState::new(invalid.clone());
        b.iter(|| {
            state.invalidate_validation();
            let mut hits = 0usize;
            for path in &paths {
                if state.error_for(black_box(path)).is_some() {
                    hits = hits.wrapping_add(1);
                }
            }
            hits
        })
    });

    c.bench_function("state/form_state_error_for_warm/invalid", |b| {
        let mut state = StateBenchFormFormState::new(invalid.clone());
        state.validate();
        b.iter(|| {
            let mut hits = 0usize;
            for path in &paths {
                if state.error_for(black_box(path)).is_some() {
                    hits = hits.wrapping_add(1);
                }
            }
            hits
        })
    });
}

const KEYSTROKES: &[&str] = &[
    "", "-", "0.", "1e5", "+3", "12a", "0", "42", "-42", "-0.5", "123.456",
];

fn bench_validate_numeric_keystrokes(c: &mut Criterion) {
    let sweep = |validate: fn(&str, bool) -> bool, require_parse: bool| {
        let mut accepted = 0usize;
        for value in KEYSTROKES {
            if validate(black_box(value), require_parse) {
                accepted = accepted.wrapping_add(1);
            }
        }
        accepted
    };

    c.bench_function("numeric/validate_signed_numeric_i32_keystrokes", |b| {
        b.iter(|| sweep(validate_signed_numeric::<i32>, true))
    });
    c.bench_function("numeric/validate_signed_numeric_f64_keystrokes", |b| {
        b.iter(|| sweep(validate_signed_numeric::<f64>, true))
    });
    c.bench_function(
        "numeric/validate_signed_numeric_f64_keystrokes_no_parse",
        |b| b.iter(|| sweep(validate_signed_numeric::<f64>, false)),
    );
    c.bench_function("numeric/validate_unsigned_numeric_u32_keystrokes", |b| {
        b.iter(|| sweep(validate_unsigned_numeric::<u32>, true))
    });
}

fn bench_field_path_chains(c: &mut Criterion) {
    c.bench_function("field_path/new_push_index_push_name_chain", |b| {
        b.iter(|| {
            let mut segments = 0usize;
            for i in 0..32 {
                let path = FieldPath::new(black_box(&["contacts"]))
                    .push_index(i % 8)
                    .push_name("email");
                segments = segments.wrapping_add(path.segments().len());
            }
            segments
        })
    });

    c.bench_function("field_path/new_push_chain_with_display", |b| {
        b.iter(|| {
            let mut len = 0usize;
            for i in 0..32 {
                let path = FieldPath::new(black_box(&["contacts"]))
                    .push_index(i % 8)
                    .push_name("email");
                len = len.wrapping_add(path.to_string().len());
            }
            len
        })
    });
}

criterion_group!(
    benches,
    bench_validate_fields,
    bench_error_for_round_trip,
    bench_form_state_cached_error_for,
    bench_validate_numeric_keystrokes,
    bench_field_path_chains,
);
criterion_main!(benches);
