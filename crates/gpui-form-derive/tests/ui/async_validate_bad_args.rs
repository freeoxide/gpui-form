use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct Form {
    #[gpui_form(validate(async_fn = 123), component(input))]
    a: String,

    #[gpui_form(validate(), component(input))]
    b: String,
}

fn main() {}
