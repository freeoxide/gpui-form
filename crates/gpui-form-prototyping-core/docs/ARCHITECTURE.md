# gpui-form-prototyping-core Architecture

`gpui-form-prototyping-core` turns `GpuiFormShape` inventory metadata into
scaffolded GPUI form code.

It is intentionally downstream from the derive system: it consumes the metadata
model produced elsewhere rather than reparsing source structs.

## Purpose

This crate exists to:

1. validate `GpuiFormShape` metadata before generation
1. compute reusable field fragments and imports
1. let callers render complete files through a pluggable layout

## Module Layout

- `src/lib.rs`: public exports
- `src/code_gen.rs`: `FormShapeAdapter`, `FormParts`, and `FormLayout`
- `src/imports.rs`: import tracking and grouped rendering
- `src/error.rs`: structured prototyping errors
- `src/implementations/`: per-component field generators

## Generation Pipeline

1. A caller iterates `inventory::iter::<GpuiFormShape>()`.
1. The caller constructs `FormShapeAdapter::new(shape)`.
1. The adapter validates and resolves the raw shape into typed field analysis.
1. Component-specific generators produce cached render fragments, event
   handlers, imports, subscriptions, and initialization code.
   Infinite-select fields are generated against one runtime
   `InfiniteSelectState` subscription instead of raw select-tree glue code,
   render through `InfiniteSelectState::levels()`, and consume the richer
   `InfiniteSelectEvent<T>` payload directly.
1. The adapter returns:
   - `FormParts` for caller-controlled assembly, or
   - a complete `syn::File` through `generate_file(&impl FormLayout)`
1. The caller formats and writes the resulting file.

## `FormParts` Role

`FormParts` is the boundary between metadata analysis and layout rendering.

It contains:

- stable identifiers
- deduplicated imports
- component creation tokens
- event/subscription/init tokens
- render fragments
- debug helpers
- flags such as `is_empty`, `has_koruma`, and `has_skipped_fields`

This allows callers to define different layout styles without reimplementing
field analysis.

## Import Strategy

Imports are tracked close to where they are needed:

- fragment-level imports live in the core generation layer
- component-specific imports live in each field generator
- `ImportSet` deduplicates and groups them into compact `use` statements

This avoids layouts having to rediscover which imports the generated fragments
need.

## Field Resolution

The generator parses `FieldVariant::value_type` as a full Rust type, not just a
bare identifier. That is important because inventory metadata may carry:

- crate-qualified enum paths
- nested module paths
- type overrides emitted by the derive layer

Each field is resolved once into a typed internal representation before any
component-specific generation runs.

## Current Custom Component Behavior

For `ComponentsBehaviour::Custom`:

- the generator still initializes custom state into generated `FormFields`
- if `FieldVariant::custom_component` is present, the generator can emit
  `Component::new(&entity)` and import the component type
- if that metadata is missing, the generator falls back to a placeholder row
- custom subscriptions are still project-specific and are not inferred

## Coordination Rules

When adding or changing a component:

1. update the field generator mapping under `src/implementations/`
1. consume any new `ComponentsBehaviour` payloads
1. update imports for newly referenced runtime types
1. verify the example generator under `examples/prototyping`

## When To Update This Document

Update this file when:

- the adapter/layout boundary changes
- `FormParts` fields change meaning
- custom component generation behavior changes
- import handling or field-resolution strategy changes
