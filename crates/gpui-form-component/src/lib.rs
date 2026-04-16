//! Runtime helpers for gpui-form components.
//!
//! This crate provides:
//! - `custom` shape contracts/macros for user-defined components
//! - `date_picker` for the localized runtime picker used by generated forms
//! - `infinite_select` for cascading selects over nested enums

/// Runtime traits/macros for user-defined custom components.
pub mod custom;

/// Runtime helpers for the localized date picker component.
pub mod date_picker;

/// Runtime helpers for the InfiniteSelect component.
pub mod infinite_select;
