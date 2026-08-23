// Feature #8: two distinct forms get distinct `<Name>FormPath` types. A
// constructor or value from one form MUST NOT satisfy the other form's path
// type — this is the whole point of typed paths over ad-hoc strings.
//
// This file is a trybuild `compile_fail` case: the lines below must be rejected
// by the compiler, proving the wrappers are non-mixable.
use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct Alpha {
    #[gpui_form(component(input))]
    a: String,
}

#[derive(GpuiForm)]
struct Beta {
    #[gpui_form(component(input))]
    b: String,
}

fn main() {
    let alpha: AlphaFormPath = AlphaFormPath::a();
    let beta: BetaFormPath = BetaFormPath::b();

    // These two lines must FAIL to compile: a value of one form's path type
    // cannot be assigned to the other form's path type, nor can they be
    // compared for equality.
    let _: AlphaFormPath = beta;
    let _ = alpha == beta;
}
