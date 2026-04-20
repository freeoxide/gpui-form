[![Build Status](https://github.com/stayhydated/gpui-form/actions/workflows/ci.yml/badge.svg)](https://github.com/stayhydated/gpui-form/actions/workflows/ci.yml)
[![Docs](https://docs.rs/gpui-form/badge.svg)](https://docs.rs/gpui-form/)
[![Crates.io](https://img.shields.io/crates/v/gpui-form.svg)](https://crates.io/crates/gpui-form)

# gpui-form

A type-safe form-generation ecosystem for `gpui` and
[`gpui-component`](https://github.com/longbridge/gpui-component), centered on
`#[derive(GpuiForm)]`.

## Compatibility

The current workspace version is `0.5.1`.

Current `main` uses:

- `gpui` pinned to revision `15d8660748b508b3525d3403e5d172f1a557bfa5`
- `gpui-component` from its GitHub default branch

If you need crates.io-aligned compatibility guarantees, prefer the matching
release/tag instead of the moving `main` branch.

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

## Supported components

- Input
- Number Input, including `number_input(as = ...)` validation overrides
- Checkbox
- Switch
- Select, including `searchable` and `partial`
- Infinite Select, including `searchable` and `max_depth`
- Date Picker via `gpui_form::runtime::date_picker`
- Custom components via `component(custom(shape = ...))` or
  `component(custom(state = ...))`

## Using custom components

- User-defined components via `component(custom(shape = ...))` and
  `gpui_form::custom_component_shape!`.
- Or derive directly on state types with `#[derive(gpui_form::CustomComponentState)]`
  and use `component(custom(state = ...))`.
- Optional `component = ...` metadata can be attached either on the field or on
  the custom state/shape so prototyping output can emit the concrete widget
  type.
- Runtime helper modules are re-exported from
  `gpui_form::{custom, date_picker, infinite_select}` and also grouped under
  `gpui_form::runtime`.
- Numeric validation helpers are available under `gpui_form::numeric` and
  `gpui_form::core::numeric`.
- Generated value holders with `#[gpui_form(skip)]` fields derive
  `::gpui_form::bon::Builder`; the facade re-exports `bon` so generated code
  has a stable path.
- Direct `gpui-form-component` dependencies are only needed when using the
  runtime implementation crate standalone.

## Workspace layout

- `gpui-form`: facade crate re-exporting `core`, `runtime`, `schema`, and derives.
- `gpui-form-core`: pure helper logic such as numeric validation.
- `gpui-form-schema`: inventory metadata and schema types.
- `gpui-form-derive`: proc macros for forms and select helpers.
- `gpui-form-codegen`: internal parse-time/token-generation support for derives.
- `gpui-form-component`: GPUI-facing runtime helpers, re-exported by the facade
  as `gpui_form::runtime`.
- `gpui-form-prototyping-core`: consumer-facing prototyping/codegen helpers that
  consume `GpuiFormShape` inventory data.

## Validation ([koruma](https://github.com/stayhydated/koruma))

If you annotate fields with Koruma validators, the derive macro mirrors validation
metadata and can surface fluent error labels. The `FormValueHolder` will wrap
component fields in `Option<T>` when needed to support required validations, and
it preserves both shorthand and builder-chain validator attrs when mirroring
them into generated holder types.

## Prototyping

Enable the `inventory` feature on `gpui-form` and use `gpui-form-prototyping-core`
to generate gpui form scaffolding from `GpuiFormShape` registrations.
See `examples/prototyping` for a working generator.

## Examples

- `examples/i18n` - localization resources used by the examples.
- `examples/some-lib` - crate defining shared example types.
- `examples/some-lib-custom-components` - external custom component state
  types and UI widgets used by the examples.
- `examples/some-lib-forms` - storybook-like gpui app showcasing generated forms. Run with `cargo run -p some-lib-forms`.
- `examples/prototyping` - prototyping generator that emits form scaffolding. Run with `cargo run -p prototyping`.
