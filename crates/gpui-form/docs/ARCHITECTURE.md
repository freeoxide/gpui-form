# gpui-form Architecture

## Purpose

`gpui-form` is the facade crate. It re-exports derive macros, core types, and custom-component helper APIs. It also hosts shared numeric validation helpers used by generated code.

## Key modules

- `src/lib.rs`: re-exports `gpui-form-core` unconditionally, `gpui-form-derive` behind `derive`, and `gpui-form-component` custom helpers (`CustomComponentShape`, `custom_component_shape!`).
- `src/numeric.rs`: input validation helpers for signed/unsigned numeric text entry.

## Data flow

1. The user derives `GpuiForm`/`SelectItem`/`InfiniteSelect` from this crate (requires the `derive` feature).
1. Optional: users derive `CustomComponentState` for custom component state types.
1. The derive macros (from `gpui-form-derive`) generate types and wiring that reference core metadata in `gpui-form-core`.
1. Custom components can be declared via `custom_component_shape!` and consumed by `component(custom(shape = ...))`.
1. Numeric fields use the helpers in `numeric.rs` for validation in number inputs.

## Feature flags

- `derive` (default): exposes the proc macros.
- `inventory`: forwards to `gpui-form-derive`'s `inventory` feature (effective only when `derive` is enabled).

## Extension points

This crate is intentionally thin. Add new behavior in:

- `gpui-form-core` (component definitions and metadata)
- `gpui-form-derive` (macro expansion)
- `gpui-form-component` (runtime helpers)

## Tests

- Numeric validation tests live in `src/numeric.rs`.
