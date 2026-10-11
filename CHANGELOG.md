# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.0] - 2026-10-11

Milestone 2 of the form feature-parity plan: validation and submit lifecycle
(validation-state cache with a live `is_valid`, the submit lifecycle tracker,
and the async validation contract). All additions are additive; the minor bump
follows the plan's one-minor-per-milestone rule.

### Added

- `SubmitTracker<E>` in `gpui-form-core` (`gpui_form::SubmitTracker` from the
  facade, module `gpui_form::submit`): the headless submit lifecycle state —
  `begin()` re-entry guard (false while a submit is in flight, and the only
  thing that counts an attempt via `submit_count()`), `is_submitting()` /
  `is_submitted()` / `is_submit_successful()` / `can_submit()`,
  `last_error()` / `take_last_error()`, `abandon()`, `set_error` / `clear_error`
  for server-side error injection, and `reset()`. A generation/epoch guard
  ignores stale async completions: a `finish` arriving after `reset()` (or with
  no attempt in flight) changes nothing.
- `SubmitAction<H>` in `gpui-form-core` (`gpui_form::SubmitAction`): the
  executor-free submit hook contract (`async fn submit(&mut self, holder: &H)`
  with `Output`/`Error` associated types, no `Send` bound). Neither type runs
  futures — driving them (GPUI `cx.spawn`, tokio) is application code, and the
  holder flows through submit by `&H` untouched.
- `submit::BoxError` (`gpui_form::BoxError`): the default submit-failure alias
  for forms without a koruma error type.
- `#[gpui_form(submit)]` (opt-in, no feature flag): emits
  `pub type <Name>FormSubmission = SubmitTracker<E>;` per form, spliced into
  both expansion branches (empty forms included). `E` is the holder's typed
  koruma error (`<Name>FormValueHolder as koruma::ValidateExt>::Error`) when
  the holder derives `Koruma` and the struct is non-generic, and `BoxError`
  otherwise. Forms that do not opt in see zero emitted diff; the holder stays
  plain data.
- `ValidationCache<H, O>` in `gpui-form-core`
  (`gpui_form::ValidationCache`): the outcome cache behind a live `is_valid`.
  Freshness is snapshot-compare — the cache stores the validated holder
  snapshot next to the outcome and a read is fresh iff `current == snapshot` —
  so it stays sound under every mutation route (including `&mut` escapes) with
  zero invalidation discipline. `stored()` is the exact-read companion that
  skips the comparison (a holder containing NaN never compares equal to its
  own clone). No serde surface; the cache is transient by construction.
- Cached validation methods on koruma-gated `<Name>FormState` facades (gated
  on `needs_koruma_derive` — either koruma enablement route — and non-generic
  source structs, the same combined gate as the `<Name>FormFieldErrors` map
  the cache stores; empty forms never emit it): a `validation` slot of
  `ValidationCache<Holder, <Name>FormFieldErrors>` plus
  `validate(&mut self) -> &<Name>FormFieldErrors` (compute, store, return),
  `validation(&self)` (fresh-only cached read), `is_valid(&self)` (cached
  verdict when fresh, otherwise a fresh compute WITHOUT caching — safe in
  render paths), `invalidate_validation(&mut self)`, and
  `error_for(&mut self, &path)` (validate-and-cache on miss).
- `AsyncFieldValidation<H>` in `gpui-form-core`
  (`gpui_form::AsyncFieldValidation`, module
  `gpui_form::async_validation`): the executor-free async per-field
  validation contract — `async fn validate_field(&self, holder: &H, field:
  &FieldPath) -> Result<(), Self::Error>`, no `Send` bound. One impl can
  serve a whole form by returning `Ok(())` for unhandled fields. Debounce,
  latest-wins cancellation, and merging async outcomes into the sync error
  map are deliberately deferred to the driving layer.
- `validate(async_fn = <path>)` field attribute (emission sketch): parses and
  records the async validator path as metadata on
  `FieldVariant::async_validator` (new public field, `Option<&'static str>`,
  default `None`) via the const-callable `with_async_validator` builder.
  Nothing executes the path yet; the reserved `async-validation` feature
  flag (default off) exists in the facade and derive crates so the future
  wiring has a stable name.
- Facade re-exports: `SubmitTracker`, `SubmitAction`, `BoxError`,
  `ValidationCache`, `AsyncFieldValidation`, and the `submit` /
  `async_validation` modules.

### Changed

- Koruma-gated facades invalidate the validation cache explicitly in
  `current_mut()`, `replace_current()`, and both reset variants (a
  reverted-to-same-value form does not keep the old verdict);
  `sync_baseline()` deliberately does not invalidate, and `state_mut()`
  remains the documented escape hatch that the snapshot comparison keeps
  sound (with `invalidate_validation()` as the manual recourse).
- The `forms-demo` example wires the M2 surface end to end: a Submit button
  drives the opt-in `SignupFormSubmission` tracker (`begin()` → a spawned
  `SubmitAction` future with simulated latency → `finish()`), disables while
  submitting, syncs the baseline on success, and renders `last_error`
  banners by re-keying the typed koruma error through
  `SignupFormFieldErrors::from_koruma_errors` into localized `validation.*`
  keys; a simulated async availability check on the email field implements
  the `AsyncFieldValidation` contract with a generation guard against stale
  completions; and the facade's validation cache feeds a live `is_valid`
  footer indicator without running koruma validation in render.

### Documentation

- Documented the submit lifecycle and live validation in the root README
  (new §Submit Lifecycle and Live Validation), the core and facade READMEs,
  the core/facade/derive architecture notes, the `use-gpui-form` skill and
  its API map, and `examples/README.md`.
- Marked feature-backlog item #7 (submit lifecycle helpers) as shipped and
  refreshed the suggested order.

[0.7.0]: https://github.com/stayhydated/gpui-form/releases/tag/v0.7.0

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
