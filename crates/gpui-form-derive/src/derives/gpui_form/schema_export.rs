//! Emits the per-form schema-export surface: `<Name>FormSchema` with a
//! `SHAPE` const built from the caller's one `GpuiFormShape` chain and a
//! `schema()` accessor projecting it (`inventory::submit!` stays feature-gated).

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

/// Emits `<Name>FormSchema` with the `schema()` accessor; `shape_construction`
/// is the form's one `GpuiFormShape` chain, registered by name via
/// `inventory::submit!` (single metadata source).
pub fn generate_schema_export(struct_name: &Ident, shape_construction: TokenStream) -> TokenStream {
    let schema_ident = format_ident!("{}FormSchema", struct_name);

    quote! {
        /// Schema-export surface for this form; `Self::schema()` returns an owned, serializable projection of the form's metadata.
        pub struct #schema_ident;

        impl #schema_ident {
            /// The form's one `GpuiFormShape` chain — the const the `inventory` submission registers.
            const SHAPE: ::gpui_form::schema::registry::GpuiFormShape = #shape_construction;

            /// Export this form's schema as an owned [`::gpui_form::schema::registry::FormSchema`].
            pub fn schema() -> ::gpui_form::schema::registry::FormSchema {
                ::gpui_form::schema::registry::FormSchema::from(&Self::SHAPE)
            }
        }
    }
}
