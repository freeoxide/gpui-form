# Form-state persistence

Status: **shipped** (commit 51ed490, "Add form-state persistence and dirty
tracking").

## Summary

Generated forms are **saveable and restorable**, and they can tell whether the
user has edited them:

1. Opt-in `serde` (de)serialization of the generated value holder (JSON; any
   other serde format works through the same derives).
2. Pure, GPUI-free helpers in `gpui-form-core` for `is_dirty`,
   `reset_to_baseline`, and diffing two holders.

See the "Saving, Restoring, and Dirty Tracking" section of the root `README.md`
for the user-facing quick start, `crates/gpui-form-core/src/state.rs` for
`FormState<H>`, and `value_holder.rs` for the conditional serde derives.

## Current state

The generated holder is plain data with conversions to/from the source struct,
and it is (de)serializable behind the `serde` feature:

- The always-derived holder derive list is `Clone`, `Debug`, optionally
  `Default`, `bon::Builder` (when any field is skipped), `koruma::Koruma`, and
  `::core::cmp::PartialEq`
  (`crates/gpui-form-derive/src/derives/gpui_form/value_holder.rs:493-514`).
  `PartialEq` (not `Eq`) is emitted **unconditionally** because
  `FormState::is_dirty` / `diff_against` are exported unconditionally and
  require `H: PartialEq`, yet `serde` is not in `default` — so the holder must
  be comparable on default features too. It is `PartialEq` rather than `Eq`
  because `number_input(as = f64)` and similar non-`Eq` field types must still
  compile.
- Under `#[cfg(feature = "serde")]` only, `::serde::Serialize` and
  `::serde::Deserialize` are added to the derive list (`value_holder.rs:517-521`).
- `gpui-form-derive` declares `serde = { workspace = true, optional = true }`
  and the `serde = ["dep:serde"]` feature
  (`crates/gpui-form-derive/Cargo.toml`); the `dep:serde` form is required
  precisely because the dependency is optional.
- `present_fields_json`, a non-invertible `Debug` dump that exists only for
  forms with skipped fields (`value_holder.rs:614`), still cannot round-trip —
  but with the `serde` feature the holder itself can.
- Dirty-tracking / reset / diff is live in `gpui-form-core` as `FormState<H>`
  (`crates/gpui-form-core/src/state.rs`): `is_dirty`,
  `reset_to_baseline`, `sync_baseline`, `diff_against`, plus
  `current`/`baseline`/`current_mut`/`replace_current`/`into_current`. It
  snapshots a `baseline` at construction and compares against it; there is no
  field-level patch/delta (that is backlog feature #9).
- The `From<Original> for Holder` and `From<Holder> for Original` / `try_from`
  conversions exist (`value_holder.rs:641-657`, `:664-677`), so the holder is
  the natural serialization boundary.

## Design notes

### 1. A `serde` feature on the facade

- `gpui-form/Cargo.toml`: `serde = ["dep:serde", "gpui-form-derive?/serde"]`
  (the `?` is required because `gpui-form-derive` is declared
  `optional = true`).
- `gpui-form-derive/Cargo.toml`: `serde = { workspace = true, optional = true }`
  plus `serde = ["dep:serde"]` — the feature **must** list `dep:serde` (not
  `serde = []`) because the dependency is optional.
- `value_holder.rs`: `::core::cmp::PartialEq` is pushed onto the `derives`
  vector built at line 493 **unconditionally**, and under
  `#[cfg(feature = "serde")]` `::serde::Serialize` and
  `::serde::Deserialize` are pushed as well. `PartialEq` is unconditional
  because `FormState::is_dirty` / `diff_against` are exported unconditionally
  and `serde` is not in `default`.

`serde`'s derive generates the correct `T: Serialize` / `T: Deserialize` bounds
from the holder's generics automatically, so monomorphized generic forms keep
working without manual bound bookkeeping.

### 2. Dirty-tracking and reset helpers in `gpui-form-core`

These are pure logic with no GPUI coupling, so they live in `gpui-form-core`
next to the existing numeric helpers. The core obstacle was the **baseline**: to
compute "dirty" the form must remember the state it started from, but
`default_expr` is parse-time only and is not retained on the runtime holder.
The shipped solution sidesteps this by snapshotting the actual passed-in holder
at construction time rather than reconstructing it from defaults.

`FormState<H>` (in `core`) owns a `baseline: H` and a `current: H`, exposing:

- `is_dirty()` — whole-value equality (`current != baseline`; requires
  `H: PartialEq`).
- `reset_to_baseline()` — restores `current` from a clone of `baseline`.
- `sync_baseline()` — snapshots `current` into `baseline` (call after a save).
- `diff_against(&H)` — whole-value boolean comparison (`current != other`);
  field-level patch/delta is backlog feature #9.

The application constructs it from the initial holder. This keeps the holder
itself a plain data struct and centralizes the baseline problem in one place
(rejected alternative: baking the baseline into the holder, which makes it
heavier for everyone and couples a runtime concern into generated code).

### 3. Edge cases

- **Skipped fields.** The holder omits `#[gpui_form(skip)]` fields, so a
  serialized holder cannot fully reconstruct a source struct via `into_original`
  (which requires the skipped values). Serde round-trips the *holder*, not
  necessarily the *source struct* — mirroring the existing
  `has_skipped_fields` limitation (`crates/gpui-form-schema/src/registry.rs:15-19`).
- **`Option<T>` wrapping.** Some components wrap in `Option<T>`
  (`default_wraps_in_option`); `serde` handles `Option` natively, so no extra
  work is required. Field-level `#[serde(...)]` passthrough (rename/skip) may be
  wanted later.
- **Custom defaults vs `Default`.** `Default` is not derived when the form has
  custom default expressions (`value_holder.rs:494`). `reset_to_baseline`
  therefore does not assume `Holder: Default`; the `FormState` baseline avoids
  that dependency.

## Scope

Covered: serializing the holder's data fields; dirty/reset/diff helpers.

Not covered:

- Per-component *runtime* state (open menus, scroll positions). Only the
  holder's data fields are serialized. `InfiniteSelectState::snapshot()` already
  covers that component's own restore needs.
- A full undo/redo stack.

## Possible follow-ups (not implemented)

1. TOML (or other format) conveniences on top of the existing serde derives
   (`toml` is already a workspace dependency).
2. Field-level `#[serde(...)]` attributes passable through
   `#[gpui_form(...)]` (backlog feature #15); default `serde` behavior is what
   ships today.
3. Field-level diff/patch building on the typed field paths foundation
   (backlog feature #9).
