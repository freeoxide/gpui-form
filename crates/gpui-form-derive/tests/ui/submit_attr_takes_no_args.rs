// `submit` is a bare word: `#[gpui_form(submit = foo)]` must be rejected
// at parse time by darling.
use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
#[gpui_form(submit = foo)]
struct Form {
    #[gpui_form(component(input))]
    name: String,
}

fn main() {}
