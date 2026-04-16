[![Build Status](https://github.com/stayhydated/gpui-form/actions/workflows/ci.yml/badge.svg)](https://github.com/stayhydated/gpui-form/actions/workflows/ci.yml)
[![Docs](https://docs.rs/gpui-form/badge.svg)](https://docs.rs/gpui-form/)
[![Crates.io](https://img.shields.io/crates/v/gpui-form.svg)](https://crates.io/crates/gpui-form)

# gpui-form

A struct derive macro for deriving [gpui-component](https://github.com/longbridge/gpui-component)... components on fields.

## Compatibility

Compatibility of `gpui-form` versions:

| `gpui-form` | `gpui-component` | `gpui` |
| :--------------- | :--------------- | :--------------------------------------------- |
| **git** | |
| `master` | `main` | rev `15d8660748b508b3525d3403e5d172f1a557bfa5` |
| **crates.io** | |
| `0.5.x` | `0.5.x` |

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

    #[gpui_form(component(select))]
    pub country: Country,

    #[gpui_form(component(checkbox))]
    pub subscribe: bool,
}
```

## Currently Supported components

- [Checkbox](https://longbridge.github.io/gpui-component/docs/components/checkbox)
- [Date Picker](https://longbridge.github.io/gpui-component/docs/components/date-picker)
- [Select](https://longbridge.github.io/gpui-component/docs/components/select)
- [Input](https://longbridge.github.io/gpui-component/docs/components/input)
- [Number Input](https://longbridge.github.io/gpui-component/docs/components/number-input)
- [Switch](https://longbridge.github.io/gpui-component/docs/components/switch)

## gpui-form components

- [Infinite Select](<>)

## Using custom components

- User-defined components via `component(custom(shape = ...))` and
  `gpui_form::custom_component_shape!`.
- Or derive directly on state types with `#[derive(gpui_form::CustomComponentState)]`
  and use `component(custom(state = ...))`.
- Runtime helper modules are re-exported from `gpui_form::{custom, infinite_select}`.
  A direct `gpui-form-component` dependency is only needed when using that crate standalone.

## Validation ([koruma](https://github.com/stayhydated/koruma))

If you annotate fields with Koruma validators, the derive macro mirrors validation
metadata and can surface fluent error labels. The `FormValueHolder` will wrap
component fields in `Option<T>` when needed to support required validations.

## Prototyping

Enable the `inventory` feature on `gpui-form` and use `gpui-form-prototyping-core`
to generate gpui form scaffolding from `GpuiFormShape` registrations.
See `examples/prototyping` for a working generator.

## Examples

- `examples/i18n` - localization resources used by the examples.
- `examples/some-lib` - crate defining shared example types.
- `examples/some-lib-custom-components` - external custom component shapes/states used by examples.
- `examples/some-lib-forms` - storybook-like gpui app showcasing generated forms. Run with `cargo run -p some-lib-forms`.
- `examples/prototyping` - prototyping generator that emits form scaffolding. Run with `cargo run -p prototyping`.
