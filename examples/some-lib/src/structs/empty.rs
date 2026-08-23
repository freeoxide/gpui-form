use es_fluent::EsFluentLabel;
use gpui_form::GpuiForm;

#[derive(Clone, Debug, Default, EsFluentLabel, GpuiForm)]
#[fluent_label(origin)]
#[gpui_form(empty)]
pub struct Empty;
