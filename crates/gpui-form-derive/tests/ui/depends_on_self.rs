// `depends_on` must name a SIBLING field of the same struct — a self-
// reference is a derive error (a field cannot reset itself on its own
// change; M4 backlog #12, the same identity guard `required_if` applies).
use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct Form {
    #[gpui_form(component(input))]
    country: Option<u8>,

    #[gpui_form(component(input), depends_on = state)]
    state: Option<u8>,
}

fn main() {}
