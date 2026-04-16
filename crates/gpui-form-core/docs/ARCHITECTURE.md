# gpui-form-core Architecture

## Purpose

`gpui-form-core` hosts pure helper logic that generated forms can depend on
without pulling in GPUI runtime types.

## Key modules

- `src/numeric.rs`: text-input validation helpers for signed and unsigned
  numeric entry.

## Data flow

1. `gpui-form-codegen` emits number-input validation calls against
   `gpui_form::numeric::*`.
1. The facade crate re-exports `gpui-form-core::numeric` at
   `gpui_form::numeric` and the crate itself as `gpui_form::core`.
1. Generated number-input code uses these helpers while editing form values.

## Notes

- This crate intentionally stays UI-neutral and does not depend on `gpui`.
- Runtime traits/helpers live in `gpui-form-runtime`.
- Static metadata lives in `gpui-form-schema`.
