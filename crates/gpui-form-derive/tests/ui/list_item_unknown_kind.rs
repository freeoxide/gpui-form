use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct Tags {
    #[gpui_form(component(list(item = banana)))]
    tags: Vec<String>,
}

fn main() {}
