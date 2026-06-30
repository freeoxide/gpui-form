#[cfg(feature = "derive")]
pub use gpui_form_component::InfiniteSelect;
#[cfg(feature = "derive")]
pub use gpui_form_derive::{CustomComponentState, GpuiForm, SelectItem};

pub use gpui_form_component as runtime;
pub use gpui_form_component::custom;
pub use gpui_form_component::custom::CustomComponentShape;
pub use gpui_form_component::custom_component_shape;
pub use gpui_form_component::date_picker;
pub use gpui_form_component::file_picker;
pub use gpui_form_component::i18n;
pub use gpui_form_component::infinite_select;
pub use gpui_form_core as core;
pub use gpui_form_core::FieldPath;
pub use gpui_form_core::FormState;
pub use gpui_form_core::numeric;
pub use gpui_form_core::path;
#[cfg(feature = "phone")]
pub use gpui_form_core::phone;
pub use gpui_form_core::state;
pub use gpui_form_schema as schema;
// Ergonomic root re-export of the width hint enum. `FieldLayout` itself stays
// under `gpui_form::schema` (it is field-level metadata consumed by generators
// and tooling, parallel to `FieldVariant`), but `LayoutWidth` is the simple
// enum an application reaches for when building layouts by hand.
pub use gpui_form_schema::LayoutWidth;

pub use bon;
