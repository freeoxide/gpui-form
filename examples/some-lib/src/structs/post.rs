//! Milestone M3 example shapes (backlog #2 "Nested forms" and #3 "Repeated
//! fields"): a nested form composed by the `component(nested)` attribute and
//! a scalar-item `Vec` list with `each(...)` element validators.

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

/// The outer form: a nested field, a scalar-item list with per-item element
/// validators, and a flat field so mixed shapes stay covered.
#[derive(Clone, Debug, Default, GpuiForm, koruma::Koruma)]
pub struct PostEditor {
    #[gpui_form(component(input))]
    #[koruma(NonEmptyValidation::<_>::builder())]
    pub title: String,

    #[gpui_form(component(list(item = input)))]
    #[koruma(each(LenValidation::<_>::builder().min(2).max(32)))]
    pub tags: Vec<String>,

    #[gpui_form(component(nested))]
    #[koruma(nested)]
    pub address: Address,
}
