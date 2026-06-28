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
