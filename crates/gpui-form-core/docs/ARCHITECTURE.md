# gpui-form-core Architecture

## Purpose
`gpui-form-core` hosts the shared data model and codegen helpers used by the derive macros and the prototyping generator. It is not a proc-macro crate, but it builds token streams for use by proc macros.

## Key modules
- `components.rs`: component option types, the `Components` enum, and `ComponentsBehaviour` metadata.
- `registry.rs`: `GpuiFormShape` and `FieldVariant`, plus `inventory` collection for prototyping.
- `names.rs`: helper for building component field identifiers.
- `implementations/*`: per-component `ComponentLayout` implementations that emit struct fields and constructor tokens.
- `implementations/__crate_paths/*`: generated crate path shims (do not edit).

## Data flow
1. `gpui-form-derive` parses `#[gpui_form(component(...))]` into `Components`.
2. `ComponentLayout` implementations build the form field structs and component constructor functions.
3. `ComponentsBehaviour` becomes runtime metadata in `FieldVariant` and is stored in `GpuiFormShape`.
4. `GpuiFormShape` is optionally registered with `inventory` for downstream prototyping codegen.

## Extension points
To add a new component:
1. Add a new option type and `Components` enum variant in `components.rs`.
2. Implement `ComponentLayout` in `implementations/`.
3. Extend `ComponentsBehaviour` and any behavior helpers (e.g., `focusable`, `subscribable`).
4. Update the prototyping generator in `gpui-form-prototyping-core`.

## Notes
- This crate depends on `syn`, `quote`, and `proc_macro2` for token construction.
- Keep the `__crate_paths` folder generated (see `just update_crate_paths`).
