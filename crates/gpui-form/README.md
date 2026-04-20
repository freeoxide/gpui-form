# gpui-form

The user-facing facade crate for the `gpui-form` workspace.

Most applications should depend on this crate directly.

## Install

```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed", rev = "15d8660748b508b3525d3403e5d172f1a557bfa5" }
gpui-component = { git = "https://github.com/longbridge/gpui-component" }

gpui-form = { version = "*", features = ["derive"] }

# Optional: shape registry for prototyping/code generation
# gpui-form = { version = "*", features = ["derive", "inventory"] }
```

## Quick Start

```rs
use gpui_form::{GpuiForm, SelectItem};
use strum::EnumIter;

#[derive(Clone, Debug, Default, EnumIter, PartialEq, SelectItem)]
pub enum Country {
    #[default]
    UnitedStates,
    France,
    Japan,
}

#[derive(Clone, Debug, Default, GpuiForm)]
pub struct UserProfile {
    #[gpui_form(component(input))]
    pub username: Option<String>,

    #[gpui_form(component(number_input))]
    pub age: Option<u32>,

    #[gpui_form(component(select), default = Country::France)]
    pub country: Country,

    #[gpui_form(component(checkbox))]
    pub subscribe: bool,
}
```

## Features

- `derive` (default): re-exports the proc macros from `gpui-form-derive`
- `inventory`: registers `GpuiFormShape` metadata for prototyping when
  `derive` is enabled

## Component Syntax

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

Supporting attributes:

- `#[gpui_form(default = <expr>)]`
- `#[gpui_form(skip)]`
- `#[gpui_form(type = <form_type>, from = <expr>, into = <expr>)]`
- `#[gpui_form(empty)]`
- `#[gpui_form(koruma)]`
- `#[gpui_form(koruma(fluent))]`

## Runtime Surface

`gpui-form` re-exports the public workspace layers directly:

- `gpui_form::core` for UI-neutral helpers such as numeric input validation
- `gpui_form::runtime` for GPUI-facing runtime helpers
- `gpui_form::schema` for schema metadata and inventory types

Compatibility re-exports are also preserved:

- `gpui_form::custom`
- `gpui_form::date_picker`
- `gpui_form::infinite_select`
- `gpui_form::numeric`
- `gpui_form::CustomComponentShape`
- `gpui_form::custom_component_shape!`
- `gpui_form::bon`

`gpui_form::bon` is intentionally re-exported because generated value holders
with `#[gpui_form(skip)]` fields derive `::gpui_form::bon::Builder`.

## Validation

Koruma attributes on your source struct can be mirrored into the generated value
holder:

```rs
use gpui_form::GpuiForm;
use koruma::{Koruma, KorumaAllFluent};
use koruma_collection::collection::NonEmptyValidation;

#[derive(Clone, Debug, GpuiForm, Koruma, KorumaAllFluent)]
#[gpui_form(koruma(fluent))]
pub struct Signup {
    #[gpui_form(component(input))]
    #[koruma(NonEmptyValidation<_>)]
    pub username: String,
}
```

## Custom Components

You can use either workflow:

- `#[derive(gpui_form::CustomComponentState)]` on a state type, then
  `component(custom(state = ...))`
- `gpui_form::custom_component_shape!`, then `component(custom(shape = ...))`

Optional `component = ...` metadata can live on the field, on the derived state
type, or on the declared shape. Field-level metadata wins.

## Prototyping

With the `inventory` feature enabled, `gpui-form` exposes:

```rs
use gpui_form::schema::registry::{GpuiFormShape, inventory};

for shape in inventory::iter::<GpuiFormShape>() {
    println!("{}", shape.struct_name);
}
```

If you need actual scaffold generation, pair this crate with
[`gpui-form-prototyping-core`](../gpui-form-prototyping-core/README.md).
