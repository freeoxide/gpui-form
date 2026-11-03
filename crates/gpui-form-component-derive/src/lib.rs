mod infinite_select;

use proc_macro::TokenStream;

/// Derive macro for infinite select enums; emits `gpui_form::infinite_select::InfiniteSelect`
/// for nested enums of unit, single-field tuple, or single-field struct variants.
#[proc_macro_derive(InfiniteSelect, attributes(tuple_enum))]
pub fn derive_infinite_select(input: TokenStream) -> TokenStream {
    infinite_select::from(input)
}
