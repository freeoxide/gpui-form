# gpui-form Architecture

## Purpose

`gpui-form` is the facade crate for the workspace. Applications can depend on
it alone and get:

- pure helper logic from `gpui-form-core`
- GPUI runtime helpers from `gpui-form-component`
- schema/registry metadata from `gpui-form-schema`
- proc macros from `gpui-form-derive`

## Key modules

- `src/lib.rs`
  - Re-exports the core crate as `gpui_form::core`
  - Re-exports `gpui-form-component` as `gpui_form::runtime`
  - Re-exports the schema crate as `gpui_form::schema`
  - Re-exports derive macros when the `derive` feature is enabled
  - Preserves root-level compatibility re-exports for `custom`,
    `date_picker`, `infinite_select`, `CustomComponentShape`,
    `custom_component_shape!`, `numeric`, and `bon`

## Data flow

1. The user derives `GpuiForm`/`SelectItem`/`InfiniteSelect` from this crate (requires the `derive` feature).
1. Optional: users derive `CustomComponentState` for custom component state types.
1. The derive macros (from `gpui-form-derive`) generate types and wiring that reference runtime metadata in `gpui-form-schema`.
1. Custom components can be declared via `custom_component_shape!` and consumed by `component(custom(shape = ...))`.
1. Generated code can target the explicit facade namespaces
   (`gpui_form::runtime`, `gpui_form::schema`, `gpui_form::core`) while older
   root-level helper paths remain available.
1. When `#[gpui_form(skip)]` fields are present, generated value holders derive
   `::gpui_form::bon::Builder`; the facade re-export keeps that generated path
   stable for users.
1. Generated and prototyped date-picker forms consume `gpui_form::runtime::date_picker`, which formats displayed dates via `jiff` + ICU4X while preserving `FromStr`-based conversion into user field types.
1. Numeric fields use helpers from `gpui-form-core`, re-exported as
   `gpui_form::numeric`.

## Feature flags

- `derive` (default): exposes the proc macros.
- `inventory`: forwards to `gpui-form-derive`'s `inventory` feature (effective only when `derive` is enabled).

## Extension points

This crate is intentionally thin. Add new behavior in:

- `gpui-form-core` (pure helper logic)
- `gpui-form-schema` (component definitions and metadata)
- `gpui-form-codegen` (derive-time component parsing and token generation)
- `gpui-form-derive` (macro expansion)
- `gpui-form-component` (runtime helper implementations, also exposed as `gpui_form::runtime`)
