# gpui-form-schema Architecture

`gpui-form-schema` owns the shared metadata model used by macro expansion,
inventory registration, and downstream prototyping.

## Purpose

This crate is the runtime-safe metadata boundary between:

- derive/codegen internals
- the user-facing facade
- downstream tooling such as `gpui-form-prototyping-core`

It should describe component behavior and discovered form shape information, but
not own proc-macro parsing or token emission.

## Modules

- `src/lib.rs`: exports `components`, `layout`, and `registry`
- `src/components.rs`: component identity and behavior payload types
- `src/layout.rs`: non-rendering layout hints (`FieldLayout`, `LayoutWidth`)
- `src/registry.rs`: `GpuiFormShape`, `FieldVariant`, and `inventory`
  collection

## Metadata Model

### `ComponentKind`

Static identity for built-in component categories. It centralizes shared traits
such as:

- snake-case component naming
- whether a component is subscribable
- whether a component is focusable
- whether generated holder fields wrap in `Option<T>` by default

### `ComponentsBehaviour`

Per-field runtime behavior metadata with payloads for:

- select behavior
- infinite-select behavior
- number-input behavior
- phone-input behavior (`PhoneInputBehaviour`, carrying the optional
  `country_field` reference for `phone_input(country = ...)`)
- file-picker behavior

This is the metadata level that downstream consumers use; derive/codegen
internals should not invent separate parallel runtime models.

### `GpuiFormShape`

Inventory-registered description of one derived form source struct.

Important fields:

- `struct_name`
- `components`
- `source_path`
- `koruma_enabled`
- `has_skipped_fields`

### `FieldVariant`

Per-field metadata for one generated component entry.

Important fields:

- `field_name`
- `value_type`
- `source_value_type`
- `optional`
- `wraps_in_option`
- `behaviour`
- `validations`
- `default_expr`
- `from_expr`
- `into_expr`
- `custom_shape`
- `custom_component`
- `custom_value_binding`
- `layout` (`FieldLayout`): non-rendering hints — `section`, `label`,
  `description`, `placeholder` (all `Option<&'static str>`), and `width`
  (`LayoutWidth`)

### `FieldLayout` and `LayoutWidth`

Non-rendering layout hints attached to each `FieldVariant` (METADATA-FIRST v1,
backlog feature #4). Key properties:

- **Metadata-only.** Nothing here describes how a field is rendered, painted,
  or laid out by GPUI. Consumers (prototyping generators, application code)
  decide how to interpret each hint.
- **`&'static str` boundary.** All string hints are `Option<&'static str>`
  because the derive emits them as string literals in the user crate (matching
  `default_expr`/`from_expr`).
- **Const-constructible.** Both `FieldLayout` and `LayoutWidth` are
  `const`-constructible (all builders are `const fn`) so the derive can build
  them inside `inventory::submit! { ... }` blocks via
  `FieldVariant::new(...).with_layout(FieldLayout::new()...)`.
- **Order-preserving sections.** `section` grouping is order-preserving:
  consecutive fields with the same section form one group; consumers must not
  reorder fields across the form.
- **Defaults.** `LayoutWidth` defaults to `Full`; an all-`None`/`Full` layout
  is `is_empty()` so consumers can skip emitting anything for it. `label`
  defaults to the field name at consumption time (not in the schema).
- **Serialization.** `LayoutWidth` derives `Display`/`EnumString` with
  `#[strum(serialize_all = "snake_case")]` so `width = half` round-trips
  consistently with `ComponentKind`.

## Data Flow

1. `gpui-form-codegen` parses field component syntax and turns it into typed
   component definitions.
1. `gpui-form-codegen` emits `ComponentsBehaviour` tokens for each field.
1. `gpui-form-derive` embeds those behavior tokens into generated
   `FieldVariant` metadata, alongside a `FieldLayout` built from the field's
   layout hints (`section`/`label`/`description`/`placeholder`/`width`).
1. When inventory registration is enabled, `gpui-form-derive` submits a
   `GpuiFormShape`.
1. `gpui-form-prototyping-core` reads that metadata (including layout hints)
   and generates scaffolded code.

## Boundary Rules

This crate should own:

- runtime-safe metadata
- shared component identity
- inventory registration types

This crate should not own:

- parsing of `#[gpui_form(...)]` attributes
- proc-macro error diagnostics
- direct GPUI runtime implementations

## Coordination Rules

When adding or changing a component:

1. update `ComponentsBehaviour` and related payload structs here
1. update `gpui-form-codegen` so the derive layer emits the new metadata
1. update `gpui-form-prototyping-core` so the generator consumes the new
   behavior correctly
1. update `gpui-form-component` if runtime support is required

## When To Update This Document

Update this file when:

- `GpuiFormShape` or `FieldVariant` fields change
- `FieldLayout` / `LayoutWidth` shape, builders, or serialization change
- component behavior payloads change
- inventory ownership or registration semantics change
