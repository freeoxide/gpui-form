use es_fluent::EsFluentKv;
use gpui_form::GpuiForm;

#[derive(Clone, Debug, Default, GpuiForm, EsFluentKv)]
#[fluent_kv(this)]
#[gpui_form(empty)]
pub struct Empty;
