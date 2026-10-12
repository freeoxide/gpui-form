// The async-validation attribute surface rejects malformed arguments at
// parse time: `async_fn` must be a path (not a literal), and `validate()`
// without `async_fn` is rejected (the field is required).
use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct Form {
    #[gpui_form(validate(async_fn = 123), component(input))]
    a: String,

    #[gpui_form(validate(), component(input))]
    b: String,
}

fn main() {}
