# gpui-form-derive Architecture

`gpui-form-derive` owns the proc-macro entry points for the workspace.

It parses user structs and enums, delegates parse-time component modeling to
`gpui-form-codegen`, and emits the generated types and inventory metadata that
make the rest of the ecosystem work.

## Entry Points

`src/lib.rs` defines:

- `#[derive(GpuiForm)]`
- `#[derive(SelectItem)]`
- `#[derive(InfiniteSelect)]`
- `#[derive(CustomComponentState)]`

## Module Layout

- `src/derives/gpui_form/mod.rs`: `GpuiForm` entry module
- `src/derives/gpui_form/expansion.rs`: top-level `GpuiForm` expansion pipeline
- `src/derives/gpui_form/components.rs`: delegates component fields into
  codegen layouts
- `src/derives/gpui_form/value_holder.rs`: generated holder types, defaults,
  conversion logic, and skip-field handling
- `src/derives/gpui_form/koruma.rs`: Koruma metadata mirroring helpers
- `src/derives/gpui_form/cfg_attr.rs`: `cfg_attr` flattening before parse-time
  inspection
- `src/derives/select_item.rs`: `SelectItem` expansion
- `src/derives/infinite_select.rs`: `InfiniteSelect` expansion
- `src/derives/custom_component_state.rs`: `CustomComponentState` expansion

## `GpuiForm` Expansion Pipeline

1. Parse the input with `syn`.
1. Flatten `cfg_attr` wrappers so downstream parsing sees effective attributes.
1. Parse struct-level and field-level `#[gpui_form(...)]` data with `darling`.
1. Parse Koruma field metadata through `koruma-derive-core`.
1. For each component field, delegate parse-time component handling to
   `gpui-form-codegen`.
1. Emit:
   - `FormFields`
   - `FormComponents`
   - `FormValueHolder`
   - conversions between the original type and the holder
   - optional inventory metadata

## Value Holder Responsibilities

`value_holder.rs` is the densest part of the derive implementation. It owns:

- optionality normalization between model fields and editable form state
- default-value seeding
- `type`/`from`/`into` conversions
- `#[gpui_form(skip)]` reconstruction behavior
- Koruma mirroring for holder validation

Important behaviors:

- originally optional fields stay optional in the holder
- input-style fields usually wrap in `Option<T>` to represent empty UI state
- skipped fields are still prefilled when converting from the original model into
  the holder
- reverse conversion becomes explicit `into_original(...)` when skipped fields
  prevent a fully automatic roundtrip

## Koruma Integration

`GpuiForm` can enable Koruma-aware holder generation even when the source struct
only contains field-level `#[koruma(...)]` attributes.

The derive layer:

- reads normalized validator metadata from `koruma-derive-core`
- mirrors validators into the holder type
- preserves shorthand and builder-chain validator forms
- injects required-value behavior where holder optionality would otherwise lose
  source-model required semantics

## Inventory Integration

When the `inventory` feature is enabled:

1. `GpuiForm` emits one `GpuiFormShape` per derived struct.
1. Each field becomes a `FieldVariant` with behavior metadata from
   `gpui-form-codegen`.
1. metadata includes validation rule identifiers, defaults, full value type
   paths, and skipped-field information for downstream generators.

## Other Derives

### `SelectItem`

- implements `gpui_component::select::SelectItem`
- optionally uses `es-fluent` titles through `#[select_item(fluent)]`

### `InfiniteSelect`

- emits an `InfiniteSelect` impl for nested enums
- supports unit, tuple, and single-field struct variants
- drives the runtime cascading-select API in `gpui-form-component`

### `CustomComponentState`

- emits a `CustomComponentShape` impl directly for a state type
- defaults constructor wiring to `Self::new(window, cx)`
- optionally stores a component path for prototyping output

## Coordination Rules

When adding a component or attribute:

1. update parse-time option support in `gpui-form-codegen`
1. update holder behavior in this crate if optionality or conversion changes
1. update `gpui-form-schema` metadata emission
1. update `gpui-form-prototyping-core` generator support
1. update user-facing docs in the facade README and derive README

## Tests

- targeted `GpuiForm` expansion tests live in
  `src/derives/gpui_form/tests.rs`
- compile-fail UI tests live under `tests/ui`

## When To Update This Document

Update this file when:

- the expansion pipeline changes
- holder conversion behavior changes
- inventory or Koruma emission rules change
- macro responsibilities move between modules
