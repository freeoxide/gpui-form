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
