use es_fluent::EsFluentKv;
use gpui_form::GpuiForm;

#[derive(Clone, Debug, Default, EsFluentKv, GpuiForm)]
#[fluent_kv(this)]
#[gpui_form(empty)]
pub struct Empty;
