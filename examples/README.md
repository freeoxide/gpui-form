# Examples

This directory is the canonical index for runnable `gpui-form` workspace
examples.

## some-lib

Source structs and enums that demonstrate the supported derive surface:

- basic input, number input, checkbox, switch, and select fields
- searchable select, date-picker conversion, and skipped-field workflows
- nested `InfiniteSelect` enums, including index-path and key-path round trips
  plus typed path-error reporting, custom persisted keys, and key-path string
  serialization
- custom component state, local shapes, and cross-crate shapes
- Koruma validation wiring
- newtype-backed numeric validation
- `cfg_attr`-gated derive usage
- empty forms and date conversion

## some-lib-custom-components

External custom component state and UI types used by the example forms.

This crate demonstrates the cross-crate custom-component shape workflow.

## some-lib-forms

Storybook-style GPUI app that renders generated forms around the example types.
The checked-in `location_form` example shows the runtime-owned
`InfiniteSelectState` flow with `form_fields()` instead of manual child-select
rebuilding.
It includes a manual `Feature Audit` story for the recent headless additions:
`FormState` dirty tracking/reset, generated typed field paths, generated holder
debug data, and pointers to the generated User story for layout sections and
`number_input(as = f64)`.
It also includes a manual `Phone Verification` story that proves dynamic
country-driven phone validation in the UI with the `phonenumber` parser rather
than regex-only checks. The story uses the shared `gpui_form::phone` helper
instead of local phone-validation boilerplate, with separate fields for general
global validation and strict selected-country matching.

Run it with:

```sh
cargo run -p some-lib-forms
```

To open the feature audit screen directly:

```sh
cargo run -p some-lib-forms -- "Feature Audit"
```

Type in the username field to see `FormState::is_dirty` flip, then use
`Reset to baseline` and `Mark current clean`. Open the generated `User` story
to test the layout sections plus `number_input(as = f64)` balance/debt fields.

To open the phone validation story directly:

```sh
cargo run -p some-lib-forms -- "Phone Verification"
```

Try `415 555 2671` with `United States`, then switch the country to `France`.
Try `01 42 68 53 00` with `France`, then switch the country to
`United States`.

## gpui-form-component-story

Storybook-style GPUI app for the reusable runtime components themselves:
infinite select, date picker, and file picker.

Run it with:

```sh
cargo run -p gpui-form-component-story
```

## prototyping

Generator example that walks `GpuiFormShape` inventory data and emits scaffolded
form files into `examples/prototyping/output`. Generated Storybook form titles
use the example app's active locale.

Run it with:

```sh
cargo run -p prototyping
```

## i18n

Shared localization assets used by the example crates. Story apps own a small
`i18n` helper around `EmbeddedI18n`; generated/story rendering calls that helper
to pass localized strings explicitly.
