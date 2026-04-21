# gpui-form-component-derive

Proc macros for component-oriented `gpui-form` runtime helpers.

Most applications should use [`gpui-form`](../gpui-form/README.md), which
re-exports `#[derive(InfiniteSelect)]`. Depend on this crate directly when you
want the infinite-select derive without the full facade or the form-generation
macros.

## What It Provides

### `#[derive(InfiniteSelect)]`

Implements `gpui_form::infinite_select::InfiniteSelect` for nested enums used
by cascading selects.

```rs
use gpui_form::InfiniteSelect;

#[derive(Clone, Debug, Default, InfiniteSelect)]
pub enum Country {
    #[default]
    USA(USAState),
    Canada(CanadaProvince),
    UK,
}
```

Variant attribute:

- `#[tuple_enum(skip)]` omits a variant from the select tree
- `#[tuple_enum(key = "...")]` overrides the stable persisted key for a variant

Behavior notes:

- derived enums expose stable `variant_key()` values plus `selection_key_path()`
- custom keys are validated for uniqueness at macro-expansion time
- fluent-backed labels still drive `variant_label()` when available

## Most Users Should Use Instead

- [`gpui-form`](../gpui-form/README.md) for the public facade
- [`gpui-form-component`](../gpui-form-component/README.md) for the runtime
  infinite-select helpers targeted by the generated impl
- [`gpui-form-derive`](../gpui-form-derive/README.md) for `GpuiForm`,
  `SelectItem`, and `CustomComponentState`
