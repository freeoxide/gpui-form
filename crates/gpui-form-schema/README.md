# gpui-form-schema

Schema and registry metadata for the `gpui-form` ecosystem.

This crate is typically consumed indirectly via `gpui-form`. You may use it
directly if you are extending components or building custom tooling on top of
the form metadata.

## What it provides

- `ComponentsBehaviour` for runtime behavior metadata.
- `GpuiFormShape` and `FieldVariant` for inventory-driven introspection.
- The schema metadata consumed by the prototyping generator.

## When to use directly

- Building your own codegen tooling around `GpuiFormShape`.
- Inspecting form metadata at build time.
- Working with runtime behavior metadata outside the facade crate.

## Notes

- Parse-time component parsing and token generation live in `gpui-form-codegen`.
- Pure helper logic lives in `gpui-form-core`.
- Runtime traits/helpers live in `gpui-form-runtime`.
