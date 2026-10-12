//! Milestone M3 compile-fail: nested item forms are sequenced after v1
//! (backlog #3: "scalar item components before nested item forms"). The
//! attribute is rejected until that milestone ships.

use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct Contact {
    #[gpui_form(component(input))]
    name: String,
}

#[derive(GpuiForm)]
struct PostEditor {
    #[gpui_form(component(list(item = nested)))]
    contacts: Vec<Contact>,
}

fn main() {}
