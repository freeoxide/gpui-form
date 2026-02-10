use gpui_form::{GpuiForm, custom_component_shape};

custom_component_shape!(
    pub ExternalTagInputsComponent,
    state = some_lib_custom_components::ExternalTagInputsState,
    new = some_lib_custom_components::ExternalTagInputsState::new,
);

/// Demonstrates the external-type case:
/// local declarative shape in this crate wrapping external state type.
#[derive(Clone, Debug, Default, GpuiForm)]
pub struct ExternalShapeVecStringInputList {
    #[gpui_form(component(custom(
        shape = ExternalTagInputsComponent,
        wraps_in_option = false
    )))]
    pub tags: Vec<String>,
}
