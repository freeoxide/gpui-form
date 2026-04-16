# gpui-form-codegen Architecture

## Purpose

`gpui-form-codegen` owns the parse-time component model and token-generation
helpers used by `gpui-form-derive`.

`gpui-form-schema` now stays focused on runtime/schema metadata (`GpuiFormShape`,
`FieldVariant`, `ComponentKind`, and component behavior descriptors), while this crate handles:

- parsing `#[gpui_form(component(...))]`
- generating `FormFields` / `FormComponents` tokens
- translating parse-time component options into runtime metadata tokens

## Key modules

- `src/components.rs`: parse-time component options, the `Components` enum, and
  helpers for runtime metadata tokens.
- `src/implementations/*`: per-component field-layout emitters used by the
  derive macro.
- `src/names.rs`: helper for generated component field identifiers.

## Data flow

1. `gpui-form-derive` parses field attributes into `gpui_form_codegen::components::Components`.
1. `Components::generate_field_layout(...)` emits the generated `FormFields` and
   `FormComponents` items for each field.
1. `Components::behaviour_tokens(...)` emits `gpui_form::schema::components::*`
   metadata so inventory/prototyping stay aligned with derive behavior.

Static component identity now comes from `gpui_form_schema::components::ComponentKind`
instead of a codegen-local discriminant. This keeps snake-case names and shared
traits like default `wraps_in_option` aligned across codegen and runtime metadata.

For `date_picker`, field state now targets `gpui_form::runtime::date_picker`
instead of `gpui_component::date_picker` directly so display formatting can be
centralized in the runtime layer.

For `select` and `infinite_select`, field defaults are emitted as optional
initial indices. If a default expression does not match any generated option,
the generated code leaves the initial selection unset instead of panicking.

## Notes

- This crate is intentionally proc-macro-adjacent: it depends on `syn`,
  `quote`, and `proc_macro2`, but is not itself a proc-macro crate.
