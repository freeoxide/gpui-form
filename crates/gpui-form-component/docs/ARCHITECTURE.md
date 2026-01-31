# gpui-form-component Architecture

## Purpose

`gpui-form-component` contains runtime helpers that the generated code relies on. Today it focuses on the InfiniteSelect component.

## Key modules

- `src/infinite_select.rs`: trait definitions and helpers for cascading selects.
- `src/lib.rs`: module export surface.

## Key types

- `InfiniteSelect`: trait implemented by derived enums to expose nested selection behavior.
- `InfiniteSelectItem`: wrapper implementing `SelectItem` for select dropdowns.
- `InfiniteSelectPath`: a compact selection path through nested enums.
- `build_from_path`: constructs a nested enum from a selection path.

## Data flow

1. `#[derive(InfiniteSelect)]` (from `gpui-form-derive`) implements the trait for nested enums.
1. Generated form code wraps variants in `InfiniteSelectItem` for display.
1. UI changes mutate an `InfiniteSelectPath`, which can be turned back into a value via `build_from_path`.

## Extension points

- Add new runtime helpers here when new component behaviors need non-macro support.
