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

#[proc_macro_derive(SelectItem)]
#[proc_macro_error]
pub fn derive_select_item_for_ftl_enum(input: TokenStream) -> TokenStream {
    derives::select_item::from(input)
}

/// Derive macro for tuple enums to expose their inner values.
///
/// This macro generates an implementation of `TupleEnumInner` trait which allows
/// accessing the inner value of tuple variants and enables cascading select behavior.
///
/// # Example
///
/// ```ignore
/// #[derive(TupleEnumInner)]
/// enum Country {
///     USA(USAState),
///     Canada(CanadaProvince),
///     UK,  // Unit variants are also supported
/// }
/// ```
///
/// All tuple variants must have the same inner type. Unit variants are allowed
/// and will return `None` from `inner()`.
#[proc_macro_derive(TupleEnumInner, attributes(tuple_enum))]
#[proc_macro_error]
pub fn derive_tuple_enum_inner(input: TokenStream) -> TokenStream {
    derives::tuple_enum_inner::from(input)
}
