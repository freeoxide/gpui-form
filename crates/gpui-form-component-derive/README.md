# gpui-form-component-derive

Proc macros for the `InfiniteSelect` runtime surface.

Most applications should use [`gpui-form`](../gpui-form/README.md), which
re-exports `#[derive(InfiniteSelect)]` as `gpui_form::InfiniteSelect`. Use this
crate directly when you want only the infinite-select derive plus the runtime
crate.

## Direct Use

When you use this crate without the facade, depend on the runtime crate
normally. The macro resolves either `gpui-form` or `gpui-form-component`,
including renamed dependencies:

```toml
[dependencies]
gpui-form-component = "*"
gpui-form-component-derive = "*"
```

## `#[derive(InfiniteSelect)]`

Implements the runtime crate's `InfiniteSelect` contract for nested enums used
by cascading selects.

```rs
use gpui_form_component::infinite_select::{InfiniteSelectPath, build_from_path};
use gpui_form_component_derive::InfiniteSelect;

#[derive(Clone, Debug, Default, InfiniteSelect, PartialEq)]
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
- `#[fluent_kv(keys = ["label", "description"], keys_this)]` is accepted for
  compatibility but emits no localization metadata. Runtime labels use
  variant-name fallbacks; localize them through your app's `rust-i18n` locales.

Behavior notes:

- derived enums must also implement `PartialEq` because the runtime
  `gpui-kit` select compares selected values
- derived enums expose stable `variant_key()` values plus `selection_key_path()`
- custom keys are validated for uniqueness within the enum
- no localization metadata is emitted; runtime trait methods use variant-name
  fallback names, and localized labels are an application concern through
  `rust-i18n`

## Most Users Should Use Instead

- [`gpui-form`](../gpui-form/README.md) for the public facade
- [`gpui-form-component`](../gpui-form-component/README.md) for the runtime
  state helpers targeted by the derive
- [`gpui-form-derive`](../gpui-form-derive/README.md) for `GpuiForm`,
  `SelectItem`, and `CustomComponentState`
