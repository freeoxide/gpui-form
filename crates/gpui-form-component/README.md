# gpui-form-component

Runtime helpers for gpui-form components.

Currently this crate focuses on InfiniteSelect support for cascading selects over
nested enums.

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

This crate is re-exported by `gpui-form` when `component` and `derive` are enabled.
