# gpui-form-runtime Architecture

## Purpose

`gpui-form-runtime` provides the stable GPUI-facing runtime surface for
generated code and advanced consumers.

## Structure

- `src/lib.rs`
  - Re-exports `gpui-form-component::custom`
  - Re-exports `gpui-form-component::date_picker`
  - Re-exports `gpui-form-component::infinite_select`
  - Re-exports `CustomComponentShape` and `custom_component_shape!`

## Data flow

1. `gpui-form-derive` and prototyping output generate custom-component,
   localized date-picker, and infinite-select code
   against the facade crate.
1. `gpui-form` re-exports this crate as `gpui_form::runtime`.
1. For backward compatibility, `gpui-form` also re-exports the same runtime
   items at the root (`gpui_form::custom`, `gpui_form::date_picker`,
   `gpui_form::infinite_select`, and related helpers).

## Notes

- The concrete runtime helper implementations currently live in
  `gpui-form-component`.
- This crate exists to make the crate layout and public namespaces match the
  rest of the ecosystem more closely.
