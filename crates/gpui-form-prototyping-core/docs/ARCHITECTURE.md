# gpui-form-prototyping-core Architecture

## Purpose

`gpui-form-prototyping-core` generates gpui form scaffolding from the `GpuiFormShape` inventory. It is intended for rapid prototyping and example generation.

## Key modules

- `code_gen.rs`: adapts `GpuiFormShape` into a `ComponentShape` and orchestrates code generation.
  Key public API:
  - `FormShapeAdapter::parts() -> FormParts` — all pre-computed fragments + identifiers.
  - `FormShapeAdapter::generate_file(layout: &impl FormLayout) -> syn::File` — generate using a caller-supplied layout.
  - `FormLayout` trait — implement to control the entire generated file shape.
  - `FormParts` — all token-stream fragments exposed as named `pub` fields for use in custom layouts.
    Debug fragments include both value-holder state and into-original status, with
    explicit "incomplete" messaging when skipped fields prevent full reconstruction.
- `imports.rs`: `ImportItem`, `ImportSet` — per-item import tracking and grouped `use` statement rendering.
- `implementations/*`: per-component `FieldCodeGenerator` implementations.
- `implementations/mod.rs`: shared traits and helpers for component identity and label/description generation.

## Data flow

1. A consumer (see `examples/prototyping`) iterates over `inventory::iter::<GpuiFormShape>()`.
1. `FormShapeAdapter::new(shape).generate_file()` is the single entry point — it returns a ready-to-format `syn::File`.
   Internally it:
   - Derives all identifiers from `GpuiFormShape` (no external `LayoutIdentities` needed).
   - Converts `shape.source_path` to a glob `use` path via `source_path_to_use_path`.
   - Calls `required_imports()` to build the minimal deduplicated import set.
   - Assembles and `quote!`-generates the full form scaffold token stream.
1. The consumer formats with `prettyplease::unparse` and writes to disk.

## Import design

Imports are declared close to where they are used:

- Framework items live in `code_gen::FRAMEWORK_IMPORTS`.
- Component-specific items live in each generator's `generate_imports` implementation.
- `ImportSet` deduplicates and groups items into compact `use parent::{a, b as c};` statements.
- `gpui::Subscription` is only emitted by generators that produce subscription calls.

## Feature flags

- `fluent`: when enabled, label/description generation uses `es-fluent` keys.

## Extension points

When adding a component:

1. Add a new `FieldGenerator` variant and map it in `code_gen.rs`.
1. Implement `FieldCodeGenerator` for the new component under `implementations/`.
1. Override `generate_imports` to declare the exact items your generated code references.
1. Ensure `ComponentsBehaviour` is handled consistently.

Current behavior for `ComponentsBehaviour::Custom`:
custom fields are initialized into generated `FormFields`, and a placeholder
render row is emitted. Prototyping does not infer subscriptions or concrete
widget rendering for custom state types; projects add those hooks manually.
