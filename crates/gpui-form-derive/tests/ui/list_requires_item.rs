use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct Tags {
    #[gpui_form(component(list))]
    tags: Vec<String>,
}

fn main() {}
