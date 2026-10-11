//! Milestone M3 compile-fail: custom components are app-owned escape
//! hatches, not list items.

use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct Tags {
    #[gpui_form(component(list(item = custom(shape = crate::TagsShape))))]
    tags: Vec<String>,
}

fn main() {}
