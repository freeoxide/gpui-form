# gpui-form

User-facing facade crate for gpui-form.

This crate re-exports:

- `GpuiForm`, `SelectItem`, and `InfiniteSelect` derive macros (behind `derive`).
- `CustomComponentState` derive macro for custom component state types.
- Core metadata types from `gpui-form-core` (always).
- `CustomComponentShape` and `custom_component_shape!` from `gpui-form-component`.

## Install

```toml
[dependencies]
# gpui + gpui-component are required by generated code

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

    #[gpui_form(component(select(default)))]
    pub country: Country,

    #[gpui_form(component(checkbox))]
    pub subscribe: bool,
}
```

## Features

- `derive` (default): proc macros for forms and select helpers.
- `inventory`: enables shape registry for prototyping when `derive` is enabled.
