# gpui-form Architecture

`gpui-form` is the facade crate and compatibility boundary for the workspace.
Application crates should be able to depend on this crate alone and get the
derive macros, runtime helpers, schema types, and compatibility re-exports they
need.

## Purpose

This crate exists to:

1. present a stable public entry point
1. centralize feature flags that gate proc-macro behavior
1. preserve root-level compatibility re-exports even when lower-level crates
   evolve

## Public Surface

`src/lib.rs` re-exports:

- `gpui_form_derive::*` behind the `derive` feature
- `gpui_form_component` as `gpui_form::runtime`
- `gpui_form_component::custom`
- `gpui_form_component::date_picker`
- `gpui_form_component::infinite_select`
- `gpui_form_core` as `gpui_form::core`
- `gpui_form_core::numeric`
- `gpui_form_schema` as `gpui_form::schema`
- `bon` as `gpui_form::bon`

The explicit namespaces (`core`, `runtime`, `schema`) are the preferred public
paths. Root-level module re-exports remain for compatibility.

## Feature Flags

- `derive` (default): enables proc-macro re-exports from `gpui-form-derive`
- `inventory`: forwards inventory-enabled derive behavior so `#[derive(GpuiForm)]`
  emits `GpuiFormShape` registrations

`inventory` is meaningful only when `derive` is also enabled.

## Dependency Role

`gpui-form` depends on four lower layers:

- `gpui-form-core` for UI-neutral helper logic
- `gpui-form-component` for GPUI runtime helpers
- `gpui-form-schema` for metadata and inventory types
- `gpui-form-derive` for proc macros

This crate should stay thin. New behavior normally belongs in one of those
lower crates and is only re-exported here.

## Control Flow

### Normal derive-driven form generation

1. A user depends on `gpui-form` with the `derive` feature.
1. The user derives `GpuiForm`, `SelectItem`, `InfiniteSelect`, or
   `CustomComponentState` through the facade.
1. Macro expansion emits code that references `gpui_form::runtime`,
   `gpui_form::schema`, and `gpui_form::core`.
1. Generated code uses compatibility re-exports only where existing generated
   paths must stay stable.

### Prototyping flow

1. The user enables `gpui-form`'s `inventory` feature.
1. `#[derive(GpuiForm)]` emits `GpuiFormShape` registrations through the facade
   path.
1. Downstream tooling iterates `gpui_form::schema::registry::inventory`.
1. `gpui-form-prototyping-core` converts those shapes into scaffolded GPUI code.

## Compatibility Notes

- `gpui_form::bon` is re-exported because generated value holders with skipped
  fields derive `::gpui_form::bon::Builder`
- root-level compatibility modules (`custom`, `date_picker`,
  `infinite_select`, `numeric`) should not be removed casually
- if a lower-level crate adds a new public runtime/type surface that should be
  first-class for end users, it usually needs a facade re-export here

## When To Update This Document

Update this file when:

- the facade re-export layout changes
- a new feature flag is introduced or forwarded
- generated code changes which facade paths it targets
- compatibility guarantees for root-level re-exports change
