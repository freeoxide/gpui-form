use es_fluent::{EsFluentThis, EsFluentVariants};
use gpui_form::GpuiForm;

gpui_form::custom_component_shape!(
    pub ExternalTagInputsComponent,
    state = some_lib_custom_components::ExternalTagInputsState,
    component = some_lib_custom_components::ExternalTagsInput,
    new = some_lib_custom_components::ExternalTagInputsState::new,
);

/// Demonstrates the external-type case:
/// local declarative shape in this crate wrapping external state type.
#[derive(Clone, Debug, Default, GpuiForm, EsFluentThis, EsFluentVariants)]
#[fluent_this(origin, members)]
#[fluent_variants(keys = ["description", "label"])]
pub struct ExternalShapeVecStringInputList {
    #[gpui_form(component(custom(
        shape = ExternalTagInputsComponent,
        component = some_lib_custom_components::ExternalTagsInput,
        wraps_in_option = false
    )))]
    pub tags: Vec<String>,
}
