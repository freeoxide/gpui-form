// `required_if` must name a sibling field of the same struct — a typo fails
// at derive time, not as a silently-dead rule (M4, backlog #13, R19).
use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct Form {
    #[gpui_form(component(switch))]
    subscribe: bool,

    #[gpui_form(component(input), required_if = subscribes)]
    email: Option<String>,
}

fn main() {}
