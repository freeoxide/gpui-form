# gpui-form-component Architecture

## Purpose

`gpui-form-component` contains runtime helpers that generated code relies on.

## Key modules

- `src/infinite_select.rs`: trait definitions and helpers for cascading selects.
- `src/custom.rs`: `CustomComponentShape` contract and `custom_component_shape!` helper macro.
- `src/lib.rs`: module export surface.

## Key types

- `InfiniteSelect`: trait implemented by derived enums to expose nested selection behavior.
- `InfiniteSelectItem`: wrapper implementing `SelectItem` for select dropdowns.
- `InfiniteSelectPath`: a compact selection path through nested enums.
- `build_from_path`: constructs a nested enum from a selection path.
- `CustomComponentShape`: shape contract used by `#[gpui_form(component(custom(...)))]`.
  - Has a default `COMPONENT_PATH: Option<&'static str> = None` associated const.
  - `custom_component_shape!` accepts an optional `component = …` arm that sets `COMPONENT_PATH`.
  - `#[derive(CustomComponentState)]` accepts `#[gpui_form_custom(component = …)]` for the same purpose.
  - A `component = …` on the field attribute always takes precedence over the shape constant.

## Data flow

1. `#[derive(InfiniteSelect)]` (from `gpui-form-derive`) implements the trait for nested enums.
1. Generated form code wraps variants in `InfiniteSelectItem` for display.
1. UI changes mutate an `InfiniteSelectPath`, which can be turned back into a value via `build_from_path`.
1. For custom components, users define a shape via `custom_component_shape!` or derive `CustomComponentState` on a state type; `GpuiForm` uses that type to generate state entity fields and component constructors.

## Extension points

- Add runtime helpers here when new component behaviors need non-macro support.
