#[test]
fn gpui_form_reports_invalid_custom_component_arguments() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/custom_component_*.rs");
}

#[test]
fn feature_8_form_path_types_do_not_mix() {
    // Feature #8 (typed field paths): two distinct forms get distinct
    // `<Name>FormPath` newtypes that cannot be assigned to or compared with
    // each other. This is the type-safety guarantee over ad-hoc strings.
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/form_path_types_do_not_mix.rs");
}

#[test]
fn gpui_form_compiles_koruma_builder_attrs_end_to_end() {
    let tests = trybuild::TestCases::new();
    tests.pass("tests/ui/koruma_builder_attrs_pass.rs");
}

#[test]
fn gpui_form_compiles_field_level_state_end_to_end() {
    // Backlog #6/#9 + touched tracking: the generated facade, patch, and
    // typed error surface compile and behave in one binary. No
    // `koruma(fluent)` here, so no trybuild locale seeding is needed.
    let tests = trybuild::TestCases::new();
    tests.pass("tests/ui/error_for_koruma_pass.rs");
}

#[test]
fn gpui_form_compiles_submit_attr_end_to_end() {
    // Backlog #7 (submit lifecycle): `#[gpui_form(submit)]` emits the
    // `<Name>FormSubmission` alias — koruma error type on koruma forms,
    // BoxError otherwise — usable end to end. No `koruma(fluent)` here, so
    // no trybuild locale seeding is needed.
    let tests = trybuild::TestCases::new();
    tests.pass("tests/ui/submit_attr_pass.rs");
}

#[test]
fn submit_attr_takes_no_arguments() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/submit_attr_takes_no_args.rs");
}

#[test]
fn async_validate_attr_rejects_malformed_arguments() {
    // M2 emission sketch: `validate(async_fn = <path>)` parses strictly —
    // non-path literals and argument-less words are darling errors.
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/async_validate_bad_args.rs");
}

#[test]
fn gpui_form_koruma_fluent_compiles_against_rust_i18n_locales() {
    seed_trybuild_locales();

    let tests = trybuild::TestCases::new();
    tests.pass("tests/ui/koruma_fluent_i18n_pass.rs");
}

fn seed_trybuild_locales() {
    use std::fs;
    use std::path::PathBuf;

    let target_dir = resolve_cargo_target_dir();
    let locales_dir = target_dir
        .join("tests")
        .join("trybuild")
        .join("gpui-form-derive")
        .join("locales");
    fs::create_dir_all(&locales_dir).expect("trybuild project locales dir");

    let en = locales_dir.join("en.yml");
    fs::write(
        &en,
        "validation:\n  range: \"Must be in the range %{left_delimiter}%{min}, %{max}%{right_delimiter}.\"\n  invalid: \"Invalid value.\"\nsignup_form:\n  seats_label: \"Seats\"\n  note_label: \"Additional notes\"\nplan_tier:\n  basic: \"Basic plan\"\n  professional: \"Professional plan\"\n",
    )
    .expect("write trybuild locales en.yml");

    let test_source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/ui/koruma_fluent_i18n_pass.rs");
    let file = fs::File::options()
        .append(true)
        .open(&test_source)
        .expect("open ui test source");
    file.set_modified(std::time::SystemTime::now())
        .expect("touch ui test source");
}

fn resolve_cargo_target_dir() -> std::path::PathBuf {
    if let Some(dir) = std::env::var_os("CARGO_TARGET_DIR") {
        return std::path::PathBuf::from(dir);
    }

    let output =
        std::process::Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
            .args(["metadata", "--format-version", "1", "--no-deps"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("run cargo metadata");
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse cargo metadata");
    std::path::PathBuf::from(
        json["target_directory"]
            .as_str()
            .expect("target_directory in cargo metadata"),
    )
}
