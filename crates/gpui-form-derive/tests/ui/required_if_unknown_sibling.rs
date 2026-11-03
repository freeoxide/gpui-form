use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct Form {
    #[gpui_form(component(switch))]
    subscribe: bool,

    #[gpui_form(component(input), required_if = subscribes)]
    email: Option<String>,
}

fn main() {}
