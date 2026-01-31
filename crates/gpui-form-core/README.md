# gpui-form-core

Shared types and helpers used by gpui-form derives and prototyping utilities.

This crate is typically consumed indirectly via `gpui-form`. You may use it directly
if you are extending components or building custom tooling on top of the form metadata.

## What it provides

- `Components` and option structs for component configuration.
- `ComponentsBehaviour` for runtime behavior metadata.
- `GpuiFormShape` and `FieldVariant` for inventory-driven introspection.
- Token generation helpers used by proc macros.

## When to use directly

- Adding a new component type.
- Building your own codegen tooling around `GpuiFormShape`.
- Inspecting form metadata at build time.
