# gpui-form-schema Architecture

## Purpose

`gpui-form-schema` hosts the shared runtime/schema metadata used by generated code
and the prototyping generator. It no longer owns derive-time token generation.

## Key modules

- `components.rs`: runtime component behavior descriptors such as
  `ComponentsBehaviour`, `SelectBehaviour`, and `NumberInputBehaviour`.
- `registry.rs`: `GpuiFormShape` and `FieldVariant`, plus `inventory`
  collection for prototyping.
  `GpuiFormShape` also carries whether source fields include `#[gpui_form(skip)]`
  so downstream generators can detect incomplete value-holder roundtrips.

## Data flow

1. `gpui-form-derive` emits `ComponentsBehaviour` values into generated
   `FieldVariant` metadata.
1. `FieldVariant` stores the full Rust value type path (`value_type`) plus
   behavior payloads needed by downstream generators.
1. `ComponentsBehaviour` becomes runtime metadata in `FieldVariant` and is stored in `GpuiFormShape`.
   Skip metadata (`has_skipped_fields`) is also propagated into `GpuiFormShape`.
1. `GpuiFormShape` is optionally registered with `inventory` for downstream prototyping codegen.

## Extension points

To add a new component behavior:

1. Extend `ComponentsBehaviour` and any behavior payload structs in `components.rs`.
1. Update `FieldVariant` consumers in `gpui-form-prototyping-core`.
1. Update `gpui-form-codegen` so derive-time parsing emits the new metadata.

Custom user-defined components are represented at runtime by
`ComponentsBehaviour::Custom`, while optional UI-component path metadata lives on
`FieldVariant::custom_component`.

## Notes

- Parse-time component parsing and token generation now live in
  `gpui-form-codegen`.
