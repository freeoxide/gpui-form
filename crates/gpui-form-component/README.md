# gpui-form-component

GPUI-facing runtime helpers for the `gpui-form` ecosystem.

Most applications should use [`gpui-form`](../gpui-form/README.md), which
re-exports this crate as `gpui_form::runtime`. Depend on this crate directly
when you want the runtime implementation layer without the facade.

## What It Provides

- `infinite_select`: runtime traits and helpers for cascading enum selects
- `date_picker`: the localized date-picker wrapper used by generated forms
- `custom`: the runtime contract for user-defined component state

## Infinite Select

`#[derive(InfiniteSelect)]` lives in `gpui-form-derive`; this crate provides the
runtime trait and helper types that generated code targets.

```rs
use gpui_form::InfiniteSelect;
use gpui_form::infinite_select::{InfiniteSelectPath, build_from_path};

#[derive(Clone, Debug, Default, InfiniteSelect)]
pub enum Country {
    #[default]
    USA(USAState),
    Canada(CanadaProvince),
    UK,
}
```

Useful runtime types:

- `InfiniteSelect`
- `InfiniteSelectItem<T>`
- `InfiniteSelectPath`
- `to_select_items::<T>()`
- `build_from_path`

## Date Picker

Generated `component(date_picker)` fields target the runtime picker in this
crate instead of `gpui_component` directly.

Key public types:

- `DatePickerState`
- `DatePicker`
- `DatePickerEvent`
- `DateDisplayStyle`
- `parse_form_date`

The runtime picker emits `Option<jiff::civil::Date>` and handles localized
display formatting with ICU4X/Jiff while generated code keeps conversion into
the final field type separate.

## Custom Components

`custom::CustomComponentShape` is the contract used by
`component(custom(...))`.

You can declare a reusable shape with the helper macro:

```rs
gpui_form::custom_component_shape!(
    pub EmailInputShape,
    state = gpui_component::input::InputState,
    new = gpui_component::input::InputState::new,
    component = gpui_component::input::Input,
);
```

Or derive directly on a state type:

```rs
#[derive(gpui_form::CustomComponentState)]
#[gpui_form_custom(
    new = crate::state::build,
    component = crate::ui::TagsInput
)]
pub struct TagsState;
```

## Most Users Should Use Instead

- [`gpui-form`](../gpui-form/README.md) for the public facade
- [`gpui-form-schema`](../gpui-form-schema/README.md) for metadata and inventory
- [`gpui-form-prototyping-core`](../gpui-form-prototyping-core/README.md) for
  scaffold generation
