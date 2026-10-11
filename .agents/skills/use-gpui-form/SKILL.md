---
name: use-gpui-form
description: "Use when Codex needs to build user-facing application forms with gpui-form, including adding #[derive(GpuiForm)] to app structs, choosing gpui_form component attributes, using generated form fields/components/value holders, wiring SelectItem or InfiniteSelect enums, and adding custom components."
---

# Use GPUI Form

## Scope Boundary

Treat this skill as a hosted public-usage guide for `gpui-form` consumers. Use
it only for user-facing application workflows: deriving forms on app models,
choosing component attributes, using generated form state, wiring select and
infinite-select enums, and adding app-owned custom components.

Do not use this skill as a contributor guide for `gpui-form` repository
internals. For build, test, format, lint, maintenance, release, or architecture
work, read the repository source, `AGENTS.md`, and the relevant crate
documentation directly.

## Documentation Sources

The repository root `README.md` (single source) carries install snippets,
component syntax details, and the full feature sections: phone validation,
Koruma validation, localization, form-state persistence, typed field paths,
custom components, file/date picker runtime, and prototyping. Reference those
sections by name instead of transcribing them.

- `references/api-map.md`: facade import map, component/attribute inventory,
  generated names, and pattern skeletons.
- `examples/README.md`: canonical index of runnable workspace examples.

## Core Workflow

1. Identify the application struct or enum that should drive the form.
2. Check the app's existing `Cargo.toml` and form code for local patterns.
3. Depend on the facade crate and import `gpui_form::{GpuiForm, SelectItem}`.
4. Add `GpuiForm` to a normal Rust struct and annotate each visible field with
   a component.
5. Use generated types named from the source struct: `<Name>FormFields`,
   `<Name>FormComponents`, `<Name>FormValueHolder`, and `<Name>FormPath`.
6. Use `#[gpui_form(default = ...)]` for initial values, `#[gpui_form(skip)]`
   for model fields that should not render, and
   `#[gpui_form(type = ..., from = ..., into = ..., component(...))]` when the
   UI edits a form-side type differing from the model field. Text input
   prototyping parses non-`String` form-side types with `FromStr`.
7. Use paths such as `gpui_form::date_picker`, `gpui_form::file_picker`, and
   `gpui_form::infinite_select` for helper state and compatibility modules.

## Component Selection

- `input` for text-like fields; `number_input` for numeric fields
  (`number_input(as = f64)` for a different numeric representation).
- `checkbox` or `switch` for `bool` fields.
- `select` for a single enum-like choice (plain, `searchable`, or `partial`);
  derive `SelectItem`, plus `EnumIter` when choices come from iteration.
- `infinite_select` for cascading/nested enum trees; derive `InfiniteSelect`
  and `PartialEq` on the enum tree.
- `phone_input` for phone fields (`phone` feature); bare form validates any
  globally valid number, `phone_input(country = <field>)` binds a sibling
  country-select field.
- `date_picker` for single dates; `file_picker` for native path selection on
  `PathBuf` fields.
- `custom(...)` when the app owns the state/widget contract (state derive,
  `custom_component_shape!`, optional `value_binding`).
- `nested` to compose another `#[derive(GpuiForm)]` struct: the holder stores
  the nested form's `<Name>FormValueHolder`; add `#[koruma(nested)]` on the
  source field to run the nested validators through the outer `validate()`.
- `list(item = <scalar kind>)` to edit a `Vec<T>`: the holder gains
  `add_<field>` / `remove_<field>` / `<field>_items`, and `each(...)`
  element validators report failures per item index. v1 is scalar items
  only (`item = nested` and `item = custom(...)` are parse errors).

Full syntax inventory: root `README.md` §Component Syntax, or
`references/api-map.md`.

## Common Patterns

- For selects, derive `SelectItem` on enum-like values and `EnumIter` when the
  app needs iteration-backed choices.
- For cascading or nested selects, derive `InfiniteSelect` and `PartialEq` on
  the enum tree and use `#[gpui_form(component(infinite_select))]`.
