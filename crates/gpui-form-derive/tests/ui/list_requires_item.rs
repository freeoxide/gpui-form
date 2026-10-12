//! Milestone M3 compile-fail: a bare `component(list)` is rejected — the
//! item component is required (`component(list(item = input))`).

use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct Tags {
    #[gpui_form(component(list))]
    tags: Vec<String>,
}

fn main() {}
