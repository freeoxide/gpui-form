# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] — port onto upstream 0.6.0

Synchronized with `stayhydated/gpui-form` `0.6.0` (merge-base `c4d7e63`, 159
upstream commits) and re-ported every fork feature onto the new
component-shape architecture. See upstream's documentation
(<https://stayhydated.github.io/gpui-form/book/>) for the new attribute
grammar (`component(MyShape)` / `hidden` / `skip` field intents), the
`gpui-form-collection` / `gpui-form-runtime` / `gpui-form-mcp` crates, MCP
form tools, and the mdBook/web pipeline that arrived with 0.6.0.

### Re-ported fork features

- **Phone input** is now a component shape:
  `component(gpui_form_collection::phone_input::PhoneInput)` behind the
  `phone` feature on `gpui-form-collection` (which enables
  `gpui-form-core/phone`). It renders a text `Input` with widget-level
  libphonenumber syntax validation; empty input stays accepted mid-entry.
  The old `component(phone_input(country = <field>))` keyword and its
  `country` metadata were removed with the built-in component registry —
  country matching is a validation-layer concern via the `gpui_form::phone`
  helpers (`validate_phone_number_for`, `PhoneCountry`,
  `validate_*_for_country_label`), demonstrated by the Phone Verification
  story and `examples/some-lib/tests/phone_input_component.rs`.
- **Form-state persistence + dirty tracking** (`gpui_form::FormState`) is
  unchanged and now compiles against the new holder conversion surface
  (`TryFrom` / `try_into_original` for shape-backed fields without defaults).
- **Value-holder `PartialEq`** is now opt-in via the new
  `#[gpui_form(partial_eq)]` container option, which emits a where-bounded
  manual impl (like the hand-generated `Clone`/`Debug`/`Default`). A blanket
  derive or unconditional impl would break forms whose storage types are not
  `PartialEq` (rustc eagerly rejects provably-unsatisfiable bounds on
  concrete types), which upstream 0.6.0 requires to compile — so forms opting
  in get a working `FormState::is_dirty`, and forms that do not simply have
  no holder `PartialEq`. The `serde` feature still opts the holder into
  `Serialize`/`Deserialize`.
- **Typed field paths** (`<Name>FormPath`, feature #8 FLAT v1) are generated
  from the same component-field list as the new `<Name>FormField` enum, and
  the two are bridged: `<Name>FormPath::from_form_field(variant)`.
- **Layout metadata** (`section = "..."`, `placeholder = "..."`,
  `width = full|half|third`) parses in the new `attrs` grammar and lands on
  `gpui_form::schema::registry::FieldVariant`
  (`with_section`/`with_placeholder`/`with_width`, `LayoutWidth` re-exported
  from the facade). `label`/`description` now use upstream's native
  MCP-metadata attributes instead of the fork's duplicates. The prototyping
  generator emits section headings from the `section` hint.
- The `number_input(as = ...)` parse override is superseded upstream by
  `gpui_form_collection::number_input::NumberInput::<T>` (and
  `ParsedInput<T, Config>`); the fork's fix is dropped with the mechanism it
  patched.

### Removed

- `vendor/gpui-storybook` and `vendor/gpui-es-fluent` path members — replaced
  by upstream's pinned `stayhydated` git revisions.
- Fork copies of `crates/*/docs/ARCHITECTURE.md` and
  `.agents/skills/use-gpui-form` — superseded by upstream's `book/` and
  top-level `skills/`.

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
