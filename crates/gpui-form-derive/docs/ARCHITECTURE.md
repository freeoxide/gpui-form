# gpui-form-derive Architecture

`gpui-form-derive` owns the main proc-macro entry points for the workspace.

It parses user structs and enums, delegates component modeling to
`gpui-form-codegen`, and emits the generated types and inventory metadata that
power the rest of the ecosystem.

## Entry Points

`src/lib.rs` defines:

- `#[derive(GpuiForm)]`
- `#[derive(SelectItem)]`
- `#[derive(CustomComponentState)]`

`#[derive(InfiniteSelect)]` is not part of this crate. It lives in
`gpui-form-component-derive` and is re-exported by the facade as
`gpui_form::InfiniteSelect`.

## Module Layout

- `src/derives/gpui_form/mod.rs`: `GpuiForm` entry module
- `src/derives/gpui_form/structs.rs`: `darling` option model for field/struct
  attributes (`GpuiFormOptions`, `ComponentField`), including the layout hints
  (`section`/`label`/`description`/`placeholder` as `Option<String>` and
  `width` via a custom `FromMeta` on `LayoutWidthMeta` that accepts a bare
  ident `full|half|third` or a quoted string, modeled on `TypeOverride`)
- `src/derives/gpui_form/expansion.rs`: top-level `GpuiForm` expansion
  pipeline; also builds the per-field `FieldLayout` tokens and appends them to
  the emitted `FieldVariant` builder chain via `.with_layout(...)` using the
  facade path `::gpui_form::schema::layout::...`
- `src/derives/gpui_form/components.rs`: delegates component fields into
  codegen layouts
- `src/derives/gpui_form/value_holder.rs`: generated holder types, defaults,
  conversion logic, and skip-field handling
- `src/derives/gpui_form/form_state.rs`: generated `<Name>FormState` facade +
  `<Name>FormPatch` (field-level state model, M1: touched tracking + per-field
  dirty/patch)
