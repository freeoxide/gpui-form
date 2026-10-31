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

    let _: AlphaFormPath = beta;
    let _ = alpha == beta;
}
