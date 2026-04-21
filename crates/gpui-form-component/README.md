# gpui-form-component

GPUI-facing runtime helpers for the `gpui-form` ecosystem.

Most applications should use [`gpui-form`](../gpui-form/README.md), which
re-exports this crate as `gpui_form::runtime`. Depend on this crate directly
when you want the runtime implementation layer without the facade.

## What It Provides

- `infinite_select`: runtime traits and helpers for cascading enum selects
- `date_picker`: localized runtime state and element wrapper for calendar date input
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
- `InfiniteSelectKeyPath`
- `InfiniteSelectPathError`
- `InfiniteSelectState<T>`
- `SearchableInfiniteSelectState<T>`
- `InfiniteSelectEvent<T>`
- `InfiniteSelectLevel<D>`
- `InfiniteSelectSnapshot<T, D>`
- `InfiniteSelectStateOptions`
- `to_select_items::<T>()`
- `path_from_value(&value)`
- `key_path_from_value(&value)`
- `build_from_path`
- `build_from_key_path`

Manual forms can subscribe to one runtime entity instead of rebuilding nested
child selects themselves:

```rs
use gpui_form::infinite_select::{InfiniteSelectEvent, InfiniteSelectState};

let location = cx.new(|cx| {
    InfiniteSelectState::new(Country::default(), window, cx)
});

cx.subscribe_in(
    &location,
    window,
    |_, _, event: &InfiniteSelectEvent<Country>, _, _| {
        let _value = event.value();
        let _path = event.path();
        let _key_path = event.key_path();
        let _changed_depth = event.changed_depth();
    },
);
```

Rendering code can iterate the runtime-owned field levels directly:

```rs
let snapshot = location.read(cx).snapshot();

for level in snapshot.levels() {
    let _label = level.label();
    let _description = level.description();
    let _select = level.select();
}
```

Derived `InfiniteSelect` enums now also expose:

- `variant_label()` for user-facing option titles
- `variant_key()` plus `selection_key_path()` for order-independent paths
- `set_child_by_key(...)` / `set_child_by_key_path(...)` for programmatic updates
- `build_from_path(...)`, `build_from_key_path(...)`, `set_path(...)`, and
  `set_key_path(...)` return `InfiniteSelectPathError` instead of failing
  silently

## Date Picker

This crate provides the localized runtime date-picker used by generated
`component(date_picker)` fields.

```rs
use gpui_form::runtime::date_picker::{
    DateDisplayStyle,
    DatePicker,
    DatePickerEvent,
    DatePickerState,
};
```

Generated forms store `Entity<DatePickerState>`, render `DatePicker`, and
convert emitted `DatePickerEvent::Change` values with `parse_form_date`.
Most application code should still go through [`gpui-form`](../gpui-form/README.md)
instead of depending on this crate directly.

## Storybook Stories

Enable the optional `storybook` feature when you want this crate to register
runtime component demos with `gpui-storybook` and compile its built-in launcher
binary.

```toml
[dependencies]
gpui-form-component = { version = "*", features = ["storybook"] }
gpui-storybook = { git = "https://github.com/stayhydated/gpui-storybook", features = ["macros"] }
```

This currently registers interactive infinite-select and date-picker demos
backed by this crate's runtime helper types.

Launch the crate-local gallery with:

```sh
cargo run -p gpui-form-component --features storybook
```

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
- [`gpui-component`](https://github.com/longbridge/gpui-component) for the
  upstream date-picker widget and other base components
- [`gpui-form-schema`](../gpui-form-schema/README.md) for metadata and inventory
- [`gpui-form-prototyping-core`](../gpui-form-prototyping-core/README.md) for
  scaffold generation
