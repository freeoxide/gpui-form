# gpui-form-prototyping-core Architecture

## Purpose
`gpui-form-prototyping-core` generates gpui form scaffolding from the `GpuiFormShape` inventory. It is intended for rapid prototyping and example generation.

## Key modules
- `code_gen.rs`: adapts `GpuiFormShape` into a `ComponentShape` and orchestrates code generation.
- `implementations/*`: per-component `FieldCodeGenerator` implementations.
- `implementations/mod.rs`: shared traits and helpers for component identity and label/description generation.

## Data flow
1. A consumer (see `examples/prototyping`) iterates over `inventory::iter::<GpuiFormShape>()`.
2. `FormShapeAdapter` maps each `GpuiFormShape` to `FieldCodeGenerator` instances based on `ComponentsBehaviour`.
3. The generator emits token streams for:
   - component construction (`cx.new` calls)
   - field initialization
   - render tree children
   - subscriptions and event handlers
4. The consumer writes the formatted output to disk.

## Feature flags
- `fluent`: when enabled, label/description generation uses `es-fluent` keys.

## Extension points
When adding a component:
1. Add a new `FieldGenerator` variant and map it in `code_gen.rs`.
2. Implement `FieldCodeGenerator` for the new component under `implementations/`.
3. Ensure `ComponentsBehaviour` is handled consistently.
