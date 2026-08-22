//! Facade crate for deriving and using generated `gpui-form` forms.

#[cfg(feature = "derive")]
pub use gpui_form_derive::GpuiForm;
#[cfg(all(feature = "derive", feature = "mcp"))]
pub use gpui_form_derive::mcp_submit;

pub use gpui_form_core as core;
pub use gpui_form_core::FieldPath;
pub use gpui_form_core::FormState;
pub use gpui_form_core::numeric;
pub use gpui_form_core::path;
#[cfg(feature = "phone")]
pub use gpui_form_core::phone;
pub use gpui_form_core::state;
#[cfg(feature = "mcp")]
pub use gpui_form_mcp as mcp;
#[cfg(feature = "runtime")]
pub use gpui_form_runtime as runtime;
pub use gpui_form_schema as schema;
pub use bon;
#[cfg(feature = "derive")]
#[doc(hidden)]
pub use strum;
