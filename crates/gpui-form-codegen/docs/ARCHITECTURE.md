# gpui-form-codegen Architecture

`gpui-form-codegen` is the parse-time component-model and token-generation layer
used by `gpui-form-derive`.

It sits between raw proc-macro attribute parsing and emitted code, giving the
workspace one place to define how built-in components behave at expansion time.

## Purpose

This crate exists to:

1. parse `component(...)` options into a typed internal model
1. map each built-in component onto generated `FormFields` and
   `FormComponents` tokens
1. emit behavior metadata tokens aligned with `gpui-form-schema`

It should stay proc-macro-adjacent rather than becoming a second proc-macro
crate.

## Modules

- `src/lib.rs`: exports the component model
- `src/components.rs`: parse-time component definitions and behavior-token
  generation
- `src/names.rs`: helper naming utilities for generated identifiers
- `src/implementations/`: per-component `ComponentLayout` implementations

## Parse-Time Component Model

`components.rs` defines the typed options for the supported component families:

- `InputOptions`
- `NumberInputOptions`
- `CheckboxOptions`
- `SwitchOptions`
- `SelectOptions`
- `InfiniteSelectOptions`
- `CustomOptions`
- `DatePickerOptions`

Important parse-time responsibilities:

- `number_input(as = ...)` stores validation-type overrides
- `select(...)` and `infinite_select(...)` store behavior options plus field
  defaults
- `custom(...)` validates that exactly one of `shape = ...` or `state = ...` is
  present
- `custom(..., wraps_in_option = false)` overrides the default holder wrapping
  rule

## Component Layout Emission

Each component implementation under `src/implementations/` emits two things:

- field entries for generated `FormFields`
- constructor functions for generated `FormComponents`

For `infinite_select`, that emitted field/runtime wiring is intentionally
coarse-grained: generated code stores one runtime `InfiniteSelectState` entity
per form field rather than separate root/child select entities.

This keeps `GpuiForm` expansion readable: the derive layer handles struct-level
coordination while per-component files own the field/runtime wiring details.

## Metadata Emission

The codegen layer also emits `gpui-form-schema` behavior tokens so that
inventory/prototyping metadata stays aligned with generated runtime behavior.

Static component identity is intentionally sourced from
`gpui_form_schema::components::ComponentKind` rather than a codegen-local enum.

## Dependency Boundaries

This crate should know about:

- proc-macro-facing parse-time types (`syn`, `quote`, `proc_macro2`)
- runtime metadata shape (`gpui-form-schema`)

This crate should not know about:

- inventory submission
- Koruma validator parsing
- user-facing facade re-export policy

Those belong in `gpui-form-derive`, `koruma-derive-core`, and `gpui-form`.

## New Component Checklist

When adding a component:

1. add its option type and parse-time definition in `components.rs`
1. add a `ComponentLayout` implementation under `src/implementations/`
1. emit the matching `ComponentsBehaviour` metadata tokens
1. update `gpui-form-schema` to define the runtime metadata shape
1. update `gpui-form-component` if runtime support is required
1. update `gpui-form-prototyping-core` so the generator understands the new
   behavior

## When To Update This Document

Update this file when:

- the parse-time component model changes
- component layout responsibilities move between modules
- metadata emission rules change
