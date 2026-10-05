mod derives;

#[cfg(feature = "mcp")]
mod mcp_submit;

use proc_macro::TokenStream;
use proc_macro_error2::proc_macro_error;

use crate::derives::gpui_form::GpuiFormOptions;

#[proc_macro_derive(GpuiForm, attributes(gpui_form, koruma))]
#[proc_macro_error]
pub fn gpui_form_derive(input: TokenStream) -> TokenStream {
    derives::gpui_form::from(
        input,
        GpuiFormOptions {
            generate_shape: cfg!(feature = "inventory"),
            generate_mcp: cfg!(feature = "mcp"),
        },
    )
}

/// Registers an async function as an MCP submit handler for generated forms.
#[cfg(feature = "mcp")]
#[proc_macro_attribute]
pub fn mcp_submit(attr: TokenStream, item: TokenStream) -> TokenStream {
    match mcp_submit::expand(attr.into(), item.into()) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

#[proc_macro_derive(SelectItem, attributes(select_item))]
#[proc_macro_error]
pub fn derive_select_item_for_ftl_enum(input: TokenStream) -> TokenStream {
    derives::select_item::from(input)
}

/// Derive macro for custom component state types used by `component(custom(...))`.
///
/// By default it calls `Self::new(window, cx)`. Override the constructor with:
/// `#[gpui_form_custom(new = path::to::constructor)]`.
#[proc_macro_derive(CustomComponentState, attributes(gpui_form_custom))]
#[proc_macro_error]
pub fn derive_custom_component_state(input: TokenStream) -> TokenStream {
    derives::custom_component_state::from(input)
}
