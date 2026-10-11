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
//! `SHAPE` is built from the **same `FieldVariant`/`FieldLayout` construction
//! chain the inventory path builds** — the caller passes the identical tokens
//! into [`generate_schema_export`] and into `inventory::submit!`. The export is
//! a read-only projection over that chain (schema ARCHITECTURE: the layers
//! "should not invent separate parallel runtime models"), so nested/list
//! composition (M3) and dependency metadata (M4) appear in the export with no
//! second emission path to keep in sync.
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
/// `shape_construction` must be the same `GpuiFormShape` construction chain
/// the caller feeds to `inventory::submit!` (single metadata source).
pub fn generate_schema_export(struct_name: &Ident, shape_construction: TokenStream) -> TokenStream {
    let schema_ident = format_ident!("{}FormSchema", struct_name);

    quote! {
        /// Schema-export surface for this form (backlog #14).
        ///
        /// `Self::schema()` returns an owned, serializable projection of this
        /// form's metadata — component kinds and behaviour payloads, value and
        /// conversion types, validation rules, layout hints, nested/list
        /// composition, and field dependencies — for documentation
        /// generation, test fixtures, prototyping tools, and non-GPUI
        /// consumers.
        pub struct #schema_ident;

        impl #schema_ident {
            /// The inventory shape this export projects from — the same
            /// `FieldVariant`/`FieldLayout` chain submitted under the
            /// `inventory` feature (single metadata source).
            const SHAPE: ::gpui_form::schema::registry::GpuiFormShape = #shape_construction;

            /// Export this form's schema.
            ///
            /// Cheap to call repeatedly: it projects the `const` shape chain
            /// into an owned [`::gpui_form::schema::registry::FormSchema`].
            pub fn schema() -> ::gpui_form::schema::registry::FormSchema {
                ::gpui_form::schema::registry::FormSchema::from(&Self::SHAPE)
            }
        }
    }
}
