# gpui-form-component Architecture

## Purpose

`gpui-form-component` contains the GPUI-facing runtime helper implementations
used by the facade crate.

## Key modules

- `src/infinite_select.rs`: trait definitions and helpers for cascading selects.
- `src/custom.rs`: `CustomComponentShape` contract and `custom_component_shape!` helper macro.
- `src/date_picker.rs`: localized single-date picker wrapper used by generated forms.
- `src/lib.rs`: module export surface.

## Key types

- `InfiniteSelect`: trait implemented by derived enums to expose nested selection behavior.
- `InfiniteSelectItem`: wrapper implementing `SelectItem` for select dropdowns.
- `InfiniteSelectPath`: a compact selection path through nested enums.
- `build_from_path`: constructs a nested enum from a selection path.
- `DatePickerState`: GPUI entity state for the runtime date picker.
- `DatePicker`: localized wrapper around `gpui_component::calendar::Calendar`.
- `DatePickerEvent`: emits `Option<jiff::civil::Date>` so generated code no longer depends on `chrono` display strings.
- `parse_form_date`: shared helper for parsing picker selections into arbitrary form field types via `FromStr`.
- `CustomComponentShape`: shape contract used by `#[gpui_form(component(custom(...)))]`.
  - Has a default `COMPONENT_PATH: Option<&'static str> = None` associated const.
  - `custom_component_shape!` accepts an optional `component = …` arm that sets `COMPONENT_PATH`.
  - `#[derive(CustomComponentState)]` accepts `#[gpui_form_custom(component = …)]` for the same purpose.
  - A `component = …` on the field attribute always takes precedence over the shape constant.

## Data flow

1. `#[derive(InfiniteSelect)]` (from `gpui-form-derive`) implements the trait for nested enums.
1. Generated form code wraps variants in `InfiniteSelectItem` for display.
1. Generated and prototyped date-picker forms target `date_picker::DatePicker` instead of `gpui_component` directly.
1. `date_picker::DatePicker` keeps calendar selection behavior in `gpui_component`, but formats the selected value for display with `jiff` + ICU4X using the active `gpui_component` locale.
1. UI changes mutate an `InfiniteSelectPath`, which can be turned back into a value via `build_from_path`.
1. For custom components, users define a shape via `custom_component_shape!` or derive `CustomComponentState` on a state type; `GpuiForm` uses that type to generate state entity fields and component constructors.
1. `gpui-form` re-exports this crate as `gpui_form::runtime` and also keeps
   root-level compatibility re-exports for the main helper modules.

## Extension points

- Add runtime helpers here when new component behaviors need non-macro support.
