use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct Tags {
    #[gpui_form(component(list(item = custom(shape = crate::TagsShape))))]
    tags: Vec<String>,
}

fn main() {}
