# gpui-form-derive

Procedural macros for gpui-form.

## Macros

### `#[derive(GpuiForm)]`

Generates form state, component fields, and a value holder.

Field attributes:

- `#[gpui_form(component(input))]`
- `#[gpui_form(component(number_input))]`
- `#[gpui_form(component(number_input(as = f64)))]` for custom numeric types
- `#[gpui_form(component(checkbox))]`
- `#[gpui_form(component(switch))]`
- `#[gpui_form(component(select))]`
- `#[gpui_form(component(select(searchable)))]`
- `#[gpui_form(component(select(partial)))]`
- `#[gpui_form(component(infinite_select))]`
- `#[gpui_form(component(infinite_select(searchable, max_depth = 3)))]`
- `#[gpui_form(component(custom(shape = my::EmailInputShape)))]`
- `#[gpui_form(component(custom(state = my::EmailInputState)))]`
- `#[gpui_form(component(custom(shape = my::EmailInputShape, wraps_in_option = false)))]`
- `#[gpui_form(component(date_picker))]`
- `#[gpui_form(skip)]` to skip a field
- `#[gpui_form(default = <expr>)]` to set default in the value holder

Notes:

- `select` expects `strum::IntoEnumIterator` and `PartialEq`.
- `#[gpui_form(default = <expr>)]` seeds the generated value holder and also drives the initial selection for `select` and `infinite_select`.
- When no field default is provided, generated default state falls back to `Default::default()`.
- `infinite_select(max_depth = ...)` clamps the generated child-select depth.
- `custom(shape = ...)`/`custom(state = ...)` expects the referenced type to implement `gpui_form::custom::CustomComponentShape`.

Struct attributes:

- `#[gpui_form(empty)]` for empty structs
- `#[gpui_form(koruma)]` or `#[gpui_form(koruma(fluent))]` to enable validation wiring

Example:

```rs
use gpui_form::GpuiForm;

#[derive(Clone, Debug, Default, GpuiForm)]
pub struct UserProfile {
    #[gpui_form(component(input))]
    pub username: Option<String>,

    #[gpui_form(component(number_input))]
    pub age: Option<u32>,

    #[gpui_form(component(select)), default = Country::France]
    pub country: Country,
}
```

### `#[derive(SelectItem)]`

Implements `gpui_component::select::SelectItem` for enums.

Requires `Display` (or `es_fluent::ToFluentString` when using `#[select_item(fluent)]`).

```rs
use gpui_form::SelectItem;

#[derive(Clone, Debug, SelectItem)]
#[select_item(fluent)]
pub enum Country {
    USA,
    France,
}
```

### `#[derive(InfiniteSelect)]`

Implements the `InfiniteSelect` trait for nested enums.

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

Variant attributes:

- `#[tuple_enum(skip)]` to omit a variant from the select tree.

### `#[derive(CustomComponentState)]`

Implements `gpui_form::custom::CustomComponentShape` for a state type.

By default, generated code calls `Self::new(window, cx)`.
You can override constructor path:

```rs
#[derive(CustomComponentState)]
#[gpui_form_custom(new = crate::state::build)]
pub struct TagsState;
```
