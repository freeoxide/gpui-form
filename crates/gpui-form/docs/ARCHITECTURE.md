# gpui-form Architecture

## Purpose
`gpui-form` is the facade crate. It re-exports the derive macros, core types, and (optionally) runtime components. It also hosts shared numeric validation helpers used by generated code.

## Key modules
- `src/lib.rs`: re-exports `gpui-form-core` unconditionally, `gpui-form-derive` behind `derive`, and `gpui-form-component` behind `component` + `derive`.
- `src/numeric.rs`: input validation helpers for signed/unsigned numeric text entry.

## Data flow
1. The user derives `GpuiForm`/`SelectItem`/`InfiniteSelect` from this crate (requires the `derive` feature).
2. The derive macros (from `gpui-form-derive`) generate types and wiring that reference core metadata in `gpui-form-core`.
3. If `component` + `derive` are enabled, generated code can reference `gpui-form-component` runtime helpers (e.g., InfiniteSelect).
4. Numeric fields use the helpers in `numeric.rs` for validation in number inputs.

## Feature flags
- `derive` (default): exposes the proc macros.
- `component`: re-exports `gpui-form-component` runtime helpers when `derive` is also enabled.
- `inventory`: forwards to `gpui-form-derive`'s `inventory` feature (effective only when `derive` is enabled).

## Extension points
This crate is intentionally thin. Add new behavior in:
- `gpui-form-core` (component definitions and metadata)
- `gpui-form-derive` (macro expansion)
- `gpui-form-component` (runtime helpers)

## Tests
- Numeric validation tests live in `src/numeric.rs`.
