# gpui-form

User-facing facade crate for gpui-form.

## Install

```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed" }
gpui-component = { git = "https://github.com/longbridge/gpui-component" }

gpui-form = { version = "*", features = ["derive"] }

# Optional inventory registry for prototyping
# gpui-form = { version = "*", features = ["derive", "inventory"] }
```

## Quick start

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

    #[gpui_form(component(select)), default = Country::France]
    pub country: Country,

    #[gpui_form(component(checkbox))]
    pub subscribe: bool,
}
```

## Features

- `derive` (default): proc macros for forms and select helpers.
- `inventory`: enables shape registry for prototyping when `derive` is enabled.

## Runtime Helpers

`gpui-form` exposes the public workspace layout directly:

- `gpui_form::core` for pure helper logic such as numeric validation
- `gpui_form::runtime` for GPUI-facing runtime helpers
- `gpui_form::schema` for metadata and inventory registry types

For backward compatibility, the facade also keeps re-exporting
`gpui_form::custom`, `gpui_form::infinite_select`, and `gpui_form::numeric`.

Most consumers only need the facade crate. Add `gpui-form-runtime` directly
when you want the runtime crate standalone, or `gpui-form-component` when you
need the lower-level implementation crate itself.
