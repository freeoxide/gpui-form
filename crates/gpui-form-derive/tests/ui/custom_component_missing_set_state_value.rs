//! `value_binding` opts a custom component into the generated seed/event
//! path; `CustomComponentValueAdapter::set_state_value` is required, so an
//! impl that omits it must fail to compile (a defaulted no-op would
//! silently drop the form value).

use gpui_form::custom::{
    CustomComponentShape, CustomComponentValueAdapter, CustomComponentValueChange,
};

pub struct EmailShape;

impl CustomComponentShape for EmailShape {
    type State = String;

    fn new(
        _window: &mut ::gpui::Window,
        _cx: &mut ::gpui::Context<'_, Self::State>,
    ) -> Self::State {
        String::new()
    }
}

// Missing `set_state_value`: the value-binding seed path would silently
// drop the form value, so this impl must not compile.
impl CustomComponentValueAdapter<String> for EmailShape {
    type Event = ();

    fn value_change(_state: &String, _event: &()) -> CustomComponentValueChange<String> {
        CustomComponentValueChange::Unchanged
    }
}

fn main() {}
