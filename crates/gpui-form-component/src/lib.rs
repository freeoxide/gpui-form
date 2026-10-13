//! Runtime GPUI components for gpui-form: custom component contracts,
//! localized date and date-range pickers, a native file picker, and
//! cascading infinite selects over nested enums.

rust_i18n::i18n!("locales", fallback = "en");

#[cfg(feature = "derive")]
pub use gpui_form_component_derive::InfiniteSelect;

/// Runtime traits/macros for user-defined custom components.
pub mod custom;

mod calendar;

mod date_format;

/// Runtime helpers for the localized date picker component.
pub mod date_picker;

/// Runtime helpers for the native file picker component.
pub mod file_picker;

/// i18n adapter used by runtime components and generated code.
pub mod i18n;

/// Runtime helpers for the InfiniteSelect component.
pub mod infinite_select;
