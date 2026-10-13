//! Generator for the per-form typed path type (`<Name>FormPath`).
//!
//! Backlog feature #8 ("Typed field paths and field IDs", FLAT v1) emits a
//! strongly-typed newtype wrapper around [`::gpui_form::core::FieldPath`] for
//! every `#[derive(GpuiForm)]` form, so validation, dirty tracking, focus,
//! analytics, and schema export share ONE typed way to name fields instead of
//! ad-hoc strings.
//!
//! # Scope
//!
//! Milestone M3 (backlog #2 "Nested forms" and #3 "Repeated fields", decision
//! [D8]) adds typed composition on top of the flat constructors:
//!
//! - nested fields emit `<field>_root()` plus a consuming appender
//!   `fn <field>(self)`;
//! - list fields emit `<field>_root()` plus `fn <field>(index: usize)`
//!   carrying a typed [`::gpui_form::core::PathSegment::Index`] segment;
//! - every form gains `join` for cross-form composition
//!   (`UserFormPath::address_root().join(&AddressFormPath::city())`).
//!
//! # Rules
//!
//! - `<Name>FormPath` carries **no generics**, even for generic source
//!   structs, because a path only names fields — it stores no typed values.
//! - Skipped fields are EXCLUDED from the constructors (they are absent from
//!   the value holder too).
//! - If a form has zero non-skipped fields, the type is still emitted with
//!   `new()` / `path()` / `into_path()` and no per-field constructors.
//! - The generated code reaches the shared primitive via the facade path
//!   `::gpui_form::core::FieldPath`, mirroring how `value_holder.rs` reaches
//!   `::gpui_form::bon`.
//! - `FieldPath` is a headless primitive (no GPUI, no serde). It is NOT behind
//!   a feature flag, so this generator emits NO `#[cfg(...)]` attrs and NO
//!   serde derives.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::DeriveInput;

use crate::derives::gpui_form::structs::{FieldComposition, FieldOptionality};

