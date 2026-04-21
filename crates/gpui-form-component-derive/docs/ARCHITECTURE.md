# gpui-form-component-derive Architecture

`gpui-form-component-derive` owns the proc-macro entry points that are specific
to the runtime helper layer in `gpui-form-component`.

## Entry Points

`src/lib.rs` defines:

- `#[derive(InfiniteSelect)]`

## Module Layout

- `src/infinite_select.rs`: `InfiniteSelect` expansion

## `InfiniteSelect`

This derive emits an implementation of
`gpui_form::infinite_select::InfiniteSelect` for nested enums.

Responsibilities:

- parse unit, single-field tuple, and single-field struct variants
- support `#[tuple_enum(skip)]` for variants that should not appear in the
  select tree
- support `#[tuple_enum(key = "...")]` for persisted key overrides
- emit recursive child traversal methods that match the runtime contract in
  `gpui-form-component`
- optionally wire `fluent_kv` label/description metadata into type and child
  labels
- validate that persisted keys stay unique within one enum

## Dependency Role

This crate should stay focused on the infinite-select runtime contract.

It should not own:

- `GpuiForm` struct parsing and token generation
- select-item derives unrelated to infinite-select
- schema metadata or inventory registration

Those belong in `gpui-form-derive`, `gpui-form-codegen`, and
`gpui-form-schema`.

## Data Flow

1. A user derives `InfiniteSelect` through `gpui-form` or directly through this
   crate.
1. The macro emits trait impls against `gpui_form::infinite_select`.
1. Direct users therefore need a crate named `gpui_form` in scope, while facade
   users get that path automatically from `gpui-form`.
1. Runtime code in `gpui-form-component` consumes that impl through
   `InfiniteSelectItem`, `InfiniteSelectPath`, `InfiniteSelectKeyPath`, and the
   path reconstruction helpers.
1. `GpuiForm` and prototyping output can then build cascading selects on top of
   those runtime helpers.

## When To Update This Document

Update this file when:

- the `InfiniteSelect` derive responsibilities change
- expansion output targets a different runtime contract
- component-oriented proc macros move into or out of this crate
