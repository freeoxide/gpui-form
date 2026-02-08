# gpui-form-derive Architecture

## Purpose

`gpui-form-derive` provides the procedural macros that turn user-defined types into gpui-form components and metadata.

## Key modules

- `src/lib.rs`: proc-macro entry points.
- `src/derives/gpui_form/mod.rs`: `GpuiForm` macro expansion entry module.
- `src/derives/select_item.rs`: `SelectItem` derive for gpui-component selects.
- `src/derives/infinite_select.rs`: `InfiniteSelect` derive for cascading enums.

## Data flow

### `GpuiForm`

1. Parse struct fields and attributes with `darling` (including `cfg_attr` flattening).
1. Convert `#[gpui_form(component(...))]` into `Components` from `gpui-form-core`.
1. Use `ComponentLayout` implementations to generate:
   - `FormFields` struct (component state entities)
   - `FormComponents` constructors
1. Generate a `FormValueHolder` that normalizes optionality and validation.
1. When `inventory` is enabled, submit a `GpuiFormShape` to the registry for prototyping.
1. If Koruma is present, mirror validation metadata and optional fluent error labels.

### `SelectItem`

- Implements `gpui_component::select::SelectItem` for enums.
- Optional `#[select_item(fluent)]` uses `es-fluent` for titles.

### `InfiniteSelect`

- Generates the `InfiniteSelect` trait implementation for nested enums.
- Supports unit, tuple, and single-field struct variants.
- Optionally reads `#[fluent_kv(...)]` to pick localized labels.

## Extension points

- New components require updates in `gpui-form-core` and `gpui-form-prototyping-core`.
- Keep generated metadata (`GpuiFormShape`) aligned with new behaviors.

## Tests

- Snapshot tests live under `src/derives/snapshots` and use `insta` + `prettyplease`.
