# gpui-form-prototyping-core Architecture

## Purpose

`gpui-form-prototyping-core` generates gpui form scaffolding from the `GpuiFormShape` inventory. It is intended for rapid prototyping and example generation.

## Key modules

- `code_gen.rs`: adapts `GpuiFormShape` into a `ComponentShape` and orchestrates code generation.
- `imports.rs`: `ImportItem`, `ImportSet` — per-item import tracking and grouped `use` statement rendering.
- `implementations/*`: per-component `FieldCodeGenerator` implementations.
- `implementations/mod.rs`: shared traits and helpers for component identity and label/description generation.

## Data flow

1. A consumer (see `examples/prototyping`) iterates over `inventory::iter::<GpuiFormShape>()`.
1. `FormShapeAdapter` maps each `GpuiFormShape` to `FieldCodeGenerator` instances based on `ComponentsBehaviour`.
1. The generator emits token streams for:
   - component construction (`cx.new` calls)
   - field initialization
   - render tree children
   - subscriptions and event handlers
1. `FormShapeAdapter::required_imports()` collects a minimal, deduplicated import set:
   - `FRAMEWORK_IMPORTS` in `code_gen.rs` covers the universal base (gpui core, form helpers, fluent).
   - Each `FieldCodeGenerator::generate_imports` implementation declares exactly the items it references.
   - `CustomCodeGenerator` additionally emits the component's qualified path when `FieldVariant::custom_component` is a `::` path (bare names come in via the source module glob).
1. The consumer writes the formatted output to disk.

## Import design

Previously `main.rs` emitted one large hardcoded `use` block for every generated file.
Now imports are declared close to where they are used:
- Framework items live in `code_gen::FRAMEWORK_IMPORTS`.
- Component-specific items live in each generator's `generate_imports` implementation.
- `ImportSet` deduplicates and groups items into compact `use parent::{a, b as c};` statements.
- `rust_decimal::Decimal` is only emitted by `NumberInputCodeGenerator` when the field type contains `Decimal`.
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
