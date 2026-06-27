# Next feature: form-state persistence

Status: **proposed** — the next feature to implement.

## Summary

Generated forms should be **saveable and restorable**, and they should know
whether the user has edited them. Today a generated `…FormValueHolder` cannot
be serialized, so a form's state cannot be written to disk/cache, sent over
IPC, or restored when reopened — and there is no way to tell whether the form
is dirty.

This feature adds:

1. Opt-in `serde` (de)serialization of the generated value holder (JSON, with
   TOML as a follow-on).
2. Pure, GPUI-free helpers for `is_dirty`, `reset_to_defaults`, and diffing two
   holders.

## Why this is next

It is the highest-value genuine gap in the workspace:

- Every form library eventually needs save/restore and dirty state; today there
  is **zero** support for either.
- It is self-contained — it touches the derive and `gpui-form-core` only, with
  no cross-crate refactor.
- It is low-risk because it is strictly additive, gated behind a feature flag.

## Current state

The generated holder is plain data with conversions to/from the source struct,
but it is not serializable:

- The holder derive list is `Clone`, `Debug`, optionally `Default`,
  `bon::Builder`, and `koruma::Koruma` — **no** `Serialize`/`Deserialize`
  (`crates/gpui-form-derive/src/derives/gpui_form/value_holder.rs:493-507`).
- `gpui-form-derive` declares **no** `serde` dependency
  (`crates/gpui-form-derive/Cargo.toml`).
- The only JSON output is `present_fields_json`, a non-invertible `Debug` dump
  that exists only for forms with skipped fields
  (`value_holder.rs:600`). It cannot round-trip.
- There is **no dirty-tracking / reset / diff** API. Field defaults are consumed
  at macro-expansion time and are not retained on the runtime holder, so there
  is no baseline to compare against.
- The `From<Original> for Holder` and `From<Holder> for Original` / `try_from`
  conversions already exist (`value_holder.rs:639-657`, `:669-677`), so the
  holder is the natural serialization boundary.

## Design

### 1. A `serde` feature on the facade

- `gpui-form/Cargo.toml`: add `serde = ["dep:serde", "gpui-form-derive/serde"]`.
- `gpui-form-derive/Cargo.toml`: add `serde = { workspace = true }` and a
  `serde = []` feature.
- `value_holder.rs`: when the feature is enabled, push `::serde::Serialize` and
  `::serde::Deserialize` onto the `derives` vector built at line 493.

`serde`'s derive generates the correct `T: Serialize` / `T: Deserialize` bounds
from the holder's generics automatically, so monomorphized generic forms keep
working without manual bound bookkeeping.

### 2. Dirty-tracking and reset helpers in `gpui-form-core`

These are pure logic with no GPUI coupling, so they belong in `gpui-form-core`
next to the existing numeric helpers. The core obstacle is the **baseline**: to
compute "dirty" the form must remember the state it started from, and
`default_expr` is currently parse-time only.

Recommended approach: a `FormState<H>` wrapper (in `core`) that owns a
`baseline: H` and a `current: H`, exposing:

- `is_dirty()` — compares `current` to `baseline` (requires `H: PartialEq`).
- `reset_to_defaults()` — restores `current` from `baseline`.
- `diff_against(&H)` — field-level comparison against another holder.

The application constructs it from the initial holder. This keeps the holder
itself a plain data struct and centralizes the baseline problem in one place
(rejected alternative: baking the baseline into the holder, which makes it
heavier for everyone and couples a runtime concern into generated code).

### 3. Edge cases to handle

- **Skipped fields.** The holder omits `#[gpui_form(skip)]` fields, so a
  serialized holder cannot fully reconstruct a source struct via `into_original`
  (which requires the skipped values). Document this: serde round-trips the
  *holder*, not necessarily the *source struct* — mirroring the existing
  `has_skipped_fields` limitation (`crates/gpui-form-schema/src/registry.rs:15-19`).
- **`Option<T>` wrapping.** Some components wrap in `Option<T>`
  (`default_wraps_in_option`); `serde` handles `Option` natively, so no extra
  work is required. Field-level `#[serde(...)]` passthrough (rename/skip) may be
  wanted later.
- **Custom defaults vs `Default`.** `Default` is not derived when the form has
  custom default expressions (`value_holder.rs:494`). `reset_to_defaults` must
  therefore not assume `Holder: Default`; the `FormState` baseline avoids that
  dependency.

## Scope

In scope: serializing the holder's data fields; dirty/reset/diff helpers.

Out of scope:

- Per-component *runtime* state (open menus, scroll positions). Only the
  holder's data fields are serialized. `InfiniteSelectState::snapshot()` already
  covers that component's own restore needs.
- A full undo/redo stack — not in this feature.

## Implementation plan

1. Add the `serde` feature plumbing to `gpui-form` and `gpui-form-derive`; emit
   `Serialize`/`Deserialize` on the holder when enabled.
2. Add `FormState<H>` with `is_dirty` / `reset_to_defaults` / `diff_against` to
   `gpui-form-core`; re-export from the facade.
3. Tests: a derive test that round-trips a holder through `serde_json`; a `core`
   test for the dirty/reset/diff helpers.
4. Doc sync (per `AGENTS.md`): root `README.md`, `crates/gpui-form/README.md`
   (feature flag + save/restore quick-start), and
   `crates/gpui-form-derive/docs/ARCHITECTURE.md` for the new derive in the list.

## Open questions

1. JSON only at first, or wire TOML in the same change? (`toml` is already a
   workspace dependency.)
2. Should field-level `#[serde(...)]` attributes be passable through
   `#[gpui_form(...)]`, or is default `serde` behavior enough for v1?
3. Does `FormState<H>` live in `gpui-form-core` unconditionally, or behind its
   own feature gate?
