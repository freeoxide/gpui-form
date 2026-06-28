# gpui-form-core Architecture

`gpui-form-core` hosts helper logic that should stay usable without `gpui` or
`gpui-component`.

## Purpose

This crate exists so generated form code can share core behavior without
pulling in runtime UI dependencies.

At the moment the crate is intentionally narrow:

- numeric text-entry validation for generated `number_input` fields
- `FormState<H>` for dirty tracking, reset, and diffing of form holder values
- `path::FieldPath` for typed field naming (backlog feature #8, FLAT v1)

## Modules

- `src/lib.rs`: module export surface (re-exports `FieldPath`, `FormState`, and
  the `path`/`state` modules)
- `src/numeric.rs`: signed and unsigned text-entry validation helpers
- `src/path.rs`: `FieldPath` typed field-path primitive
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
  Field-level patch and delta reporting is backlog feature #9 and will build on
  the `FieldPath` naming primitive.
- The holder's serde derives live on the derive crate behind its `serde`
  feature; `FormState` itself is unconditional (no feature gate, no new
  dependencies). The facade re-exports it as `gpui_form::FormState`.

## FieldPath

`path.rs` defines `FieldPath`, the headless, GPUI-free, serde-free typed
field-path primitive (backlog feature #8, FLAT v1). It is the shared naming
foundation for the upcoming field-level validation (#6), field-level diff/delta
reporting (#9), schema export (#14), and nested/list paths (#2/#3).

Representation choice: `Box<[&'static str]>`. Clones perform a single box
allocation (no per-segment heap work), satisfying the "clones are cheap"
contract for handing paths to validation, dirty-tracking, analytics, or schema
code. The implementer is free to switch to a single-segment-optimized enum
later without changing the public surface, since the representation is private.

Public surface (EXACT per the feature contract):

- `pub fn new(segments: &[&'static str]) -> Self`
- `pub fn segments(&self) -> &[&'static str]`
- `pub fn is_empty(&self) -> bool`

Trait impls: `Clone`, `core::fmt::Debug` (debug-list render `["a", "b"]`),
`Eq`/`PartialEq` (by segment sequence, order matters), `core::hash::Hash`
(by segment slice), `core::fmt::Display` (segments joined by `.`; empty path
renders as `""`).

Scope boundaries (documented in rustdoc and user-facing docs):

- FLAT v1: a path is a list of static segments, typically a single field name.
  Typed nested-path and list-item-path constructors arrive with backlog
  features #2 ("Nested forms") and #3 ("Repeated fields"). Hand-built
  multi-segment paths via `FieldPath::new(&["a", "b"])` work today; typed
  composition is later.
- The primitive is unconditional (no feature gate). When the derive crate's
  `serde` feature is on, the generated `<Name>FormPath` newtypes (which wrap
  this primitive) MAY carry a serialization story; the core primitive itself
  stays serde-free, mirroring how `FormState` handles serialization.
- The derive crate wraps this primitive per form as `<Name>FormPath`, reachable
  via `Deref`/`AsRef`/`into_path` through the facade as
  `gpui_form::FieldPath`.

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
   `gpui_form::numeric`, `FormState` plus the `state` module as
   `gpui_form::FormState` / `gpui_form::state`, and `FieldPath` plus the `path`
   module as `gpui_form::FieldPath` / `gpui_form::path`.
1. Generated number-input code uses the helpers during incremental text edits
   before committing parsed values into the holder.
1. Application code wraps a generated `...FormValueHolder` in `FormState` to
   track edits, reset, or diff against a restored value.
1. Generated `<Name>FormPath` types wrap `FieldPath` so validation, dirty
   tracking, analytics, and schema export share ONE typed way to name fields.

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
- the `FieldPath` public surface, trait impls, representation, or FLAT-v1
  scope boundary changes
