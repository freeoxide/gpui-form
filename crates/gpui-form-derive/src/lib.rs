mod derives;

use proc_macro::TokenStream;

use crate::derives::gpui_form::GpuiFormOptions;

#[proc_macro_derive(GpuiForm, attributes(gpui_form, koruma))]
pub fn gpui_form_derive(input: TokenStream) -> TokenStream {
    derives::gpui_form::from(
        input,
        GpuiFormOptions {
            generate_shape: cfg!(feature = "inventory"),
        },
    )
}

#[proc_macro_derive(SelectItem, attributes(select_item))]
pub fn derive_select_item_for_ftl_enum(input: TokenStream) -> TokenStream {
    derives::select_item::from(input)
}

/// Derive macro for custom component state types used by `component(custom(...))`.
///
/// By default it calls `Self::new(window, cx)`. Override the constructor with:
/// `#[gpui_form_custom(new = path::to::constructor)]`.
#[proc_macro_derive(CustomComponentState, attributes(gpui_form_custom))]
pub fn derive_custom_component_state(input: TokenStream) -> TokenStream {
    derives::custom_component_state::from(input)
}
