//! Runtime helpers for gpui-form components.
//!
//! This crate currently focuses on the `infinite_select` module, which powers
//! cascading selects over nested enums derived with `#[derive(InfiniteSelect)]`.

/// Runtime traits/macros for user-defined custom components.
pub mod custom;

/// Runtime helpers for the InfiniteSelect component.
pub mod infinite_select;
