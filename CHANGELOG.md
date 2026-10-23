# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.11.0] - 2026-10-23

First release published to crates.io under the `freeoxide-gpui-form*`
names. Carries the locale single-source-of-truth rework, the
`set_state_value` contract change, and the GPUI stack bumps since
0.10.0; the breaking item follows the project's 0.x minor convention.

### Added

- Runtime selectable bounds for the date pickers: `DatePickerState` and
  `DateRangePickerState` take `min_date`/`max_date` options (and the
  underlying `Calendar` element gained the same builders). Days outside the
  bounds render disabled through the calendar's existing disable machinery
  and reject clicks; one-sided and default (unbounded) behavior is pinned by
  click-through tests. Bounds are a runtime capability only — schema/codegen
  integration is deliberately left for a future feature change.

### Changed

- **Localization has one source of truth: the process-global `rust-i18n`
  locale.** The `I18n` GPUI global no longer keeps its own locale copy
  (previously synced only by `select_language`/`install`, so a direct
  `rust_i18n::set_locale` was invisible to the bridge while component `t!()`
  calls followed the global — the two could drift). `I18n` now delegates
  every read to `rust_i18n::locale()` and every write goes through
  `rust_i18n::set_locale` exactly once; `init` no longer resets the locale
  to `"en"`, and `I18n::new_with_language` now selects the global locale.
  The `gpui-form-i18n` and `gpui-form-component` locale tables are
  reconciled to identical key sets (the bridge crate gains the pluralized
  `file_picker.paths_selected_one`/`_other` keys and drops the flat
  `paths_selected` key the runtime never read); parity tests cover the
  reconciled keys in both crates, and a GPUI test pins that `change_locale`,
  `localize_message`, and `t!()` always agree.
- Derive-emitted numeric metadata gains its consumer: the recorded
  `NumberInputBehaviour.kind` and the emitted input validator branch are now
  produced by ONE shared parse-time classification in `gpui-form-codegen`
  (`number_input_kind`), so a `number_input(as = ...)` field cannot record
  one numeric family and validate as another. A codegen test pins the
  kind/validator agreement across bare and `as`-override fields; the field
  stays undeprecated (it is the schema-export wire surface, and the derive
  emits its construction into user crates).
- CI lint gates now combine strictness with feature coverage: the `clippy`
  job runs `cargo clippy --workspace --all-features --all-targets -- -D
  warnings` (it previously ran without `-D warnings`), and the publish
  workflow's release gate adds `--all-features` to its `-D warnings` clippy
  run. Neither gate previously covered both axes.
- **Breaking (0.x minor): `CustomComponentValueAdapter::set_state_value` is
  now a required method.** The `value_binding` opt-in generates calls to it
  (seeding component state from the form value), and the defaulted no-op let
  opt-ins compile while silently dropping the form value. Implementations
  must now provide the hook — an intentionally inert component writes an
  explicit empty body, so the omission is visible at the impl site. A
  compile-fail trybuild fixture (`gpui-form-derive`
  `tests/ui/custom_component_missing_set_state_value.rs`) pins the contract.
- Prototyping generation hardening (`gpui-form-prototyping-core`): malformed
  metadata now returns typed `PrototypingError`s instead of panicking or
  emitting non-compiling scaffolds — non-`Vec` list value types return
  `InvalidListValueType` (previously a silent `c_void` fallback), and
  `ImportSet::to_token_stream` is fallible (`InvalidImports`, collecting all
  offenders) instead of panicking on malformed import strings.
  `component(list(..., add_label = "..."))` overrides now reach the generated
  scaffold's Add button (previously derive-side only), and
  `collect_i18n_keys` omits the `{form}.{field}_add` key when the override
  is set.