- For custom widgets, derive `CustomComponentState` on a state type or declare
  a reusable shape with `gpui_form::custom_component_shape!`.
- For value-bound custom widgets, implement
  `gpui_form::custom::CustomComponentValueAdapter<T>` on the shape and use
  `component(custom(shape = ..., value_binding))`.
- For typed field naming (validation, dirty tracking, focus, analytics, schema
  export), use the generated `<Name>FormPath` constructors such as
  `UserProfileFormPath::username()`; skipped fields have no constructor.
- For nested forms, name the root with `<field>_root()` and compose deeper
  paths with `join`:
  `UserFormPath::address_root().join(&AddressFormPath::city())` names
  `address.city`. Route nested errors with `error_for` over any
  `<field>...` path, and re-key into the nested form's own map with
  `<field>_errors()` when you need the inner validator kinds.
- For repeated fields, name an item with `<field>(index)` — a typed index
  segment, never a formatted string (`tags.2`). `error_for(&TagsFormPath::tags(2))`
  is `Some` only when item 2 failed; `tags_element_error(2)` exposes the
  element error directly. Touched/dirty track the whole list through
  `<field>_root()` in v1.
- For layout intent, attach non-rendering hints with `section`, `label`,
  `description`, `placeholder`, and `width`.
- For submit flows, add `#[gpui_form(submit)]` and drive the generated
  `<Name>FormSubmission` tracker yourself: `begin()` → your async save →
  `finish(result)`. Gate the button on `can_submit()` plus the facade's
  `is_valid()`; render failures through `last_error()` re-keyed with
  `<Name>FormFieldErrors::from_koruma_errors`.
- For live validity, call `state.validate()` from change handlers and read
  `state.is_valid()` in render (cached verdict when fresh, accurate
  recompute when stale — safe in render paths).
- For async checks ("is this username taken?"), implement
  `gpui_form::AsyncFieldValidation<Holder>` (one impl per form, `Ok(())` for
  unhandled fields) and drive it from your own spawn; own debounce and
  latest-wins cancellation app-side.
- Keep consumer code focused on app models, form state, rendering, and
  app-owned components.

## Feature Gotchas

### Phone (`phone` feature)

Headless validation lives in `gpui_form::phone`:
`validate_phone_number` (any valid global number),
`validate_phone_number_for_country_label` (parsed country must match the
selection), `validate_optional_phone_number` /
`validate_required_phone_number` for explicit empty handling, plus
`_for_country_label` variants. Implement `gpui_form::phone::PhoneCountry` on
an app country enum once instead of duplicating parser and country-matching
logic per UI. Details: root `README.md` §Phone Number Validation.

### Serde and dirty tracking (`serde` feature)

