# gpui-form-component

Runtime helpers for gpui-form components.

This crate provides:

- InfiniteSelect support for cascading selects over nested enums.
- Custom component shape helpers used by `#[derive(GpuiForm)]`.

## InfiniteSelect

`#[derive(InfiniteSelect)]` is provided by `gpui-form-derive` (re-exported by `gpui-form`).
This crate provides the runtime trait and helpers.

```rs
use gpui_form::InfiniteSelect;
use gpui_form_component::infinite_select::{InfiniteSelectPath, build_from_path};

#[derive(Clone, Debug, Default, InfiniteSelect)]
pub enum Country {
    #[default]
    USA(USAState),
    Canada(CanadaProvince),
    UK,
}
```

The generated form code uses:

- `InfiniteSelectItem<T>` for select dropdown items.
- `InfiniteSelectPath` to track selection depth.
- `build_from_path` to reconstruct values from a selection path.

Generated code references this crate directly. Add `gpui-form-component` as a
dependency when using runtime component helpers.

## Custom component shapes

Use `custom_component_shape!` to define a shape consumed by:
`#[gpui_form(component(custom(shape = ...)))]`.

```rs
gpui_form_component::custom_component_shape!(
    pub EmailInputShape,
    state = gpui_component::input::InputState,
    new = gpui_component::input::InputState::new,
);
```

You can also derive directly on a state type with
`#[derive(gpui_form::CustomComponentState)]` and use
`#[gpui_form(component(custom(state = ...)))]`.
