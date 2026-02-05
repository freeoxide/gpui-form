[![Build Status](https://github.com/stayhydated/gpui-form/actions/workflows/ci.yml/badge.svg)](https://github.com/stayhydated/gpui-form/actions/workflows/ci.yml)
[![Docs](https://docs.rs/gpui-form/badge.svg)](https://docs.rs/gpui-form/)
[![Crates.io](https://img.shields.io/crates/v/gpui-form.svg)](https://crates.io/crates/gpui-form)

# gpui-form

A struct derive macro for deriving [gpui-component](https://github.com/longbridge/gpui-component)... components on fields.

## Compatibility

Compatibility of `gpui-form` versions:

| `gpui-form` | `gpui-component` |
| :------------ | :--------------- |
| **git** | |
| `master` | `main` |
| **crates.io** | |
| `0.6.x` | `0.6.x` |
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

    #[gpui_form(component(select(default)))]
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

## Custom components

- [Infinite Select](<>)

## Validation (Koruma)

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
- `examples/some-lib-forms` - storybook-like gpui app showcasing generated forms. Run with `cargo run -p some-lib-forms`.
- `examples/prototyping` - prototyping generator that emits form scaffolding. Run with `cargo run -p prototyping`.
