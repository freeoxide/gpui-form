#[test]
fn gpui_form_reports_invalid_custom_component_arguments() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/custom_component_*.rs");
}
