# gpui-form-prototyping-core Architecture

## Purpose

`gpui-form-prototyping-core` generates gpui form scaffolding from the
`GpuiFormShape` inventory. It is intended for rapid prototyping and example
generation.

## Key modules

- `code_gen.rs`: adapts `GpuiFormShape` into validated form fragments and orchestrates code generation.
  Key public API:
  - `FormShapeAdapter::parts() -> Result<FormParts, PrototypingError>` — all pre-computed fragments + identifiers, or a structured metadata error.
  - `FormShapeAdapter::generate_file(layout: &impl FormLayout) -> Result<syn::File, PrototypingError>` — generate using a caller-supplied layout.
  - `FormLayout` trait — implement to control the entire generated file shape.
  - `FormParts` — all token-stream fragments exposed as named `pub` fields for use in custom layouts.
    Debug fragments include both value-holder state and into-original status. When
    skipped fields prevent full reconstruction, the into-original debug row prints
    a partial JSON payload for present fields using debug-formatted original-side values.
    `FormParts` also exposes `has_koruma` and `has_skipped_fields` flags so layouts can
    branch submit/reset helper generation based on validation and reconstruction capabilities.
- `imports.rs`: `ImportItem`, `ImportSet` — per-item import tracking and grouped `use` statement rendering.
- `implementations/*`: per-component `FieldCodeGenerator` implementations.
- `implementations/mod.rs`: shared traits and helpers for type parsing,
  component identity, and label/description generation.
  `ResolvedField` is the validated, typed view of one `FieldVariant`; component
  generators operate on that instead of reparsing identifiers and types from
  raw strings.

## Data flow

1. A consumer (see `examples/prototyping`) iterates over `inventory::iter::<GpuiFormShape>()`.
1. `FormShapeAdapter::new(shape).generate_file(&layout)` is the high-level entry point — it returns a ready-to-format `syn::File` or a `PrototypingError`.
   Internally it:
   - Derives all identifiers from `GpuiFormShape`.
   - Converts `shape.source_path` to a glob `use` path via `source_path_to_use_path`.
   - Validates shape metadata before token generation so malformed identifiers / types / paths are reported as errors instead of panics.
   - Resolves each field once into a typed `ResolvedField`, then caches per-field imports, render fragments, subscriptions, and initialization tokens in a single analysis pass.
   - Builds the minimal deduplicated import set from those cached field parts plus prototyping-core's own shared fragments.
   - Produces `FormParts` and hands them to the caller-supplied `FormLayout`, which assembles the final `syn::File`.
1. The consumer formats with `prettyplease::unparse` and writes to disk.

Field type handling is based on parsing `FieldVariant::value_type` as a full
Rust type path. The generator no longer assumes field metadata is a bare
identifier, so qualified paths like `some_lib::country::Country` remain intact
in emitted code.

Built-in date-picker scaffolds import `gpui_form::runtime::date_picker::*`
instead of `gpui_component::date_picker::*`. The generated handlers consume
`Option<jiff::civil::Date>` events and use
`gpui_form::runtime::date_picker::parse_form_date` so concrete target types are
inferred from the assignment site instead of being spelled out in the emitted
form code.

## Import design

Imports are declared close to where they are used:

- Fragment-level items live in `code_gen::FRAGMENT_IMPORTS`.
- Component-specific items live in each generator's `generate_imports` implementation.
- `ImportSet` deduplicates and groups items into compact `use parent::{a, b as c};` statements.
- Layout-specific imports remain the caller's responsibility inside `FormLayout`.
- `gpui::Subscription` is only emitted when generated fragments actually produce subscription calls.

## Feature flags

- `fluent`: when enabled, label/description generation uses `es-fluent` keys.

## Extension points

When adding a component:

1. Add a new generator type under `implementations/`.
1. Register it in `implementations::field_generator(...)`.
1. Implement `FieldCodeGenerator` for the new component under `implementations/`.
1. Override `generate_imports` to declare the exact items your generated code references.
1. Ensure `ComponentsBehaviour` payloads are handled consistently.

Current behavior for `ComponentsBehaviour::Custom`:
custom fields are initialized into generated `FormFields`. When
`FieldVariant::custom_component` is present, prototyping emits
`Component::new(&entity)` and imports the component type when needed; when that
metadata is absent, it falls back to a placeholder render row. Prototyping
still does not infer subscriptions for custom state types; projects add those
hooks manually.
