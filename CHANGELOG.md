# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.6.0] - 2026-10-11

Milestone 1 of the form feature-parity plan: the field-level state model
(touched tracking, per-field dirty/patch, typed error access), keyed by the
existing typed field paths. All additions are additive; the minor bump follows
the plan's one-minor-per-milestone rule.

### Added

- `TouchedFields` in `gpui-form-core` (`gpui_form::TouchedFields` from the
  facade): a dense, form-agnostic touched-field bitset keyed by
  declaration-order index. Unknown indices are a no-op on every mutator and
  `false` on every query. Touched state is transient UI-session state and is
  never serialized.
- `<Name>FormState`, generated for every `#[derive(GpuiForm)]` form: a facade
  composing `gpui_form::core::FormState` + `TouchedFields` behind typed
  `<Name>FormPath` accessors — `touch`, `is_touched`, `touched_fields`,
  `clear_touched`, `clear_all_touched`, `is_dirty`, `is_field_dirty`,
  `changed_fields`, `patch`, plus value accessors and escape hatches
  (`state()`/`state_mut()`). Reset semantics: `reset_to_baseline()` clears the
  touched set (fresh canvas); `reset_to_baseline_keeping_touched()` and
  `sync_baseline()`/`replace_current()` keep it. Holders stay plain data; all
  state lives on the facade.
- `<Name>FormPatch`, generated per form: one `Option<holder-field-type>` slot
  per non-skipped field (`Some(None)` = changed to cleared) with
  `between`/`is_empty`/`changed_fields`/`apply_to`. Under the derive crate's
  `serde` feature it also derives `Serialize`/`Deserialize` with
  `skip_serializing_if = "Option::is_none"` per field, so the wire format is
  the PATCH tri-state (absent = unchanged, `null` = cleared, value = set).
- Declaration-order tables on `<Name>FormPath`: `FIELD_NAMES`, `FIELD_COUNT`,
  `all_paths()`, `field_index()`, and `path_for_index()`. Dense indices exist
  only for FLAT v1 single-segment paths naming a real field; hand-built
  multi-segment or unknown paths map to `None` and are inert in field-level
  state.
- Typed error access for koruma forms (any enablement route — struct-level
  `#[gpui_form(koruma)]` or field-level `#[koruma(...)]` attributes):
  `holder.validate_fields()` returns a `<Name>FormFieldErrors` map re-keying
  koruma's typed per-field error structs by `<Name>FormPath`, with
  `error_for(&path)`, typed per-field accessors, `failing_fields()`, and an
  `issue_keys()` helper on the `<Name>FormFieldError` borrowing enum mapping
  failed field-level validators to existing `validation.*` i18n keys. Failures
  nested inside `#[koruma(newtype)]`/`#[koruma(nested)]` inner types report
  the generic `validation.invalid` key and stay reachable through the typed
  accessors. The raw koruma `validate()` is untouched
  (`map.is_empty()` == `validate().is_ok()`).
- `FormFieldErrors` trait in `gpui-form-core`
  (`gpui_form::FormFieldErrors`): the form-agnostic contract
  (`is_empty`/`failing_paths`) implemented by every generated
  `<Name>FormFieldErrors` map, for code generic over any form's errors.

### Changed

- The `forms-demo` example now drives the generated `SignupFormState` facade:
  inputs mark touched on blur and other component kinds on change, the footer
  lists the live changed/touched fields through `FieldPath` display, per-field
  errors render through `error_for` + `issue_keys`, and rendering no longer
  runs koruma validation per frame or builds a throwaway dirty-check
  `FormState` + holder clone (validation is recomputed on the Validate button
  and on change once displayed).
- `gpui-form-core` scope notes updated: `FormState<H>` itself stays
  boolean-level; the field-level diff and touched tracking live on the
  generated facade.

### Documentation

- Documented the field-level state model in the root README (§Saving,
  Restoring, and Dirty Tracking; §Typed Field Paths; §Validation With Koruma),
  the core and derive READMEs, the core/derive/facade architecture notes, the
  `use-gpui-form` skill and its API map, and
  `docs/form-state-persistence.md`.
- Marked feature-backlog items #6 (typed error access) and #9 (patch and
  delta generation) as shipped and refreshed the suggested order.

[0.6.0]: https://github.com/stayhydated/gpui-form/releases/tag/v0.6.0

## [0.5.2] - 2026-06-30

### Added

- `component(phone_input)` form field, behind the `phone` feature. It renders as
  a text `Input` and stores its value as `Option<String>`. Use the bare
  `phone_input` for any globally valid number, or `phone_input(country = <field>)`
  to match a sibling country-select field. The country binding is stored as
  `PhoneInputBehaviour::country_field` metadata, and the generated control
  validates a globally parseable number as a baseline. Empty input is accepted so
  a half-typed field is not flagged while the user is still typing. The component
  is wired through `gpui-form-schema`, `gpui-form-codegen`, `gpui-form-derive`,
  and `gpui-form-prototyping-core`.
- `PhoneCountry` trait in `gpui_form::phone`, with `phone_country_id()` and
  `phone_country_label()`, plus a `validate_phone_number_for(raw, &country)`
  helper. An application country enum maps to a libphonenumber id and label once
  instead of matching at every call site.
- Optional and required phone helpers. `validate_optional_phone_number` accepts
  empty input; `validate_required_phone_number` rejects it. Both have a
  `_for_country_label` variant, and there is a new
  `PhoneNumberValidationError::Required`.
- Result-inspection helpers on `PhoneNumberValidation`: `is_empty`,
  `is_valid_or_empty`, `validated`, and `country`.

### Changed

- The `Phone Verification` example story now shows the parsed result as separate
  rows for status, country, and E.164 instead of one status string, and uses the
  new `PhoneCountry` trait and helpers.

### Documentation

- Documented both phone modes (global and country-bound) and the new helpers in
  the root, facade, core, and derive READMEs, the supported-component lists, the
  `use-gpui-form` skill, and the codegen and schema architecture notes.
- Marked the phone part of feature-backlog item #18 as shipped.

[0.5.2]: https://github.com/stayhydated/gpui-form/releases/tag/v0.5.2
