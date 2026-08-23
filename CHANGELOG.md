# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

[0.5.2]: https://github.com/freeoxide/gpui-form/releases/tag/v0.5.2
