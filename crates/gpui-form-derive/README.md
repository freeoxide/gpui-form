# gpui-form-derive

Procedural macros for the `gpui-form` ecosystem.

Most users should depend on [`gpui-form`](../gpui-form/README.md) with the
default `derive` feature instead of using this crate directly.

## Macros

### `#[derive(GpuiForm)]`

Turns a struct into generated form state, helper constructors, and a typed value
holder.

```rs
use gpui_form::GpuiForm;

#[derive(Clone, Debug, Default, GpuiForm)]
pub struct UserProfile {
    #[gpui_form(component(input))]
    pub username: Option<String>,

    #[gpui_form(component(number_input))]
    pub age: Option<u32>,
}
```

Supported component forms:

- `#[gpui_form(component(input))]`
- `#[gpui_form(component(number_input))]`
- `#[gpui_form(component(number_input(as = f64)))]`
- `#[gpui_form(component(checkbox))]`
- `#[gpui_form(component(switch))]`
- `#[gpui_form(component(select))]`
- `#[gpui_form(component(select(searchable)))]`
- `#[gpui_form(component(select(partial)))]`
- `#[gpui_form(component(infinite_select))]`
- `#[gpui_form(component(infinite_select(searchable, max_depth = 3)))]`
- `#[gpui_form(component(date_picker))]`
- `#[gpui_form(component(custom(shape = my::Shape)))]`
- `#[gpui_form(component(custom(state = my::State)))]`
- `#[gpui_form(component(custom(shape = my::Shape, component = my::ui::Widget)))]`
- `#[gpui_form(component(custom(shape = my::Shape, wraps_in_option = false)))]`

Supporting field attributes:

- `#[gpui_form(default = <expr>)]`
- `#[gpui_form(skip)]`
- `#[gpui_form(type = <form_type>)]`
- `#[gpui_form(from = <expr>)]`
- `#[gpui_form(into = <expr>)]`

Supporting struct attributes:

- `#[gpui_form(empty)]`
- `#[gpui_form(koruma)]`
- `#[gpui_form(koruma(fluent))]`

Behavior notes:

- `select` expects enum-like values that can populate a `gpui_component` select
- `default = ...` also drives the initial selection for `select` and
  `infinite_select`
- `custom(..., wraps_in_option = false)` keeps the generated value-holder field
  as `T` instead of `Option<T>`
- `type`/`from`/`into` let the generated holder edit a type that differs from
  the original model field
- when skipped fields are present, the generated value holder keeps builder
  support and exposes `into_original(...)` instead of an unconditional reverse
  conversion

### `#[derive(SelectItem)]`

Implements `gpui_component::select::SelectItem` for enums.

```rs
use gpui_form::SelectItem;

#[derive(Clone, Debug, SelectItem)]
pub enum Country {
    USA,
    France,
}
```

Optional attribute:

- `#[select_item(fluent)]` uses `es-fluent` for titles

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

- derived enums expose both index paths (`selection_path()`) and key paths
  (`selection_key_path()`)
- root option titles use `variant_label()` when fluent label metadata is
  available, otherwise they fall back to the variant name
- key-based helpers such as `variant_key()`, `set_child_by_key(...)`, and
  `set_child_by_key_path(...)` let callers persist selections without relying on
  enum ordering
- `InfiniteSelectKeyPath` also supports `Display`, `FromStr`, and serde string
  round-trips for persisted paths
- runtime path helpers now return `InfiniteSelectPathError` so invalid persisted
  paths report the failing depth and bad key/index instead of just returning
  `None`

### `#[derive(CustomComponentState)]`

Implements `gpui_form::custom::CustomComponentShape` directly for a state type.

```rs
use gpui_form::CustomComponentState;

#[derive(Clone, Debug, CustomComponentState)]
#[gpui_form_custom(
    new = crate::state::build,
    component = crate::ui::TagsInput
)]
pub struct TagsState;
```

By default, the generated implementation calls `Self::new(window, cx)`.

## Feature Flags

- `inventory`: enables `GpuiFormShape` registration for `#[derive(GpuiForm)]`

## Most Users Should Use Instead

- [`gpui-form`](../gpui-form/README.md) for the facade
- [`gpui-form-schema`](../gpui-form-schema/README.md) when you need runtime
  metadata rather than proc-macro expansion
