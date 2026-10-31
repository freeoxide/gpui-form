use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct Form {
    #[gpui_form(component(input))]
    country: Option<u8>,

    #[gpui_form(component(input), depends_on = state)]
    state: Option<u8>,
}

fn main() {}
