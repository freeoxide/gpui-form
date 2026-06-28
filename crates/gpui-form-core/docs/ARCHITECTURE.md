# gpui-form-core Architecture

`gpui-form-core` hosts helper logic that should stay usable without `gpui` or
`gpui-component`.

## Purpose

This crate exists so generated form code can share core behavior without
pulling in runtime UI dependencies.

At the moment the crate is intentionally narrow:

- numeric text-entry validation for generated `number_input` fields
- `FormState<H>` for dirty tracking, reset, and diffing of form holder values

## Modules

- `src/lib.rs`: module export surface (re-exports `FormState` and the `state`
  module)
- `src/numeric.rs`: signed and unsigned text-entry validation helpers
- `src/state.rs`: `FormState<H>` form-state helper

## FormState

`state.rs` defines `FormState<H>`, the pure, GPUI-free side of form-state
persistence and dirty tracking (feature #1). It owns two private copies of a
holder value `H`:

- `baseline`: the value captured at `new` (or last `sync_baseline`); the "saved"
  reference point.
- `current`: the live, possibly-edited value the UI mutates.

Both fields are private; callers reach them only through accessors so the two
slots cannot drift out of the baseline/current pairing by accident. The impl is
split by trait bound:

- `H: Clone` for construction and the mutating helpers (`new`, `current_mut`,
  `replace_current`, `into_current`, `reset_to_baseline`, `sync_baseline`).
- `H: PartialEq` for `is_dirty` and `diff_against`.

Scope boundaries (documented in rustdoc and user-facing docs):

- The type stores holder **data** only. It owns no GPUI types and no component
  runtime UI state (open menus, scroll positions, `InfiniteSelectState`
  snapshots).
- Dirty/diff is **boolean-level** (`current != baseline` / `current != other`).
  Field-level patch and delta reporting is backlog feature #9.
- The holder's serde derives live on the derive crate behind its `serde`
  feature; `FormState` itself is unconditional (no feature gate, no new
  dependencies). The facade re-exports it as `gpui_form::FormState`.

## Numeric Validation Semantics

`numeric.rs` validates editable text before parsing it into the destination
type.

The helpers intentionally allow intermediate editing states:

- empty strings
- `-` for signed numeric entry

They intentionally reject invalid forms early:

- leading zero patterns such as `01` and `00`
- misplaced or repeated `-`
- non-digit characters for unsigned entry

An optional `require_parse` flag determines whether the helper only validates
the text shape or also verifies that the text can parse into `T`.

## Data Flow

1. `gpui-form-codegen` emits `number_input` handlers that call numeric helpers
   through facade paths such as `gpui_form::numeric::*`.
1. The facade re-exports this crate as `gpui_form::core`, the numeric module as
   `gpui_form::numeric`, and `FormState` plus the `state` module as
   `gpui_form::FormState` / `gpui_form::state`.
1. Generated number-input code uses the helpers during incremental text edits
   before committing parsed values into the holder.
1. Application code wraps a generated `...FormValueHolder` in `FormState` to
   track edits, reset, or diff against a restored value.

## Boundary

This crate should remain:

- UI-neutral
- independent from GPUI entity types
- small enough that lower layers can depend on it without dragging in runtime
  dependencies

If logic needs GPUI state, subscriptions, or component types, it belongs in
`gpui-form-component` instead.

## When To Update This Document

Update this file when:

- new helper modules are added
- numeric validation semantics change
- generated `number_input` code starts depending on new core helpers
- `FormState` API, trait bounds, or scope boundaries change
