# gpui-form Architecture

`gpui-form` is the facade crate and compatibility boundary for the workspace.
Application crates should be able to depend on this crate alone and get the
derive macros, runtime helpers, schema types, and compatibility re-exports
they need.

## Purpose

This crate exists to:

1. present a stable public entry point
1. centralize public feature flags
1. re-export lower-level crates under consistent paths
1. preserve compatibility re-exports for older generated code and examples

## Public Surface

`src/lib.rs` re-exports:

- `gpui_form_derive::{CustomComponentState, GpuiForm, SelectItem}` behind the
  `derive` feature
- `gpui_form_component::InfiniteSelect` behind the `derive` feature
- `gpui_form_component` as `gpui_form::runtime`
- `gpui_form_component::custom`
- `gpui_form_component::date_picker`
- `gpui_form_component::file_picker`
- `gpui_form_component::infinite_select`
- `gpui_form_core` as `gpui_form::core`
- `gpui_form_core::numeric`
- `gpui_form_core::path` and `gpui_form_core::FieldPath` for the pure,
  GPUI-free, serde-free typed field-path primitive (feature #8, FLAT v1)
- `gpui_form_core::state` and `gpui_form_core::FormState` for pure, GPUI-free
  form-state dirty tracking / reset / diff (feature #1)
- `gpui_form_schema` as `gpui_form::schema`
- `gpui_form_schema::LayoutWidth` as `gpui_form::LayoutWidth` (ergonomic root
  re-export of the width-hint enum; `FieldLayout` itself stays under
  `gpui_form::schema` alongside `FieldVariant` since it is field-level
  metadata consumed by generators)
- `bon` as `gpui_form::bon`

The explicit namespaces (`core`, `runtime`, `schema`) are the preferred public
paths. Root-level module re-exports remain for compatibility.

## Feature Flags

- `derive` (default): enables the public derive surface
  - `GpuiForm`, `SelectItem`, and `CustomComponentState` come from
    `gpui-form-derive`
  - `InfiniteSelect` comes through `gpui-form-component`, which re-exports the
    proc macro from `gpui-form-component-derive`
- `inventory`: forwards inventory-enabled `GpuiForm` behavior so
  `#[derive(GpuiForm)]` emits `GpuiFormShape` registrations
- `serde`: enables form-state persistence and dirty tracking. It pulls in
  `serde` as a direct optional dependency and forwards
  `gpui-form-derive/serde`, which adds `Serialize`, `Deserialize`, and
  `PartialEq` to the generated `...FormValueHolder`. `FormState` itself is
  re-exported unconditionally (it lives in `gpui-form-core` with no feature
  gate); only the holder serde derives require this feature.
- `phone`: forwards `gpui-form-core/phone` and exposes parser-backed,
  country-matching phone validation helpers at `gpui_form::phone`.

`inventory` and `serde` forward to `gpui-form-derive` via the optional
dependency (`gpui-form-derive?/...`) and are therefore meaningful only when
`derive` is also enabled. `phone` is independent of `derive`.

## Dependency Role

`gpui-form` depends on four lower layers:

- `gpui-form-core` for UI-neutral helper logic
- `gpui-form-component` for GPUI runtime helpers and the facade's
  `InfiniteSelect` re-export
- `gpui-form-schema` for metadata and inventory types
- `gpui-form-derive` for `GpuiForm`, `SelectItem`, and `CustomComponentState`

This crate should stay thin. New behavior normally belongs in one of those
lower crates and is only re-exported here.

## Control Flow

### Facade-driven derives

1. A user depends on `gpui-form` with the `derive` feature.
1. `GpuiForm`, `SelectItem`, and `CustomComponentState` resolve to
   `gpui-form-derive`.
1. `InfiniteSelect` resolves through `gpui-form-component` to
   `gpui-form-component-derive`.
1. `GpuiForm` generated code references `gpui_form::runtime`,
   `gpui_form::schema`, `gpui_form::core`, and compatibility re-exports such as
   `gpui_form::numeric`.
1. `InfiniteSelect` generated code targets `gpui_form::infinite_select`.

### Prototyping flow

1. The user enables `gpui-form`'s `inventory` feature.
1. `#[derive(GpuiForm)]` emits `GpuiFormShape` registrations through the facade
   path.
1. Downstream tooling iterates `gpui_form::schema::registry::inventory`.
1. `gpui-form-prototyping-core` converts those shapes into scaffolded GPUI
   code.

## Compatibility Notes

- `gpui_form::bon` is re-exported because generated value holders with skipped
  fields derive `::gpui_form::bon::Builder`
- `gpui_form::FieldPath` and `gpui_form::path` are re-exported because
  generated `<Name>FormPath` types reach the shared primitive via the facade
  path `::gpui_form::core::FieldPath`, mirroring how value holders reach
  `::gpui_form::bon`
- root-level compatibility modules (`custom`, `date_picker`, `file_picker`,
  `infinite_select`, `numeric`) should not be removed casually
- if a lower-level crate adds a new public surface that should be first-class
  for end users, it usually needs a facade re-export here

## When To Update This Document

Update this file when:

- the facade re-export layout changes
- a feature flag is introduced, removed, or re-routed
- generated code changes which facade paths it targets
- compatibility guarantees for root-level re-exports change
