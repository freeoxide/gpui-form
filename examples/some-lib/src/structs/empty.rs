use es_fluent::EsFluentThis;
use gpui_form::GpuiForm;

#[derive(Clone, Debug, Default, EsFluentThis, GpuiForm)]
#[fluent_this(origin)]
#[gpui_form(empty)]
pub struct Empty;