- `src/derives/gpui_form/errors.rs`: generated `<Name>FormFieldErrors` map,
  `<Name>FormFieldError` borrowing enum, and the holder's `validate_fields()`
  (backlog #6 typed error access, koruma-gated)
- `src/derives/gpui_form/field_path.rs`: generated `<Name>FormPath` typed
  field-path newtype (backlog feature #8, FLAT v1)
- `src/derives/gpui_form/koruma.rs`: Koruma metadata mirroring helpers
- `src/derives/gpui_form/cfg_attr.rs`: `cfg_attr` flattening before parse-time
  inspection
- `src/derives/select_item.rs`: `SelectItem` expansion
- `src/derives/custom_component_state.rs`: `CustomComponentState` expansion

## `GpuiForm` Expansion Pipeline

1. Parse the input with `syn`.
1. Flatten `cfg_attr` wrappers so downstream parsing sees effective
   `#[gpui_form(...)]` data.
1. Parse struct-level and field-level `#[gpui_form(...)]` data with `darling`.
1. Parse Koruma field metadata through `koruma-derive-core`.
1. For each component field, delegate component-specific modeling to
   `gpui-form-codegen`.
1. Emit:
   - `FormFields`
   - `FormComponents`
   - `FormValueHolder`
   - `FormPath` (typed field-path newtype wrapping `::gpui_form::core::FieldPath`,
     plus the `FIELD_NAMES`/`FIELD_COUNT`/`all_paths()`/`field_index()`/
     `path_for_index()` tables added in M1)
   - `FormState` + `FormPatch` (the field-level state facade and patch type from
     `form_state.rs`; unconditional, spliced into BOTH the empty-form early
     return and the main expansion)
   - `FormFieldErrors` + `FormFieldError` + the holder's `validate_fields()`
     (from `errors.rs`; gated on `needs_koruma_derive` in `value_holder.rs`)
   - conversions between the original type and the holder
   - optional inventory metadata

## Value Holder Responsibilities

`value_holder.rs` owns:

- optionality normalization between model fields and editable form state
- default-value seeding
- `type`/`from`/`into` conversions
- `#[gpui_form(skip)]` reconstruction behavior
- Koruma mirroring for holder validation

The holder stays PLAIN DATA: the only member M1 adds is the pure
`validate_fields()` method (koruma-gated, a function of the value like
koruma's own `validate()`). Touched/dirty/error/submit state never lives on
the holder — it lives on the generated `<Name>FormState` facade and the core
primitives it composes.

Important behaviors:

- originally optional fields stay optional in the holder
- input-style fields usually wrap in `Option<T>` to represent empty UI state
- skipped fields are still prefilled when converting from the original model
  into the holder
- reverse conversion becomes explicit `into_original(...)` when skipped fields
  prevent a fully automatic round trip

### Feature `serde` (form-state persistence)

The holder's `#[derive(...)]` list is built in `value_holder.rs` and starts from
`Clone, Debug` (plus `Default`, the `bon::Builder` when skipped fields are
present, and the relevant `Koruma` derives). `::core::cmp::PartialEq` is ALWAYS
pushed onto that list, regardless of features. Under `#[cfg(feature = "serde")]`
the block additionally pushes `::serde::Serialize` and `::serde::Deserialize`.

Rationale:

- `PartialEq` (not `Eq`) is emitted unconditionally and deliberately.
  `number_input(as = f64)` and other non-`Eq` field types would break
  compilation under `Eq`, and `PartialEq` is the exact bound required by
  `gpui_form_core::FormState::is_dirty` / `diff_against` — both of which are
  exported unconditionally. Gating `PartialEq` behind `serde` would mean
  `is_dirty` fails to compile on default features, so the holder must derive
  `PartialEq` even when serialization is off.
- The facade forwards its `serde` feature to `gpui-form-derive/serde`, so any
  expansion produced while the derive crate is built with the feature inherits
  the `Serialize`/`Deserialize` derives. Serialization is opt-in; comparison
  is not.

A holder carrying `#[gpui_form(skip)]` fields still round-trips through serde on
its own, but cannot fully reconstruct the source struct via `into_original`
(skipped values are absent from the holder). This mirrors the existing
`has_skipped_fields` behavior. Per-field serde passthrough (rename/skip) is
out of scope here and tracked as backlog feature #15.

### Field Path Generation (feature #8, FLAT v1)

`field_path.rs` owns `generate_field_path(original_input, fields)`, spliced
into BOTH expansion branches — the empty-form early-return in `expansion.rs`
AND the main non-empty expansion — so every form emits a path type even with
zero non-skipped fields. It mirrors how `value_holder.rs` is invoked right
before it at each call site.

The generated `<Name>FormPath` (`format_ident!("{}FormPath", input.ident)`,
the same convention as `<Name>FormValueHolder` in `value_holder.rs`):

- is a newtype `pub struct <Name>FormPath(::gpui_form::core::FieldPath)` with
  a PRIVATE tuple field, reached via `path()`/`into_path()`/`Deref`/`AsRef`
- derives `Clone, Debug, Eq, PartialEq, ::core::hash::Hash` unconditionally —
  no `serde` derives, no `#[cfg]` gates (the wrapped primitive is headless
  and not behind a feature flag)
- exposes `new(&[&'static str])` plus one same-named `pub fn <field>() -> Self`
  per NON-skipped field. Skipped fields are excluded by the same
  `filter(|f| !f.skip)` used for component fields, mirroring how skipped
  fields are absent from the holder. If a form has zero non-skipped fields,
  the type is still emitted with `new`/`path`/`into_path` and no per-field ctors
- carries NO generics, even for generic source structs. A path only names
  fields — it stores no typed values — so `Display`, `AsRef`, and `Deref`
  impls likewise take no generics. This diverges from `value_holder.rs`, which
  threads `split_for_impl()` through the holder struct and its impls
- reaches the shared primitive via the facade path
  `::gpui_form::core::FieldPath`, mirroring how `value_holder.rs` reaches
  `::gpui_form::bon` and how `expansion.rs` reaches `::gpui_form::schema`.
  Do NOT use `::gpui_form_core` directly in emitted tokens

FLAT v1 scope: each constructor names a single field. Typed nested-path and
list-item-path constructors arrive with backlog features #2 ("Nested forms")
and #3 ("Repeated fields"); hand-built multi-segment paths via
`<Name>FormPath::new(&["a", "b"])` work today.

M1 extends the same impl with the declaration-order tables the field-level
state model keys on: `FIELD_NAMES` (`&'static [&'static str]`, non-skipped
fields only), `FIELD_COUNT`, `all_paths()` (declaration order),
`field_index()` (dense index, `Some` only for FLAT v1 single-segment paths
naming a real field), and `path_for_index()` (the inverse, a generated
`match`). Iteration for touched/dirty/error surfaces comes from
`all_paths()` — never from ad-hoc per-field string lists — so M3's
multi-segment paths extend one surface.

### Field-Level State Facade + Patch (M1: touched, per-field dirty, patch)

`form_state.rs` owns `generate_form_state(original_input, fields)`, spliced
into BOTH expansion branches next to the path tokens. It emits:

- `<Name>FormState`: a facade COMPOSING the standalone core primitives —
  `::gpui_form::core::FormState<<Name>FormValueHolder>` (baseline/current)
  and `::gpui_form::core::TouchedFields` (sized from `FIELD_COUNT`) — never
  new state fields on the holder or on core `FormState<H>` (which stays a
  frozen two-slot generic). Facade methods: value accessors and
  escapes (`state()`/`state_mut()`), reset semantics
  (`reset_to_baseline` clears touched; `reset_to_baseline_keeping_touched`
  and `sync_baseline`/`replace_current` keep it), touched accessors
  (`touch`/`is_touched`/`touched_fields`/`clear_touched`/`clear_all_touched`,
  all path-keyed and inert for unknown paths), and per-field dirty
  (`is_field_dirty` — a generated `match` on path segments comparing holder
  fields, `false` + `debug_assert!` for unknown paths; empty forms emit a
  plain `false` arm since they have no valid paths at all),
  `changed_fields`, `patch`.
- `<Name>FormPatch`: one `Option<holder-field-type>` slot per non-skipped
  field (so wrapped/originally-optional fields read as `Option<Option<T>>`;
  `Some(None)` = changed-to-cleared). `between`/`is_empty`/`changed_fields`/
  `apply_to` are pure holder-typed operations — comparison happens on holder
  field types, which all implement `PartialEq` because the holder derives it,
  so `type`/`from`/`into` conversion fields need no special handling. Under
  the derive crate's `serde` feature the patch also derives
  `Serialize`/`Deserialize` with
  `#[serde(skip_serializing_if = "Option::is_none", default)]` per field
  (the PATCH tri-state wire format).

Generics note: the facade and patch carry the source struct's generics
verbatim, diverging from `<Name>FormPath`'s no-generics rule. A path names no
types so it can be generics-free; the facade names the holder
(`FormState<<Name>FormValueHolder<..>>`) and a generics-free facade would
break generic source structs that compile today.

### Typed Field Errors (backlog #6, M1)

`errors.rs` owns `generate_field_errors(original_input, fields)`, invoked from
`value_holder.rs` under `needs_koruma_derive` — the SAME gate that puts
`#[derive(Koruma)]` on the holder (NOT `koruma_options.is_some()`, which
would silently drop field-attr-only koruma forms). It emits:

- `<Name>FormFieldErrors`: one private `Option<..>` slot per koruma-
  participating non-skipped field. Koruma only generates a top-level error
  getter for holder fields that carry koruma attrs, so the emitter filters on
  the exact predicate `value_holder.rs` uses to mirror attrs
  (synthetic-required OR field validators OR element validators OR newtype).
  Slots hold owned clones of koruma error structs.
- `<Name>FormFieldError<'a>`: a borrowing enum, one variant per participating
  field, with `path()` and `issue_keys()` (the failed field-level validators'
  `validation.*` keys, mirrored from the holder's koruma attr list — the same
  getter-name computation koruma uses, snake of the last path segment with a
  full-path fallback on collisions). A single uniform `all()` method is NOT
  emitted: the per-field koruma validator enums are distinct types, so a
  per-variant return type cannot exist on one method; koruma's `all()`
  stays reachable through the variant payloads.
- the holder's `validate_fields()`: runs koruma `validate()` once and re-keys
  the result; `map.is_empty()` == `validate().is_ok()`.

Slot shapes mirror koruma's top-level getters: regular fields and
newtype-with-validators expose `&<Holder><Field>KorumaValidationError`;
`#[koruma(nested)]` fields expose `Option<&<Inner as ValidateExt>::Error>`;
bare newtypes expose the inner error directly (behind `Option` when the
holder field is option-wrapped). Inner failures (newtype `Inner`, nested)
report the generic `validation.invalid` key in `issue_keys()` — inner
validator kinds live on the inner type and are invisible to this crate.
`error_for`/`failing_fields` match on the FULL single-segment path, so
unknown and multi-segment paths return `None`.

Generic source structs are excluded (the emitter returns an empty stream):
koruma's generated error types carry a subset of the source generics chosen
by usage analysis that cannot be reconstructed here. Generic koruma forms
keep compiling exactly as before, minus this opt-in surface.

`gpui-form-core::validation::FormFieldErrors` is the form-agnostic trait the
generated map implements (`is_empty`, `failing_paths`).

## Koruma Integration

`GpuiForm` can enable Koruma-aware holder generation even when the source
struct only contains field-level `#[koruma(...)]` attributes.

The derive layer:

- reads normalized validator metadata from `koruma-derive-core`
- mirrors validators into the holder type
- preserves shorthand and builder-chain validator forms
- injects required-value behavior where holder optionality would otherwise lose
  source-model required semantics

Under `koruma(fluent)` the holder additionally derives `KorumaAllDisplay` and
emits an i18n surface — `<FORM>_I18N_KEY_PREFIX`,
per-field `<FIELD>_LABEL_KEY`, `validation_issue_key`, and
`localized_validation_issue` backed by `rust_i18n::t!` — that resolves against
the consuming crate's `rust-i18n` backend.

## Inventory Integration

When the `inventory` feature is enabled:

1. `GpuiForm` emits one `GpuiFormShape` per derived struct.
1. Each field becomes a `FieldVariant` with behavior metadata from
   `gpui-form-codegen`.
1. Metadata includes validation rule identifiers, defaults, full value type
   paths, custom component UI paths, skipped-field information, and a
   `FieldLayout` built from the field's layout hints for downstream generators.
1. The `FieldLayout` is appended to the `FieldVariant` builder chain as
   `.with_layout(::gpui_form::schema::layout::FieldLayout::new()#section#label#description#placeholder.with_width(...))`.
   Layout hints on `#[gpui_form(skip)]` fields are ignored: the
   `FieldVariant`-construction `filter_map` returns `None` for skipped fields
   before any layout tokens are built, so no variant (and no layout) is emitted.

## Other Derives

### `SelectItem`

- implements `::gpui_kit::component::select::SelectItem`
- accepts `#[select_item(fluent)]` for enums that do not implement `Display`;
  emits `title()` bodies that resolve `"<enum_snake>.<variant_snake>"` through
  `rust_i18n::t!` and fall back to the variant name on a miss (the emitted
  tokens resolve against the consuming crate's `rust-i18n` backend, so that
  crate must host `rust_i18n::i18n!` at its root)

### `CustomComponentState`

- emits a `CustomComponentShape` impl directly for a state type
- defaults constructor wiring to `Self::new(window, cx)`
- optionally stores a component path for prototyping output

## Coordination Rules

When adding a component or attribute:

1. update parse-time option support in `gpui-form-codegen`
1. update holder behavior in this crate if optionality or conversion changes
1. update `gpui-form-schema` metadata emission
1. update `gpui-form-prototyping-core` generator support
1. update user-facing docs in the facade README and derive README

If the change also affects `InfiniteSelect`, update
`gpui-form-component-derive`, `gpui-form-component`, and their docs in the same
change.

## Tests

- targeted `GpuiForm` expansion tests live in
  `src/derives/gpui_form/tests.rs`
- compile-fail UI tests live under `tests/ui`
- `tests/serde_round_trip.rs` exercises the `serde` feature end-to-end (holder
  serde round-trip, `Option` fields, skipped-field holder, and `PartialEq`
  comparability). It is gated with `#![cfg(feature = "serde")]`, so it is
  excluded from the default-feature build that proves feature-OFF still compiles.
- `tests/form_state_dirty_without_serde.rs` is the UNGATED regression test that
  proves `FormState::is_dirty` compiles and behaves under DEFAULT features
  (no `serde`). It locks in the unconditional `PartialEq` derive on the holder;
  if `PartialEq` ever gets re-gated behind `serde`, this test stops compiling.
- `tests/field_path.rs` exercises the feature-#8 path type end-to-end (per-field
  ctors, skip exclusion, `Display`/`Deref`/`AsRef`/`into_path`, empty-form
  branch, distinct types per form). It is NOT gated — `FieldPath` is
  unconditional.
- `tests/form_state_facade.rs` exercises the M1 facade + path tables (touched
  lifecycle, reset semantics, `FIELD_COUNT`/`all_paths`/`field_index`
  round-trips, skipped-field exclusion, empty-form branch).
- `tests/patch_diff.rs` exercises the M1 patch (storage tri-state coverage:
  `WrappedOption`/`OriginallyOptional`/`Plain`, conversion fields, `apply_to`,
  declaration-order `changed_fields`). Deliberately not serde-gated.
- `tests/error_for.rs` exercises the koruma-gated typed error surface
  (field-attr-only koruma forms, struct-level koruma forms, newtype
  wrapper/bare getter shapes, `issue_keys`, the `FormFieldErrors` trait).
- `tests/field_level_state_without_serde.rs` is the R13 default-features
  regression test for the M1 emission (mirrors
  `form_state_dirty_without_serde.rs`): the facade/patch/error surface must
  compile and behave with the `serde` feature OFF.
- `tests/ui/error_for_koruma_pass.rs` is a trybuild pass fixture compiling the
  whole M1 surface (facade + patch + `error_for`) end-to-end in one binary.

## When To Update This Document

Update this file when:

- the expansion pipeline changes
- holder conversion behavior changes
- the holder derive list (`Clone`/`Debug`/`Default`/`Builder`/`Koruma` plus the
  unconditional `PartialEq`, plus the `serde` feature's
  `Serialize`/`Deserialize`) changes
- inventory or Koruma emission rules change
- macro responsibilities move between modules
- the generated `<Name>FormPath` shape changes (constructors, generics,
  trait impls, or the facade path convention)
- the layout-hint attribute set, `LayoutWidthMeta` parsing, or the emitted
  `.with_layout(...)` chain changes