Enable the facade `serde` feature and wrap the holder in
`gpui_form::FormState` for save/restore and dirty tracking. `FormState` itself
is re-exported unconditionally; only the holder serde derives need the
feature. Scope: `FormState` stores holder data only (no runtime UI state, no
undo/redo); its own dirty/diff is boolean-level; holders with
`#[gpui_form(skip)]` fields round-trip through serde but cannot fully
reconstruct the source struct (per-field passthrough is backlog #15). For
field-level state, use the generated `<Name>FormState` facade instead: it
composes `FormState` + `gpui_form::TouchedFields` and exposes `touch`,
`is_touched`, `touched_fields`, `is_field_dirty`, `changed_fields`, and
`patch()` (a `<Name>FormPatch` with one `Option<holder-field-type>` slot per
non-skipped field; `Some(None)` = changed-to-cleared). Reset semantics:
`reset_to_baseline()` clears touched, `reset_to_baseline_keeping_touched()`
and `sync_baseline()` keep it. Call `touch(&path)` from app-authored blur or
change handlers — the derive does not wire events. Details: root `README.md`
§Saving, Restoring, and Dirty Tracking.

### Nested forms and repeated fields

- The nested form must derive `GpuiForm` itself and resolve by the
  `<Name>FormValueHolder` naming convention — a nested field whose type is
  not a derived form fails at name resolution in YOUR crate (the derive
  cannot check it).
- Nested error recursion (`<field>_errors()`) requires the nested form to be
  a non-generic koruma form; plain `error_for` routing over the nested slot
  works for every nested field.
- v1 list boundary: `Vec<T>` only, add/remove only (no reorder), scalar item
  components only. Per-item input entities are app-authored — the derive
  emits holder operations and metadata, not per-item subscriptions.
- List-item paths have no dense touched index in v1; item-level touched
  tracking arrives with the post-v1 milestone.

### Typed field paths

Every form emits `<Name>FormPath`, a typed newtype over
`gpui_form::FieldPath` (no feature flag, no GPUI, no serde). One constructor
per non-skipped field; skipped fields have none. The same type carries
`FIELD_NAMES`, `FIELD_COUNT`, `all_paths()`, `field_index()`, and
`path_for_index()` for declaration-order iteration and dense index mapping
(single-segment flat paths only; unknown or hand-built multi-segment paths map
to `None` and are inert in field-level state). Hand-built multi-segment paths
via `<Name>FormPath::new(&["a", "b"])` work today; typed nested/list
composition arrives with backlog #2/#3. Koruma forms also gain path-keyed
error access: `holder.validate_fields()` →
`errors.error_for(&<Name>FormPath::field())` with per-field `issue_keys()`
(`validation.*` keys; inner newtype/nested failures report
`validation.invalid`). Details: root `README.md` §Typed Field Paths and
§Validation With Koruma.

### Submit lifecycle and live validation

`#[gpui_form(submit)]` (opt-in, no feature flag) emits a
`<Name>FormSubmission` alias over the headless core `SubmitTracker` —
koruma forms alias over the typed koruma error (`ValidateExt::Error`),
others over `gpui_form::submit::BoxError`. The tracker never runs futures:
wrap `begin()` → your future → `finish(result)` in `cx.spawn` yourself.
`begin()` returns `false` while a submit is in flight (double-click guard),
stale completions (after `reset()`, or with no attempt in flight) are
ignored via the generation guard, and `set_error`/`clear_error` inject
server-side errors. Koruma-gated `<Name>FormState` facades additionally
compose `gpui_form::ValidationCache`: call `state.validate()` when
validation should refresh, read `state.is_valid()` anywhere (never lies),
`state.validation()` for the cached map, `invalidate_validation()` as the
manual escape; resets invalidate, `sync_baseline()` does not. Async
validation is contract-only (`gpui_form::AsyncFieldValidation`) — the
`validate(async_fn = ...)` attribute parses/records metadata but nothing
executes it yet. Details: root `README.md` §Submit Lifecycle and Live
Validation.

### Localization (rust-i18n 4)

Same backend as `gpui-kit` widgets, so widget and form text share one active
locale. Workflow: each localizing crate owns its `locales/<locale>.yml` files
and calls `rust_i18n::i18n!("locales", fallback = "en")` — `t!` is
crate-local. Switch app-wide locale through `gpui_form::i18n`
(`init(cx)`, `change_locale(cx, ...)`, `localize_message(cx, key)`); built-in
runtime copy ships en/fr-FR/zh-CN. `#[gpui_form(koruma(fluent))]` and
`#[select_item(fluent)]` resolve `<form>.<field>_label`, `validation.<kind>`,
and `<enum>.<variant>` keys through the app's own locales, falling back to
humanized keys or variant names. `t!` returns `Cow<str>` — deref rather than
calling unstable `str` methods. Details: root `README.md` §Localization
(rust-i18n).

### Layout and section hints

`section`, `label`, `description`, `placeholder`, `width = full | half |
third` are metadata-only in v1: they describe intent, they do not drive GPUI
rendering. Section grouping is order-preserving. Hints on
`#[gpui_form(skip)]` fields are ignored. Prototyping groups by `section`,
prefers `label`, and emits `description` where it already produces help text;
the width enum is re-exported as `gpui_form::LayoutWidth`. Details: root
`README.md` §Layout and Section Hints.
