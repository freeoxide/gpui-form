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
- For layout intent, attach non-rendering hints with `section`, `label`,
  `description`, `placeholder`, and `width`.
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
undo/redo); dirty/diff is boolean-level (field-level diff is backlog #9);
holders with `#[gpui_form(skip)]` fields round-trip through serde but cannot
fully reconstruct the source struct (per-field passthrough is backlog #15).
Details: root `README.md` §Saving, Restoring, and Dirty Tracking.

### Typed field paths

Every form emits `<Name>FormPath`, a typed newtype over
`gpui_form::FieldPath` (no feature flag, no GPUI, no serde). One constructor
per non-skipped field; skipped fields have none. Hand-built multi-segment
paths via `<Name>FormPath::new(&["a", "b"])` work today; typed nested/list
composition arrives with backlog #2/#3. Details: root `README.md` §Typed Field
Paths.

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
