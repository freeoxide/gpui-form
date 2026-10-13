//! Generator for the schema-export surface (backlog #14, milestone M5).
//!
//! Emits exactly one item per form:
//!
//! ```ignore
//! pub struct <Name>FormSchema;
//!
//! impl <Name>FormSchema {
//!     const SHAPE: ::gpui_form::schema::registry::GpuiFormShape = /* chain */;
//!     pub fn schema() -> ::gpui_form::schema::registry::FormSchema {
//!         ::gpui_form::schema::registry::FormSchema::from(&Self::SHAPE)
//!     }
//! }
//! ```
//!
//! # Single metadata source
//!
//! `SHAPE` is built from the one `FieldVariant`/`FieldLayout` construction
//! chain per form — the caller passes those identical tokens only here, and
//! the `inventory::submit!` registration references `SHAPE` by name (the
//! macro evaluates its value inside a `static` initializer, so a const path
//! is a legal value). The export is a read-only projection over that chain
//! (schema ARCHITECTURE: the layers "should not invent separate parallel
//! runtime models"), so nested/list composition (M3) and dependency
//! metadata (M4) appear in the export with no second emission path to keep
//! in sync.
//!
//! # Why not gated on the `inventory` feature
//!
//! The export needs only the registry *types* (`::gpui_form::schema::…`),
//! which the facade always exposes; `inventory::submit!` registration stays
//! behind the derive crate's `inventory` feature. Every form therefore gains
//! the accessor, including `#[gpui_form(empty)]` forms (which export an empty
//! field list), and inventory-off builds pay no inventory dependency.
//!
//! # Shape rules
//!
//! - The emitted type is a unit struct with no generics, following the
//!   `<Name>FormPath` no-generics rule for non-holder state (pipeline hazard
//!   3): the shape constant carries no type parameters.
//! - The name follows the reserved `<Name>Form*` prefix convention
//!   (pipeline hazard 12).

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

/// Emits `<Name>FormSchema` with the `schema()` accessor.
///
/// `shape_construction` is the form's ONE `GpuiFormShape` construction
/// chain; the caller's `inventory::submit!` references the emitted
/// `<Name>FormSchema::SHAPE` const instead of re-emitting the chain
/// (single metadata source).
pub fn generate_schema_export(struct_name: &Ident, shape_construction: TokenStream) -> TokenStream {
    let schema_ident = format_ident!("{}FormSchema", struct_name);

    // Generated-item docs stay one-line summaries (the derive output is
    // ~45% #[doc] payload by bytes otherwise); the removed essay lives in
    // this generator comment: `Self::schema()` returns an owned,
    // serializable projection of this form's metadata — component kinds and
    // behaviour payloads, value and conversion types, validation rules,
    // layout hints, nested/list composition, and field dependencies — for
    // documentation generation, test fixtures, prototyping tools, and
    // non-GPUI consumers. `schema()` is cheap to call repeatedly: it
    // projects the `const` shape chain.
    quote! {
        /// Schema-export surface for this form (backlog #14); `Self::schema()` returns an owned, serializable projection of the form's metadata.
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
