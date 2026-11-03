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

impl CustomComponentValueAdapter<String> for EmailShape {
    type Event = ();

    fn value_change(_state: &String, _event: &()) -> CustomComponentValueChange<String> {
        CustomComponentValueChange::Unchanged
    }
}

fn main() {}
