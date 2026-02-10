mod derives;

use proc_macro::TokenStream;
use proc_macro_error2::proc_macro_error;

use crate::derives::gpui_form::GpuiFormOptions;

#[proc_macro_derive(GpuiForm, attributes(gpui_form))]
#[proc_macro_error]
pub fn gpui_form_derive(input: TokenStream) -> TokenStream {
    derives::gpui_form::from(
        input,
        GpuiFormOptions {
            generate_shape: cfg!(feature = "inventory"),
        },
    )
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

/// Derive macro for infinite select enums to expose their inner values.
///
/// This macro generates an implementation of `InfiniteSelect` trait which allows
/// accessing the inner value of tuple variants and enables cascading select behavior.
///
/// # Example
///
/// ```ignore
/// #[derive(InfiniteSelect)]
/// enum Country {
///     USA(USAState),
///     Canada(CanadaProvince),
///     UK,  // Unit variants are also supported
/// }
/// ```
///
/// All tuple variants must have the same inner type. Unit variants are allowed
/// and will return `None` from `inner()`.
#[proc_macro_derive(InfiniteSelect, attributes(tuple_enum))]
#[proc_macro_error]
pub fn derive_infinite_select(input: TokenStream) -> TokenStream {
    derives::infinite_select::from(input)
}