- Derive-emitted schema metadata spells type strings in canonical Rust form
  (`"Vec<String>"`, not the proc-macro printer's `"Vec < String >"`), and the
  non-type exported strings now follow the same rule: `default_expr`,
  `from_expr`/`into_expr`, and the async-validator path export as
  `"crate::defaults::country()"`, not `"crate :: defaults :: country ()"`
  (validation rule names and the M4 sibling names were already plain
  identifiers). The pinned export fixtures were updated deliberately.
- Derive-emitted items (`<Name>FormPath`, `<Name>FormState`,
  `<Name>FormPatch`, the schema export, the submit alias) carry one-line
  summary docs instead of multi-paragraph essays; the value-holder struct
  itself emits no doc at all (it is generator-owned plumbing named
  `<Name>FormValueHolder`). The removed detail lives in the derive
  generators' source comments, so rustdoc still documents every generated
  item; after the trim the flat expansion snapshot measures 9,786 chars
  (~9.8 KB) with a 29.4% `#[doc]` payload share, down from 45.0% (5,806 of
  12,911 chars) before the trim.
- Bumped the GPUI stack: `gpui-kit`/`gpui-kit-assets` 0.7.0 → 0.7.3 and
  `gpui-pre`/`gpui-pre-platform` 0.3.7 → 0.3.9 (0.7.1+ hard-pin the newer
  `gpui-pre`, so the two move together). No API changes in this workspace;
  406 workspace tests, clippy, fmt, and the release `forms-demo` build are
  green on the new stack. `gpui-kit` 0.7.3 re-exports itself as `gpui`, so
  `forms-demo`'s `use gpui_kit::*;` glob now collides with the extern crate —
  its gpui import is pinned with a leading `::`. The two demo-level focus
  workarounds stay: 0.7.3 does not change the Input frame-focus split
  (upstream longbridge/gpui-kit#3183 remains open).
- Bumped the GPUI stack again: `gpui-kit`/`gpui-kit-assets` 0.7.3 → 0.8.0.
  No source changes in this workspace; workspace check, clippy, and test
  gates are green on the new stack.

### Removed

- **The `async-validation` feature flags on `freeoxide-gpui-form` and
  `freeoxide-gpui-form-derive`.** They gated literally nothing: the
  `validate(async_fn = ...)` attribute parses and records metadata
  unconditionally, and no code keyed on either flag. Finishing async
  validation execution remains a future feature; a wiring phase that needs a
  gate can re-introduce one when emission actually lands.

[0.11.0]: https://github.com/freeoxide/gpui-form/releases/tag/v0.11.0

## [0.10.0] - 2026-10-11

Milestone 5 of the form feature-parity plan: schema export (backlog #14). A
read-only projection of the finished metadata model — the same
`FieldVariant`/`FieldLayout` chain the `inventory` feature registers — into an
owned, serializable document for documentation generation, test fixtures,
prototyping tools, and non-GPUI consumers. All additions are additive; the
minor bump follows the plan's one-minor-per-milestone rule.

### Added

- `<Name>FormSchema::schema()` (backlog #14): every `#[derive(GpuiForm)]`
  form — including `#[gpui_form(empty)]` forms, and with no `inventory`
  feature enabled — emits a `<Name>FormSchema` unit type whose `schema()`
  accessor projects the form's shape chain into a
  `gpui_form::schema::registry::FormSchema` document. The derive passes the
  identical `GpuiFormShape` construction tokens to both the
  `inventory::submit!` registration and the export's `SHAPE` const, so the
  export cannot drift from the registered metadata.
- Export view types in `gpui-form-schema` (`registry::FormSchema`,
  `FormSchemaField`, `FormSchemaComponent`, `FormSchemaLayout`) with
  `From<&GpuiFormShape>`/`From<&FieldVariant>` conversions and convenience
  accessors (`field`, `field_names`, `has_validations`, `is_nested`,
  `is_list`, `kind`). Field entries cover component behaviour payloads
  (select flags, phone country sibling, number validation type, nested form
  struct, list item kind/add label), value and conversion types, validation
  rule names, defaults, custom component hints, layout hints, the async
  validator path, and the M4 `depends_on`/`required_if` payloads.
- `gpui_form::FormSchema` root re-export on the facade (entry types stay
  under `gpui_form::schema::registry`).
- `schema_markdown(&FormSchema)` in `gpui-form-prototyping-core`: renders the
  exported document as a Markdown reference table (docs/fixture generation).
- forms-demo: a footer "Schema" debug toggle printing the exported signup
  schema — the devtools seed — plus the `app.schema` locale key in
  en/fr-FR/zh-CN.

### Changed

- `serde` is now a direct (non-optional) dependency of `gpui-form-schema`:
  the export view types — and `ComponentKind`, `NumberInputKind`, and
  `LayoutWidth` — derive `Serialize`/`Deserialize` unconditionally, so
  schema export round-trips with no feature flag anywhere in the chain.
  This is deliberately separate from the facade's opt-in `serde` feature,
  which remains about holder persistence. The wire format uses snake_case
  component tags matching `ComponentKind`'s `Display` and omits absent
  `Option` hints (`skip_serializing_if` + `default`).
- Every form's expansion gains the additive `<Name>FormSchema` item; the
  flat-form expansion snapshots were regenerated accordingly.
- Package rename for publishing under the `freeoxide` account: all nine
  workspace packages now publish as `freeoxide-gpui-form`,
  `freeoxide-gpui-form-core`, `freeoxide-gpui-form-codegen`,
  `freeoxide-gpui-form-component`, `freeoxide-gpui-form-component-derive`,
  `freeoxide-gpui-form-derive`, `freeoxide-gpui-form-i18n`,
  `freeoxide-gpui-form-prototyping-core`, and `freeoxide-gpui-form-schema`
  (versions stay `0.10.0`; the fresh names have no crates.io collision).
  Zero source churn by design: every crate keeps its current lib target
  name (`gpui_form`, `gpui_form_core`, `gpui_form_component`, ...), so all
  `::gpui_form::...` paths in generated code, examples, and user code
  compile unchanged, and in-workspace dependency keys keep their
  historical names through `package =` remaps in the workspace dependency
  table. The repository URL is now
  https://github.com/freeoxide/gpui-form.

[0.10.0]: https://github.com/freeoxide/gpui-form/releases/tag/v0.10.0

## [0.9.0] - 2026-10-11

Milestone 4 of the form feature-parity plan: field dependency metadata
(backlog #12) and conditional requiredness (backlog #13). The partial-holder
item (backlog #10) is closed WITHOUT new surface: decision [D10] evaluated
the 0.6.0 `<Name>FormPatch` tri-state (`None` = unchanged, `Some(None)` =
cleared, `Some(value)` = set; serialized absent/`null`/value) and found it
already expresses cleared-vs-unchanged, so "send only touched fields" is
patch-filtering by `touched_fields()` — no `#[gpui_form(partial_holder)]`
type is generated. All additions are additive; the minor bump follows the
plan's one-minor-per-milestone rule.

### Added

- `#[gpui_form(required_if = <field>)]` (backlog #13): conditional
  requiredness on flat fields whose holder stores `Option<T>`. The attribute
  names a sibling field of the same struct — a bare ident
  (`required_if = subscribe`) or the quoted form (`required_if =
  "subscribe"`) — existence-checked at derive time, with no expression
  language (the plan's R19). Controlling-sibling shapes resolved at derive
  time: `bool` (required when `true`), `Option<bool>` (required when
  `Some(true)` — an explicit off does not require), and any other
  `Option<T>` (required when set). Both dependency attributes require a
  `component(...)` on the carrying field.
- The `required_if` desugaring: one generated koruma validator per form
  (`<Name>FormRequiredIfValidation<T>`, following the reserved
  `<Name>Form*` prefix), mirrored onto the holder field as
  `#validator::<Option<_>>::builder().when(<sibling truthiness>)` so
  koruma's own `validate()` constructs it with the controlling sibling's
  live holder value. Violations surface through the M1 typed error surface
  — `validate_fields()`, `error_for(&<Name>FormPath::field())`,
  `issue_keys()` reporting the shared `validation.required` key (no new
  locale keys) — not as display-only metadata. The rule composes with
  mirrored `#[koruma(...)]` validators on the same field (each reports its
  own key), forces the koruma derive for the form, and suppresses the
  synthetic unconditional `RequiredValidation` (requiring the field always
  would defeat the condition). Forms using it require `koruma` as a direct
  dependency, like every koruma-gated form.
- `gpui_form::core::required_when(requires, present)` — the pure
  conditional-required predicate every desugared validator delegates to
  (re-exported as `gpui_form::required_when`); core keeps no rule engine
  and no expression evaluation.
- `#[gpui_form(depends_on = <field>)]` (backlog #12): records the sibling
  relationship as schema metadata — `FieldVariant::depends_on` stores the
  sibling's field name (the `FieldPath` name-segment representation) — for
  conditional loading, dependent validation, and reset-on-parent-change
  consumers. The derive emits no event wiring ([D5]).
- `gpui-form-schema`: `FieldVariant` gains `depends_on` /
  `required_if` payloads with `with_depends_on` / `with_required_if` const
  builders and getters (`&'static str`, `Copy` payloads only — the
  `inventory::submit!` const-constructibility contract is preserved), and
  the `"RequiredIfValidation"` rule kind rides the `FieldVariant`
  validations list for required-if fields.
- `gpui-form-prototyping-core`: scaffolds consume `depends_on` metadata for
  reset-on-parent-change placeholders — the parent component's generated
  change handling resets every dependent field's holder value (`None` for
  Option-wrapped holders, `Default` otherwise) across the input,
  number_input, phone_input, select, infinite_select, date_picker,
  file_picker, switch, and checkbox generators. Re-syncing the dependent's
  widget stays app-authored; fields without dependents emit identical
  tokens.
- `forms-demo`: a `state` select recording `depends_on = country` that the
  country handler resets (holder value and widget) on every change; the
  newsletter checkbox flipping email between optional and required through
  `required_if = newsletter` with the localized typed error; and a "Save
  changes (touched only)" button printing the PATCH tri-state filtered to
  the touched fields (the [D10] outcome in app code). `some-lib`'s `User`
  carries both attributes as the example shapes.

### Changed

- Nothing breaking. Generated emission for forms that use neither attribute
  is byte-identical (pinned by the flat-form expansion snapshot plus a
  zero-diff unit test); a form WITH `required_if` additionally derives
  `Koruma` on its holder and gains the validator type in its module.
- `FieldOptionality::needs_required_validation()` (derive-internal) returns
  `false` for fields carrying `required_if`.

### Documentation

- Root `README.md`: new "Field Dependencies and Conditional Requiredness"
  section (worked example, desugaring rules, reset-on-parent-change
  sketch), attribute inventory entries, and the touched-filtered patch
  payload documented in "Saving, Restoring, and Dirty Tracking" (the [D10]
  outcome — no separate partial-holder section).
- `crates/gpui-form-derive/README.md` attribute inventory;
  `crates/gpui-form-core/README.md` helper list;
  `crates/gpui-form-prototyping-core/README.md` field-dependency
  scaffolding note.
- `.agents/skills/use-gpui-form`: Common Patterns + a Feature Gotchas
  subsection; `references/api-map.md` attribute list + a new pattern block.
- ARCHITECTURE notes updated in `gpui-form-derive` (parsing, sibling
  resolution, desugaring, D10 outcome), `gpui-form-codegen` (field-level
  attribute placement scope note), `gpui-form-schema` (payloads),
  `gpui-form-core` (`required_when`), and
  `gpui-form-prototyping-core` (dependent resets).
- `examples/README.md` covers the new some-lib shapes and the forms-demo
  M4 showcase; `docs/feature-backlog.md` marks #12/#13 shipped (M4),
  closes #10 by #9 per [D10], and updates the suggested order.

[0.9.0]: https://github.com/freeoxide/gpui-form/releases/tag/v0.9.0

## [0.8.0] - 2026-10-11

Milestone 3 of the form feature-parity plan: nested forms (backlog #2) and
repeated `Vec` fields (backlog #3) with typed composed paths. This milestone
carries the plan's two budgeted breaking changes (decision [D8]'s core
`PathSegment` representation and the `gpui-form-schema` enum-variant
additions), which is why it takes the minor bump on its own.

### Added

- `component(nested)` (backlog #2): composes another `#[derive(GpuiForm)]`
  struct. The generated holder stores the nested form's own
  `<Name>FormValueHolder` (Plain storage; `Option<Nested>` stays
  `Option`), conversions compose through the nested form's `From` impls,
  and `#[koruma(nested)]` on the source field runs the nested form's
  validators through the outer `validate()` — including bare
  `#[koruma(nested)]` with no other validators. Value-only: no leaf
  widget/entity is generated (like checkbox/switch); rendering the group is
  application- or prototyping-authored.
- `component(list(item = <scalar kind>, add_label = "..."))` (backlog #3):
  edits a `Vec<T>` (or `Option<Vec<T>>`). The holder stores the plain item
  `Vec` (v1: no per-item `Option`) and gains `add_<field>(value)`,
  `remove_<field>(index)` (same semantics as `Vec::remove`), and
  `<field>_items() -> &[T]`. `each(...)` element validators stay
  element-scoped on the holder so koruma reports failures per item index
  (`element_errors`). v1 boundaries (parse-time rejections): `item` is
  required, `item = nested` and `item = custom(...)` are not supported yet,
  and only scalar item kinds parse (`input`, `number_input`, `phone_input`,
  `checkbox`, `switch`, `select`, `infinite_select`, `date_picker`,
  `file_picker`).
- Typed path composition on `<Name>FormPath`: nested fields emit
  `<field>_root()` plus a consuming appender `fn <field>(self)`; list fields
  emit `<field>_root()` plus `fn <field>(index: usize)` carrying a typed
  index segment; every form gains `join(&impl AsRef<FieldPath>)` for
  cross-form composition —
  `UserFormPath::address_root().join(&AddressFormPath::city())` names
  `address.city`, a list item composes the same way (`tags.2.name`). The
  M1 field-level state surface (touched, per-field dirty, patch) keys
  composed fields by their root paths.
- Nested/list typed error access on koruma forms: `error_for` routes any
  `<field>...` path to the nested slot and `<field>` / `<field>.<index>`
  paths on list fields (`Some` for an item path only when that item failed
  its `each(...)` validators — the typed index segment is extracted and
  used, never a string); `<field>_errors()` re-keys a nested field's error
  into the nested form's own `FormFieldErrors` map for the inner validator
  kinds; `<field>_element_error(index)` exposes a list field's element
  error directly; `failing_paths()` expands per failing item index
  (`tags.1`). Flat koruma forms with a bare `#[koruma(nested)]` and no
  other validators keep their exact pre-0.7 emission (no mirrored marker,
  no slot, no new `validate()` failures) — the composed surface is opted
  into with `component(nested)`.
- `PathSegment` in `gpui-form-core` (re-exported as `gpui_form::PathSegment`):
  the typed path-segment enum `Name(&'static str) | Index(usize)` with
  `as_name`/`as_index` and `Display`. `FieldPath` gains the additive
  composition constructors `from_segments`, `push_name`, `push_index`, and
  `join`, plus `AsRef<FieldPath> for FieldPath`; `FieldPath::new` stays the
  names-only constructor so flat-path call sites keep compiling.
- `<Name>FormValueHolderConversionError` gained an additive
  `index: Option<usize>` slot: list-field conversion failures can name the
  failing item (`Display` renders "Failed to convert item N of field X").
  v1 scalar items never fail conversion, so every emitted construction site
  passes `None`; the slot exists for the per-item conversion paths sequenced
  after v1.
- `gpui-form-schema`: `ComponentKind::Nested`/`List` and
  `ComponentsBehaviour::Nested(NestedBehaviour { form_struct })` /
  `List(ListBehaviour { item, item_struct, add_label })` metadata payloads
  (Copy, const-constructible — the inventory contract), plus
  `FieldVariant::is_nested()`/`is_list()` conveniences. Both new kinds are
  value-only, non-subscribable, non-focusable, and never Option-wrap.
  `ComponentKind` also derives `Default` (`Input`) so `ListBehaviour` can
  derive `Default`.
- `gpui-form-prototyping-core`: two composed-field generators — nested
  fields host the nested form's own scaffold as an `Entity<<Nested>Form>`
  scaffold-level struct field inside a bordered group (imported by the
  `crate::<snake>::<Nested>Form` convention through the new
  `ImportItem::owned_path`), and list fields emit a
  `<field>_list_control` member rendering repeatable rows over
  `<field>_items()` with per-row Remove buttons and an Add button appending
  a typed default. Both manage no subscriptions ([D5]/R17 — per-item entity
  churn stays app-authored), and the shared section-heading logic skips
  them. `FormParts` gained the `composed_struct_fields`,
  `composed_struct_inits`, and `composed_impl_members` fragments for
  layouts to splice. `collect_i18n_keys` emits index-free keys for the
  composed fields (a nested group contributes `{form}.{field}_label`; a
  list adds `{form}.{field}_item_label`, `{form}.{field}_add`, and
  `{form}.{field}_remove`).

### Changed

- **Breaking (decision [D8]):** `gpui_form::core::FieldPath::segments()` now
  returns `&[PathSegment]` instead of `&[&'static str]`. Code that compared
  segments against strings (`path.segments() != ["email"]`) must compare
  against `PathSegment::Name("email")`; code that only builds paths through
  `FieldPath::new`/`<Name>FormPath` constructors, renders them, or
  hashes/compares whole paths is unchanged. `Debug` output changed shape
  (`[Name("address")]` instead of `["address"]`); `Display` output for
  name-only paths is identical, and list items render their index
  (`tags.2`). This is the representation change the feature-parity plan
  budgeted to this milestone.
- **Breaking (exhaustive matches):** `gpui-form-schema`'s public
  `ComponentKind` and `ComponentsBehaviour` enums gained the `Nested`/`List`
  variants (pipeline hazard 9). Downstream exhaustive `match`s over either
  enum must add arms or a wildcard; workspace consumers (derive, codegen,
  prototyping-core) were updated in the same change.
- Generated flat-form token shape: the routing matches emitted for
  `error_for`, `is_field_dirty`, and `field_index` now pattern-match
  `PathSegment::Name(..)` instead of bare strings (the D8 consequence); all
  other flat emission is unchanged, pinned by a new flat-form expansion
  snapshot test (`crates/gpui-form-derive/tests/snapshots/`, one golden per
  feature state of the derive crate).
- `gpui-form-prototyping-core`'s `FormParts` gained public fields
  (`composed_struct_fields`, `composed_struct_inits`,
  `composed_impl_members`); layouts that destructure with `..` are
  unaffected.

### Documentation

- Root `README.md`: new "Nested Forms and Repeated Fields" section
  (worked examples for both components, composed paths, per-item error
  access, and the v1 boundaries), component-syntax inventory, and the typed
  field paths scope note updated for `PathSegment` and composition.
- `crates/gpui-form-derive/README.md` and
  `crates/gpui-form-prototyping-core/README.md` updated for the new syntax
  and scaffold fragments; `examples/README.md` covers the new some-lib
  shapes and forms-demo sections.
- `.agents/skills/use-gpui-form`: Component Selection and Common Patterns
  entries for nested/list fields, a gotchas subsection (v1 boundaries,
  naming-convention failure mode, error-recursion requirements), and
  `references/api-map.md` syntax/generated-name updates.
- ARCHITECTURE notes updated in `gpui-form-core` (path module scope),
  `gpui-form-derive` (composition resolution, error routing),
  `gpui-form-codegen` (component model), `gpui-form-schema` (behaviour
  payloads + semver note), `gpui-form-prototyping-core` (composed
  generators, i18n keys, FormParts), and the facade (re-exports).
- `docs/feature-backlog.md`: backlog #2 and #3 marked shipped (M3 v1) with
  scope notes; suggested order updated.

[0.8.0]: https://github.com/freeoxide/gpui-form/releases/tag/v0.8.0

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

[0.7.0]: https://github.com/freeoxide/gpui-form/releases/tag/v0.7.0

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

[0.6.0]: https://github.com/freeoxide/gpui-form/releases/tag/v0.6.0

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
