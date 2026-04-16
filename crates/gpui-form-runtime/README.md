# gpui-form-runtime

GPUI-facing runtime traits and helpers for `gpui-form`.

## What it provides

- `custom` helpers for user-defined component shapes.
- `infinite_select` traits and helpers for cascading selects over nested enums.
- Re-exports of `CustomComponentShape` and `custom_component_shape!`.

## Notes

- This crate is the stable runtime namespace for generated code and advanced
  users.
- Lower-level helper implementations currently live in `gpui-form-component`.
- Static metadata lives in `gpui-form-schema`.
- Pure helper logic such as numeric validation lives in `gpui-form-core`.
