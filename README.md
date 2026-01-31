[![Build Status](https://github.com/stayhydated/gpui-form/actions/workflows/ci.yml/badge.svg)](https://github.com/stayhydated/gpui-form/actions/workflows/ci.yml)
[![Docs](https://docs.rs/gpui-form/badge.svg)](https://docs.rs/gpui-form/)
[![Crates.io](https://img.shields.io/crates/v/gpui-form.svg)](https://crates.io/crates/gpui-form)

# gpui-form

Derive macros and helpers for building `gpui-component` forms from Rust structs, with optional validation and localization.

This framework gives you:

- `#[derive(GpuiForm)]` to generate form state, value holders, and metadata.
- `#[derive(SelectItem)]` for select dropdowns (optional fluent labels).
- `#[derive(InfiniteSelect)]` for cascading enum selects.
- Optional Koruma validation wiring (including fluent error labels).
- Inventory-based shape registry for prototyping codegen.

## Crates

- `gpui-form`: user-facing facade crate.
- `gpui-form-derive`: proc macros (`GpuiForm`, `SelectItem`, `InfiniteSelect`).
- `gpui-form-core`: component definitions, registry, and helpers.
- `gpui-form-component`: runtime helpers (InfiniteSelect).
- `gpui-form-prototyping-core`: codegen utilities for prototyping.
- `gpui-form-internal-macros`: internal proc macros used by core.

## Installation

Add the facade crate and enable the features you need:

```toml
[dependencies]
# gpui + gpui-component are required by generated code

gpui = { git = "https://github.com/zed-industries/zed" }
gpui-component = { git = "https://github.com/longbridge/gpui-component" }

# Derive macros (default)
gpui-form = { version = "*", features = ["derive"] }

# Optional: runtime components like InfiniteSelect
# gpui-form = { version = "*", features = ["derive", "component"] }

# Optional: inventory registry for prototyping
# gpui-form = { version = "*", features = ["derive", "inventory"] }
```

Note: `component` and `inventory` are only effective when `derive` is enabled.

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

## Supported components

- Input
- Number Input
- Checkbox
- Switch
- Select
- Date Picker
- Infinite Select (via `gpui-form-component`)

## Infinite Select

Use `#[derive(InfiniteSelect)]` to build cascading selects from nested enums:

```rs
use gpui_form::InfiniteSelect;

#[derive(Clone, Debug, Default, InfiniteSelect)]
pub enum Country {
    #[default]
    USA(USAState),
    Canada(CanadaProvince),
    UK,
}

#[derive(Clone, Debug, Default, InfiniteSelect)]
pub enum USAState {
    #[default]
    California,
    Texas,
}
```

Then use `#[gpui_form(component(infinite_select))]` on the field.

## Validation (Koruma)

If you annotate fields with Koruma validators, the derive macro mirrors validation
metadata and can surface fluent error labels. The `FormValueHolder` will wrap
component fields in `Option<T>` when needed to support required validations.

## Prototyping

Enable the `inventory` feature on `gpui-form` and use `gpui-form-prototyping-core`
to generate gpui form scaffolding from `GpuiFormShape` registrations.
See `examples/prototyping` for a working generator.

## Examples

- `examples/some-lib-forms` - gpui app showcasing generated forms.
- `examples/prototyping` - codegen from inventory.

## Compatibility

Compatibility of `gpui-form` versions:

| `gpui-form` | `gpui-component` |
| :------------ | :--------------- |
| **git** | |
| `master` | `main` |
| **crates.io** | |
| `0.5.x` | `0.5.x` |
