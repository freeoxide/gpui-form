# gpui-form-component-derive

Proc macros for the `InfiniteSelect` runtime surface.

Most applications should use [`gpui-form`](../gpui-form/README.md), which
re-exports `#[derive(InfiniteSelect)]` as `gpui_form::InfiniteSelect`. Use this
crate directly when you want only the infinite-select derive plus the runtime
crate.

## Direct Use

When you use this crate without the facade, make the runtime crate available to
the macro as `gpui_form`:

```toml
[dependencies]
gpui_form = { package = "gpui-form-component", version = "*" }
gpui-form-component-derive = "*"
```

## `#[derive(InfiniteSelect)]`

Implements `gpui_form::infinite_select::InfiniteSelect` for nested enums used
by cascading selects.

```rs
use gpui_form_component_derive::InfiniteSelect;

#[derive(Clone, Debug, Default, InfiniteSelect)]
pub enum Country {
    #[default]
    USA(USAState),
    Canada(CanadaProvince),
    UK,
}
```

Variant attributes:

- `#[tuple_enum(skip)]` omits a variant from the select tree
- `#[tuple_enum(key = "...")]` overrides the stable persisted key for a variant

Behavior notes:

- derived enums expose stable `variant_key()` values plus `selection_key_path()`
- custom keys are validated for uniqueness within the enum
- fluent-backed labels and descriptions are used when the enum carries the
  matching metadata

## Most Users Should Use Instead

- [`gpui-form`](../gpui-form/README.md) for the public facade
- [`gpui-form-component`](../gpui-form-component/README.md) for the runtime
  state helpers targeted by the derive
- [`gpui-form-derive`](../gpui-form-derive/README.md) for `GpuiForm`,
  `SelectItem`, and `CustomComponentState`