/// Generates the `<Name>FormPath` newtype plus its impls for a single form.
///
/// The returned tokens are spliced into the final derive output next to the
/// value-holder tokens. They always emit a path type — even for a form with
/// zero non-skipped fields — per the feature #8 contract.
pub fn generate_field_path(
    original_input: &DeriveInput,
    fields: &[FieldOptionality],
) -> TokenStream {
    let form_path_ident = format_ident!("{}FormPath", original_input.ident);

    // One freestanding `pub fn <field>() -> Self` constructor per non-skipped
    // FLAT field. Named identically to the field, callable as
    // `<Name>FormPath::<field>()`. `stringify!` yields the field ident as a
    // `&'static str`, matching the call shape `FieldPath::new(&[&'static str])`.
    //
    // Milestone M3 composition (backlog #2/#3, decision [D8]):
    // - nested fields emit `<field>_root()` (the field itself) plus a
    //   consuming appender `fn <field>(self)` that pushes the field name
    //   onto any path;
    // - list fields emit `<field>_root()` plus `fn <field>(index: usize)`
    //   producing `[Name("<field>"), Index(index)]` — a typed index segment,
    //   never a formatted string;
    // - every form gains `join` for cross-form composition:
    //   `UserFormPath::address_root().join(&AddressFormPath::city())`
    //   -> `address.city`.
    let constructors: Vec<TokenStream> = fields
        .iter()
        .filter(|f| !f.skip)
        .map(|f| {
            let field_name = &f.field_name;
            match &f.composition {
                FieldComposition::Nested { .. } => {
                    let root_ident = format_ident!("{}_root", field_name);
                    quote! {
                        /// Root path naming this nested field itself.
                        pub fn #root_ident() -> Self {
                            Self(::gpui_form::core::FieldPath::new(&[stringify!(#field_name)]))
                        }

                        /// Consuming appender: push this nested field's name onto an existing path.
                        pub fn #field_name(self) -> Self {
                            Self(self.into_path().push_name(stringify!(#field_name)))
                        }
                    }
                },
                FieldComposition::List { .. } => {
                    let root_ident = format_ident!("{}_root", field_name);
                    quote! {
                        /// Root path naming this list field itself.
                        pub fn #root_ident() -> Self {
                            Self(::gpui_form::core::FieldPath::new(&[stringify!(#field_name)]))
                        }

                        /// Path naming one list item via a typed index segment (never a formatted string).
                        pub fn #field_name(index: usize) -> Self {
                            Self(::gpui_form::core::FieldPath::from_segments(&[
                                ::gpui_form::core::PathSegment::Name(stringify!(#field_name)),
                                ::gpui_form::core::PathSegment::Index(index),
                            ]))
                        }
                    }
                },
                FieldComposition::Flat => {
                    quote! {
                        /// Build a path naming this field.
                        pub fn #field_name() -> Self {
                            Self(::gpui_form::core::FieldPath::new(&[stringify!(#field_name)]))
                        }
                    }
                },
            }
        })
        .collect();

    // Feature #8 FLAT v1 + field-level state (M1): the declaration-order name
    // table and dense index mapping that touched/dirty/error state keys on.
    // Skipped fields are absent (they have no constructor and no holder slot).
    let non_skipped: Vec<&FieldOptionality> = fields.iter().filter(|f| !f.skip).collect();
    let field_name_literals: Vec<String> = non_skipped
        .iter()
        .map(|f| f.field_name.to_string())
        .collect();
    let field_count = non_skipped.len();
    let path_for_index_arms: Vec<TokenStream> = non_skipped
        .iter()
        .enumerate()
        .map(|(index, f)| {
            let field_name = &f.field_name;
            // Composed fields have no zero-argument constructor — their
            // root constructor names the field for dense-index state.
            let root_ctor = if f.composition.is_flat() {
                field_name.clone()
            } else {
                format_ident!("{}_root", field_name)
            };
            let index_token = proc_macro2::Literal::usize_unsuffixed(index);
            quote! {
                #index_token => Some(Self::#root_ctor()),
            }
        })
        .collect();

    // Generated-item docs stay one-line summaries (the derive output is
    // ~45% #[doc] payload by bytes otherwise); the removed essays live in
    // these generator comments:
    // - `<Name>FormPath` exists since backlog feature #8 (FLAT v1); each
    //   constructor names a single non-skipped field, with typed nested and
    //   list-item paths added by backlog features #2/#3 (M3, decision [D8]).
    // - `field_index`: dense declaration-order index for single-segment root
    //   paths naming a non-skipped field (flat constructors and nested/list
    //   `_root()` constructors). `None` for hand-built multi-segment paths
    //   and list-item paths — item-level touched tracking has no dense index
    //   in v1.
    // - `new`: mirrors `::gpui_form::core::FieldPath::new`; useful for
    //   hand-built multi-segment paths — prefer the per-field constructors,
    //   the `_root()` constructors, and `join`.
    // - `join`: `UserFormPath::address_root().join(&AddressFormPath::city())`
    //   names `address.city`; a list item composes the same way
    //   (`PostEditorFormPath::contacts(2).join(&ContactFormPath::name())`
    //   names `contacts.2.name`). The result keeps THIS form's type, so
    //   wrong-form composition still cannot mix path types at the roots.
    quote! {
        /// Typed path to a field on this form, generated by `#[derive(GpuiForm)]` (backlog feature #8).
        #[derive(Clone, Debug, Eq, PartialEq, ::core::hash::Hash)]
        pub struct #form_path_ident(::gpui_form::core::FieldPath);

        impl #form_path_ident {
            /// All non-skipped field names in declaration order.
            pub const FIELD_NAMES: &'static [&'static str] = &[#(#field_name_literals),*];

            /// Number of non-skipped fields; the dense index space used by field-level state (touched/dirty/errors).
            pub const FIELD_COUNT: usize = #field_count;

            /// Every field path in declaration order.
            pub fn all_paths() -> impl Iterator<Item = Self> {
                (0..Self::FIELD_COUNT)
                    .map(|index| Self::path_for_index(index).expect("index < FIELD_COUNT"))
            }

            /// Dense declaration-order index for a single-segment root path; `None` for multi-segment or list-item paths.
            pub fn field_index(&self) -> Option<usize> {
                match self.path().segments() {
                    [::gpui_form::core::PathSegment::Name(name)] => Self::FIELD_NAMES
                        .iter()
                        .copied()
                        .position(|candidate| candidate == *name),
                    _ => None,
                }
            }

            /// Inverse of [`Self::field_index`]: the path for a dense index, or `None` when `index >= FIELD_COUNT`.
            pub fn path_for_index(index: usize) -> Option<Self> {
                match index {
                    #(#path_for_index_arms)*
                    _ => None,
                }
            }

            /// Build a path from an explicit slice of static segments (mirrors [`::gpui_form::core::FieldPath::new`]).
            pub fn new(segments: &[&'static str]) -> Self {
                Self(::gpui_form::core::FieldPath::new(segments))
            }

            /// Compose this path with any other form's path: append all of `other`'s segments, keeping this form's type.
            pub fn join(
                self,
                other: &impl ::core::convert::AsRef<::gpui_form::core::FieldPath>,
            ) -> Self {
                Self(self.into_path().join(other.as_ref()))
            }

            #(#constructors)*

            /// Borrow the underlying shared [`::gpui_form::core::FieldPath`].
            pub fn path(&self) -> &::gpui_form::core::FieldPath {
                &self.0
            }

            /// Consume this typed path and return the underlying [`::gpui_form::core::FieldPath`].
            pub fn into_path(self) -> ::gpui_form::core::FieldPath {
                self.0
            }
        }

        impl ::core::fmt::Display for #form_path_ident {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::core::fmt::Display::fmt(&self.0, f)
            }
        }

        impl ::core::convert::AsRef<::gpui_form::core::FieldPath> for #form_path_ident {
            fn as_ref(&self) -> &::gpui_form::core::FieldPath {
                &self.0
            }
        }

        impl ::core::ops::Deref for #form_path_ident {
            type Target = ::gpui_form::core::FieldPath;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    }
}
