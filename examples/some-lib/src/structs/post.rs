//! Example shapes: a nested form composed via `component(nested)` and a
//! scalar-item `Vec` list with `each(...)` element validators.

use gpui_form::GpuiForm;
use koruma_collection::collection::{LenValidation, NonEmptyValidation};

/// The nested form: derives `GpuiForm` itself, so `AddressFormValueHolder`
/// and `AddressFormPath` exist for the outer form to compose with.
#[derive(Clone, Debug, Default, GpuiForm, koruma::Koruma)]
pub struct Address {
    #[gpui_form(component(input))]
    #[koruma(NonEmptyValidation::<_>::builder())]
    pub city: String,

    #[gpui_form(component(input))]
    pub zip: Option<String>,
}

#[derive(Clone, Debug, Default, GpuiForm, koruma::Koruma)]
pub struct PostEditor {
    #[gpui_form(component(input))]
    #[koruma(NonEmptyValidation::<_>::builder())]
    pub title: String,

    #[gpui_form(component(list(item = input, add_label = "Add another tag")))]
    #[koruma(each(LenValidation::<_>::builder().min(2).max(32)))]
    pub tags: Vec<String>,

    #[gpui_form(component(nested))]
    #[koruma(nested)]
    pub address: Address,
}
