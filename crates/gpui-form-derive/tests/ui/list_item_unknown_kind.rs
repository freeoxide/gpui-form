//! Milestone M3 compile-fail: unknown list item kinds are rejected at parse
//! time (`item = banana`).

use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct Tags {
    #[gpui_form(component(list(item = banana)))]
    tags: Vec<String>,
}

fn main() {}
