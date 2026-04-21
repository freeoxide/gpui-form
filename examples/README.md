# Examples

This directory is the canonical index for runnable `gpui-form` workspace
examples.

## some-lib

Source structs and enums that demonstrate the supported derive surface:

- basic input, number input, checkbox, switch, and select fields
- nested `InfiniteSelect` enums, including index-path and key-path round trips
  plus typed path-error reporting
- custom component state and custom shapes
- Koruma validation wiring
- `cfg_attr`-gated derive usage
- empty forms and date conversion

## some-lib-custom-components

External custom component state and UI types used by the example forms.

This crate demonstrates the cross-crate custom-component shape workflow.

## some-lib-forms

Storybook-style GPUI app that renders generated forms around the example types.
The checked-in `location_form` example shows the runtime-owned
`InfiniteSelectState` flow with `levels()` snapshots instead of manual
child-select rebuilding.

Run it with:

```sh
cargo run -p some-lib-forms
```

## prototyping

Generator example that walks `GpuiFormShape` inventory data and emits scaffolded
form files into `examples/prototyping/output`. Infinite-select fields are
emitted against the runtime `InfiniteSelectState` helper, `levels()` rendering,
and a single change event.

Run it with:

```sh
cargo run -p prototyping
```

## i18n

Shared localization assets used by the example crates.
