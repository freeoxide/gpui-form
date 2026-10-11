# Form feature parity plan

Status: **living** — executable plan for closing the react-hook-form / TanStack
Form feature gaps in `gpui-form` (state, validation, submission, field
collections) without breaking the headless-first architecture.

Produced 2026-10-11 from five independent perspective audits (feature parity,
headless API design, codegen strategy, delivery/workflow planning, adversarial
risk review) over the same repository state; conflicts between perspectives are
resolved in the [Decision log](#0-executive-summary). Companion to
[`feature-backlog.md`](feature-backlog.md), which stays the candidate list —
this document turns the state/validation/submission subset into a sequenced,
validated delivery plan.

## 0. Executive summary

The shipped foundations (`FormState<H>` baseline model, FLAT v1 typed paths,
koruma derive-time validation, opt-in serde) already cover the *hard* part of
web-form-library parity. The gaps are additive — but three of them sit on top
of decisions that must be made once, up front:

1. New per-form state arrives as a **generated `<Name>FormState` facade
   composing standalone core primitives**, never as fields on `FormState<H>`
   or on the value holder ([D1], [D2]).
2. Per-field error access is **generated bridge code over koruma's existing
   typed per-field error structs** — koruma already aggregates per field; the
   gap is path-keyed access and a cache, not error generation ([D3], R7).
3. Nested (#2) and repeated (#3) fields are **gated on deciding the
   `FieldPath` segment representation** (typed index segments vs. a list-item
   path type), because today's segments are `&'static str` and cannot name
   `contacts.2` ([D8], R9).

Delivery is five milestones (M1 field-level state, M2 validation + submit
lifecycle, M3 nested/repeated composition, M4 partial holders + dependency
metadata, M5 schema export), each independently shippable with one minor
bump on the 0.x line and no new cargo features unless a new optional
dependency appears.

### Decision log

| ID | Decision | Driven by |
|----|----------|-----------|
| D1 | `FormState<H>` stays frozen (no new fields, bounds, or methods). Per-form state is a generated `<Name>FormState` facade composing core primitives; every primitive stays usable standalone. | R4, R10 |
| D2 | Holders never gain touched/dirty/error/submit fields or serde surfaces for them. Generated code references only `::gpui_form::…` paths; new runtime types are hand-written in `gpui-form-core` (or `gpui-form-component` for GPUI glue). | R11, R14 |
| D3 | Koruma remains the only validation engine. Per-field error access (`error_for`, `field_errors`, `validate_fields`) is generated per-form bridge code over koruma's typed error structs. Error maps keyed by `String` are rejected. | R7, R8 |
| D4 | Touched tracking v1 covers the input family (`InputEvent::Blur` verified in gpui-base); other component kinds mark touched on value change until their event surfaces are verified. | R1, R2 |
| D5 | The derive generates state + metadata only; event wiring (subscriptions, blur hooks, change handlers) stays app-authored (or prototyping-scaffold-authored). No generated `cx.subscribe_in` from the derive. | R5, R6 |
| D6 | The validation cache is fresh iff `current == validated snapshot` (snapshot-compare), making it sound even when callers mutate through `current_mut()` escapes. | R4 |
| D7 | Submit lifecycle is a headless `SubmitTracker` with a generation/epoch guard against stale async completions, surfaced per form through an opt-in `#[gpui_form(submit)]` type alias. | R12 |
| D8 | Gate for M3: before any nested/repeated-field work, decide between `enum PathSegment { Name(&'static str), Index(usize) }` (0.6 breaking change of the core primitive) and a separate typed list-item path type. String formatting (`format!("tags.{}", i)`) is rejected. | R9 |
| D9 | One minor bump per milestone on the 0.x line (M1 → 0.6.0 … M5 → 0.10.0). New facade cargo features only when a new optional dependency appears. | R13, R15 |
| D10 | Partial holder (#10) is gated: ship patch (#9) first; build `partial_holder` only if the patch tri-state cannot express "cleared vs unchanged". | Rejection 10 |

---

## 1. Feature Parity Matrix (react-hook-form / TanStack Form vs gpui-form)

This matrix judges `gpui-form` the way a react-hook-form (RHF) v7.x / TanStack
Form v1 practitioner would: every state, validation, submission,
field-collection, and dev-ergonomics feature from those libraries is classified
into exactly one of five buckets. Perspective on the fundamental shape
difference: RHF/TanStack keep a runtime form instance whose state is observed
reactively; `gpui-form` generates **plain data holders**
(`<Name>FormValueHolder`), **GPUI entities** (`<Name>FormFields`/
`<Name>FormComponents`), **typed paths** (`<Name>FormPath`), and **derive-time
validation wiring** (koruma). Reactivity is replaced by GPUI entity observation
+ explicit mutation, so many "features" translate rather than port. Library
claims are cited to verified doc pages; repo claims cite `file:line`.

### SHIPPED (already in gpui-form)

| Feature | RHF/TanStack API | gpui-form status today | Evidence |
|---|---|---|---|
| Typed form values model | RHF `register()`-collected values; TanStack `FormState.values` | Generated `<Name>FormValueHolder` struct with public typed fields; conversions `From<Original> for Holder` and `From<Holder> for Original`/`try_from` | `crates/gpui-form-derive/src/derives/gpui_form/value_holder.rs:683-704` (holder + From), `:654-674` (reverse conversions), `crates/gpui-form-core/src/state.rs:61-64` |
| Default values | RHF `defaultValues`; TanStack `defaultValues` option | `default = <expr>` seeds holder `Default` impl (custom defaults suppress blanket `Default` derive) and initial select choices; bon builder for prefill when fields are skipped | `value_holder.rs:491-497`, `:498-500`, `:143-205`; README "Common field-level helpers" (`README.md:116-124`) |
| Whole-form dirty tracking | RHF `formState.isDirty`; TanStack `state.isDirty` | `FormState<H>::is_dirty()` — whole-value `current != baseline` | `crates/gpui-form-core/src/state.rs:130-132`; re-exported `gpui_form::FormState` (`crates/gpui-form/src/lib.rs:15`) |
| Reset form | RHF `reset()`; TanStack `reset()` | `FormState::reset_to_baseline()` restores `current` from `baseline` | `state.rs:112-114` |
| Mark clean after save | RHF `reset(undefined, { keepValues })`-ish; TanStack re-sync | `FormState::sync_baseline()` snapshots current into baseline | `state.rs:119-121` |
| Reset with new values (non-reactive) | RHF `values` option / TanStack `reset(values)` | `FormState::replace_current(holder)` overwrites live value, baseline untouched (equal-to-baseline clears dirty) | `state.rs:100-102`; README save/restore section (`README.md:466-533`) |
| Values persistence (save/restore) | Not first-class in either lib (getValues → your storage) | Opt-in `serde` feature adds `Serialize`/`Deserialize` to the holder; round-trips any serde format | `value_holder.rs:516-520`; facade feature `crates/gpui-form/Cargo.toml:20`; `docs/form-state-persistence.md` |
| Typed field identifiers (flat) | RHF string names; TanStack `DeepKeys<T>` type-level paths | `FieldPath` primitive + generated `<Name>FormPath` newtype, one constructor per non-skipped field; `Deref`/`AsRef` to the primitive | `crates/gpui-form-core/src/path.rs:74-156`; generator `crates/gpui-form-derive/src/derives/gpui_form/field_path.rs:44-83`; facade `lib.rs:14,17` |
| Per-field validation rules | RHF `register` rules (`required/min/max/pattern/validate`); TanStack field `validators` | Field-level koruma attrs (`#[koruma(...)]`) mirrored onto the generated holder: required injection for Option-wrapped fields, builder-chain validators, newtype/nested decomposition | `value_holder.rs:431-489` (mirror loop, required at `:447-451`, newtype/nested at `:471-477`); README "Validation With Koruma" (`README.md:375-417`) |
| Collect ALL errors per field | RHF `criteriaMode: "all"` | Koruma error enums expose `.all()` issue lists per field — all-mode is the only (and default) behavior | `examples/forms-demo/src/main.rs:371-406` (`e.username().all()`, `e.age().all()`) |
| Built-in validators: required, email, length, numeric range, non-empty | RHF built-in rules + resolver libs (zod/yup); TanStack validator fns | Via koruma + koruma-collection validators mirrored at derive time | `value_holder.rs:437-460`; demo usage `main.rs:92-101` |
| Phone-number validation (parser-backed) | Neither lib ships this | `component(phone_input)` + headless `gpui_form::phone` helpers (`validate_phone_number`, `..._for_country_label`, `PhoneCountry` trait), behind the `phone` feature; sibling-country binding recorded as metadata | README `README.md:132-248`; facade feature `Cargo.toml:19`; `crates/gpui-form-schema/src/components.rs:99` (`PhoneInputBehaviour`); `crates/gpui-form-core/src/phone.rs` |
| Localized error messages | Manual in both libs (resolve messages yourself) | `koruma(fluent)` emits `<FORM>_I18N_KEY_PREFIX`, per-field `<FIELD>_LABEL_KEY`, `validation_issue_key(kind)`, `localized_validation_issue(kind)` through rust-i18n | `value_holder.rs:722-779`; README `README.md:409-417`; locale-regression tests `main.rs:689-758` |
| Controlled-component integration escape hatch | RHF `Controller`/`useController`; TanStack `useField` | Generated component constructors per field (`SignupFormComponents::username_input(...)`) + custom components with optional `CustomComponentValueAdapter` value binding | `main.rs:140-147`; README "Custom Components" (`README.md:587-637`); `crates/gpui-form-component/src/custom.rs` |
| Programmatic get/set values | RHF `getValues`/`setValue`; TanStack `getFieldValue`/`setFieldValue` | Holder fields are plain `pub` fields — direct typed access/mutation; `FormState::current()/current_mut()` for the stateful case | `value_holder.rs:538-552` (pub field defs); `state.rs:81-95` |
| Numeric input validation discipline | RHF `valueAsNumber`/rules; TanStack validate fns | Character-level checks + `FromStr` parse against validation type (`number_input(as = ...)`); explicitly not regex-driven | `docs/feature-backlog.md:339-366` (#17, invariant statement); `crates/gpui-form-core/src/numeric.rs` |
| Prototyping/introspection registry | (Beyond both libs — closest is TanStack devtools/formOptions) | `inventory` feature registers `GpuiFormShape` (struct name, per-field `FieldVariant` with behavior/validations/layout metadata) consumed by the prototyping codegen | `crates/gpui-form-schema/src/registry.rs:4-63`, `:66-130` (`FieldVariant`); facade `Cargo.toml:18` |

### PARTIAL (exists but weaker)

| Feature | RHF/TanStack API | gpui-form status today | Evidence |
|---|---|---|---|
| Per-field typed error access | RHF `formState.errors.<name>` / `getFieldState(name)`; TanStack `field.state.meta.errors` | Koruma generates per-field error enums (`err.username().all()`), but access is whole-form-first (`validate().err()`), the accessor set comes from koruma not from `FieldPath`, and apps hand-match validator variants to i18n keys. Missing: path-keyed `error_for(path)`, `field_errors()` map, `validate_fields()` — exactly backlog #6 | Demo `main.rs:371-406` (hand-written variant matches); backlog `docs/feature-backlog.md:154-170` (#6) |
| Manual validation trigger | RHF `trigger(name?)` (one/some/all); TanStack `form.validate()`/`validateField()` | `holder.validate()` validates the whole holder only; no single-field or subset trigger | `main.rs:371,578` (whole-form calls only); backlog #6 proposes `validate_fields()` |
| Typed dynamic paths (nested/array names) | TanStack `DeepKeys<T>`, `DeepValue<T>` type-level paths; RHF nested `name="address.city"` | `FieldPath` supports hand-built multi-segment paths (`new(&["a","b"])`) but FLAT v1 emits single-field constructors only; typed nested/list composition deferred to #2/#3 | `path.rs:12-18` (scope note); `field_path.rs:8-13`; README `README.md:572-582` |
| Section/grouped forms | TanStack `FormGroupApi`/`FormGroupMeta` incl. per-group submission state | `section/label/description/placeholder/width` hints are METADATA-FIRST v1 on `FieldLayout` — order-preserving grouping for generators, no per-group validation/submission state, no rendering | `crates/gpui-form-schema/src/layout.rs`; `crates/gpui-form-schema/src/lib.rs:5`; README `README.md:266-316`; backlog #4 status `docs/feature-backlog.md:101-124` |
| Cross-field validation | RHF validate fn reading `getValues()`; TanStack form-level validators + field listeners | Only narrow support: `phone_input(country = <field>)` records the sibling field as metadata (value-layer enforcement "remains open"), plus koruma newtype/nested type composition; no general cross-field validator hook | README `README.md:167-172`; backlog #18 status `docs/feature-backlog.md:365-416`; dependencies #12/#13 `docs/feature-backlog.md:267-295` |
| External schema resolver (pluggable validation engine) | RHF `resolver` (zod/yup/Joi/Ajv/Vest); TanStack Standard Schema (`standardSchemaValidators`) | Koruma IS the schema layer, wired at derive time — you cannot plug a different validation engine at runtime. In Rust terms this is a deliberate translation (koruma ≈ the zod ecosystem), but the pluggability is absent | `value_holder.rs:431-489` (koruma-only path); uncertainty: whether koruma-core supports struct-level validators the derive could mirror was not audited — flagged |
| Watch / value subscription | RHF `watch`/`useWatch`/`subscribe`; TanStack field/form `listeners` + `form.Subscribe` | Solved by GPUI entity observation: `cx.subscribe_in(&input, ...)` per component, hand-mirrored into the holder (10 explicit subscriptions in the demo). No generated sync glue, no selector/debounce/computed watchers, no change listeners on the plain-data holder | `main.rs:151-162` (subscription list), `:184-299` (hand-written handlers) |
| Single-field reset to defaults | RHF `resetField(name)`; TanStack `resetField(field)` | Only whole-holder reset; `default_expr` is parse-time only and not retained on the runtime holder, so per-field defaults are unavailable at runtime | `docs/form-state-persistence.md:79-96` ("the baseline obstacle"); `state.rs:112-121` (no per-field API) |
| Async default values / `isLoading` | RHF async `defaultValues` + `formState.isLoading`; TanStack async loading patterns | `replace_current()` covers "data arrived after form construction", but there is no loading-state concept | `state.rs:100-102` |

### SOLVED-DIFFERENTLY (the translation, not the port)

| Feature | RHF/TanStack API | gpui-form status today | Evidence |
|---|---|---|---|
| Runtime validation engine vs derive-time wiring | RHF runtime rules/resolver; TanStack runtime validator fns per cause | Validation rules are declared as attributes and **composed into the holder at derive time**; execution is still runtime (`validate()` on data). Translation: the "schema object" becomes the generated holder's koruma impl; validator selection happens at compile time | `value_holder.rs:431-514` (derive list + koruma attrs); demo execution `main.rs:371` |
| Reactive state observation vs entity observation | RHF Proxy-based `formState` re-renders; TanStack store + `Subscribe` | GPUI entities (`Entity<InputState>` etc.) emit typed events; the app subscribes and mutates the holder, then `cx.notify()`. No fine-grained proxy tracking — coarser, explicit, and compile-checked instead | `main.rs:151-162`, `:190-199` |
| Headless library + renderer | RHF/TanStack headless; you render inputs | The derive generates both the data holder AND per-field gpui-kit entity constructors + component constructors (`<Name>FormComponents::<field>_<kind>()`) | `main.rs:140-147` (`SignupFormComponents::username_input`, `::country_select`, `::birth_date_date_picker`); `crates/gpui-form-derive/src/derives/gpui_form/components.rs:9` |
| Type-safety of paths/values | TanStack `DeepKeys` type gymnastics; RHF string names (weak) | Stronger than both in the direct-access dimension: holder fields are real struct fields (nonexistent field = compile error), FLAT `FormPath` constructors are generated per field; weaker for dynamic/path-driven iteration | `field_path.rs:55-65`; `value_holder.rs:538-552` |
| Reusable form presets | TanStack `formOptions` | The derive itself is the preset factory — one macro produces holder, fields, components, paths, and (opt-in) inventory shape | `crates/gpui-form-derive/src/derives/gpui_form/expansion.rs:48-58`, `:212-220` |

### MISSING-WANTED (in plan scope)

| Feature | RHF/TanStack API | gpui-form status today | Evidence |
|---|---|---|---|
| Field arrays (`Vec<T>`) with typed add/remove/reorder | RHF `useFieldArray` (`append/prepend/insert/swap/move/update/replace/remove`, keyed `fields`); TanStack `pushValue/insertValue/replaceValue/removeValue/swapValues/moveValue/clearValues` | Absent. Only escape hatch: `component(custom(state = TagsInputState))` on `Vec<String>` (app-owned state). Backlog #3 scopes a conservative v1 (Vec only, add/remove first) | Backlog `docs/feature-backlog.md:75-98` (#3); README custom example `README.md:596-605` |
| Nested forms / nested object fields | RHF nested names + hierarchical resolver errors; TanStack `FieldGroupApi`/`DeepKeys` | Absent. Backlog #2 (flatten vs nested paths, nested error paths, section metadata) | Backlog `docs/feature-backlog.md:49-73` (#2) |
| Submit lifecycle state | RHF `handleSubmit(onValid, onInvalid)`, `isSubmitting/isSubmitted/isSubmitSuccessful/submitCount`; TanStack `submissionAttempts`, `isSubmitting`, `onSubmitInvalid` | Absent — apps compose `validate()` → `try_from()` themselves. Backlog #7 sketches `is_submitting/last_error/submit(async)` but does NOT mention `is_submit_successful`, `submission_attempts` (submit count), or `can_submit` | Backlog `docs/feature-backlog.md:170-183` (#7); conversions exist at `value_holder.rs:654-674` |
| Field-level diff / per-field dirty / patch | RHF `formState.dirtyFields`; TanStack `field.state.meta.isDirty`, per-field meta map | Absent (whole-value dirty only). Backlog #9 covers it, building on `FieldPath` | `state.rs:124-131` (explicit scope note); backlog #9 `docs/feature-backlog.md:210-223` |
| Error-state helpers incl. server errors | RHF `setError`/`clearErrors`, reactive `errors` option; TanStack `setErrorMap` | Absent — validation output is recomputed from data on every `validate()`; there is nowhere to inject a server error into an otherwise-valid form | `value_holder.rs:501-505` (koruma-derived `validate` only); **not tracked by any backlog item** |
| Focus management | RHF `shouldFocusError` (focus first errored field), `setFocus(name)`; array focus options | Absent. `FieldPath` docs name focus as a future consumer; schema has a `focusable()` behavior flag, but no generated focus orchestration exists | `path.rs:5-8`; `crates/gpui-form-schema/src/components.rs:163`; **no backlog item** |
| Validation modes / live `isValid` | RHF `mode` (`onChange/onBlur/onSubmit/onTouched/all`), `reValidateMode`, live `isValid`/`isValidating`/`validatingFields`; TanStack `onMount/onChange/onBlur/onSubmit/onDynamic` + `revalidateLogic` | Absent — validation is purely on-demand; nothing recomputes `isValid` as the user types. Backlog #6 is error *access* only; trigger semantics are untracked | `main.rs:577-583` (validate only on button click); backlog #6 `docs/feature-backlog.md:154-170` |
| Touched-field tracking | RHF `formState.touchedFields`, `getFieldState().isTouched`; TanStack `field.state.meta.isTouched/isBlurred`, form-level `isTouched` | Absent — no blur/touched model anywhere in holder or state; required for onBlur validation modes | No API in `state.rs` or `value_holder.rs`; **no backlog item** (backlog #10 only uses the phrase as a use case, `docs/feature-backlog.md:244`) |
| Async validation | RHF async validate rules + async resolvers; TanStack per-cause async validators (`TOnChangeAsync` etc.), `isValidating` | Absent — koruma `validate()` is synchronous. Backlog #5 is async *select providers*, not validation | Backlog `docs/feature-backlog.md:134-153` (#5); demo sync `validate()` `main.rs:371` |
| Partial holders / PATCH payload | (Neither lib first-class; RHF `dirtyFields` pruning is the folk pattern) | Absent. Backlog #10 (optional-field holder, tri-state unchanged/clear/set) | Backlog `docs/feature-backlog.md:225-246` (#10) |
| Field dependency metadata / conditional requiredness | RHF validate fns reading other fields; TanStack listeners + form-level validators | Absent. Backlog #12 (`depends_on`) and #13 (`required_if`) | Backlog `docs/feature-backlog.md:267-295` (#12, #13) |
| Schema export | (Adjacent: zod schema reuse, JSON Schema tooling) | Absent. Backlog #14 (`UserProfileFormSchema::schema()`) — foundations (`GpuiFormShape`, `FieldVariant`) already exist | Backlog `docs/feature-backlog.md:297-307` (#14); `crates/gpui-form-schema/src/registry.rs:66-130` |
| Wizard / step metadata | (Neither lib native; multi-step is manual in RHF/TanStack) | Absent. Backlog #11 | Backlog `docs/feature-backlog.md:248-266` (#11) |
| Serde attribute passthrough | RHF/TanStack: n/a (values are JS objects) | Absent — per-field `serde(rename/skip)` on the holder. Backlog #15 | Backlog `docs/feature-backlog.md:309-321` (#15); `docs/form-state-persistence.md:129-131` |
| Devtools / debug introspection | RHF Devtools package; TanStack Form Devtools | Absent — no in-app debug render of holder + validation/dirty/touched state. A GPUI-native debug panel would be the translation | **No backlog item** |
| Remaining text-domain validation (email/URL/slug/UUID as first-class components) | RHF pattern rules; resolver lib schemas | Email exists only via koruma `EmailValidation` attr; phone shipped as a component, other domains open. Backlog #18 (status: phone shipped, email/URL/slug/UUID + cross-field country enforcement open) | Backlog `docs/feature-backlog.md:365-416` (#18); README `README.md:132-137` |

### MISSING-REJECT (deliberately out of scope; justified)

| Feature | RHF/TanStack API | Why rejected in gpui-form | Evidence |
|---|---|---|---|
| `shouldUnregister` / `unregister` / mount-lifecycle registration | RHF `shouldUnregister`, `unregister(name)`; TanStack `field.mount()` cleanup | React DOM mount/unmount concept. GPUI entities live as long as the owning view holds them; the generated field set is fixed at compile time by the derive. Dynamic removal = arrays (#3), not registration | Derive emission is static: `expansion.rs:48-58,212-220` |
| Dynamic schema fields at runtime (add whole new fields) | RHF dynamic `register` of unknown names | Derive-time field set is the core value proposition (compile-time type safety priority #1 in AGENTS.md). Collections are the sanctioned dynamic surface (#3) | `docs/feature-backlog.md:7-9` (scope boundary) |
| Native HTML validation bridge (`shouldUseNativeValidation`, `progressive`, `:valid/:invalid`) | RHF Constraint API options | HTML/DOM-only concepts; no analog in GPUI | README `README.md:515-519` scope exclusions analog |
| `isLoading` async-defaults pipeline | RHF async `defaultValues` + `isLoading` | Rust apps load data before constructing the holder; `FormState::replace_current` covers late arrival. A loading state machine belongs to the app, not the form lib | `state.rs:100-102` |
| Undo/redo stack | (Formik-adjacent; not core RHF/TanStack) | Explicitly out of scope in the shipped persistence feature | `docs/form-state-persistence.md:122-124`; `state.rs:14-16` |
| Component runtime UI state persistence (open menus, scroll) | n/a | Explicitly out of scope; only holder data persists (`InfiniteSelectState::snapshot()` covers its own needs) | README `README.md:516-519`; `docs/form-state-persistence.md:117-124` |
| SSR/server-action merge (`mergeForm`, TanStack `onServer`) | TanStack server/hydration APIs | No SSR in a desktop GPUI app; partial-holder PATCH payloads (#10) are the honest analog | Backlog #10 `docs/feature-backlog.md:225-246` |
| Form-level `disabled` propagation | RHF `disabled` option (v7.48) | gpui-kit widgets own enabled state; form-wide enable/disable is trivially app-level. Low value for the derive. (A per-field `disabled = <expr>` metadata hint might be worth one line under #4 later — flagged, not recommended now) | No disabled feature on the facade surface; no backlog item |

### Gaps the backlog does NOT track

1. **Touched/blurred-field tracking** — the single biggest untracked gap. It is
   a prerequisite for `onBlur`/`onTouched` validation modes and for the "edit
   screens that only send touched fields" use case cited inside backlog #10
   (`docs/feature-backlog.md:244`), yet no item defines the state model.
2. **Validation trigger modes + live `isValid`** — backlog #6 is path-keyed
   error *access*; nothing covers *when* validation runs (onChange/onBlur/
   onSubmit) or a derived live validity signal.
3. **Manual error injection (`setError`/`clearErrors`/server-error merge)** —
   with pure data-recomputed validation there is no place to put a server
   error. Needed for realistic submit flows.
4. **Focus management** (`set_focus(path)`, focus-first-errored-field) —
   `FieldPath`'s own docs name focus management as a future consumer
   (`path.rs:5-8`) and the schema already carries `focusable()`
   (`components.rs:163`), but no backlog item owns it.
5. **Form-level (cross-field) validation rules** — "end date after start
   date" invariants have no story: koruma mirroring is per-field
   (`value_holder.rs:431-489`), and #12/#13 cover dependency *metadata*, not
   arbitrary cross-field validators. Unverified: whether koruma supports
   struct-level rules the derive could mirror.
6. **Async validation** — backlog #5 is async select *providers*; async
   *validators* (with `is_validating`/`validating_fields`) appear nowhere.
7. **Submit-result completeness** — #7 sketches `is_submitting`/`last_error`;
   `is_submit_successful`, `submission_attempts`, and `can_submit` are absent
   from the sketch.
8. **Devtools/debug introspection** — a GPUI debug panel rendering holder +
   validation/dirty/touched state is cheap after #6/#9 and untracked.
9. **Per-field reset to default (`resetField`)** — blocked by `default_expr`
   being parse-time only (`docs/form-state-persistence.md:79-96`); worth one
   backlog line (possibly folded into #9's state model).
10. **`delayError`-style debounced error display** — minor; natural sibling of
    the trigger-modes gap. Not urgent.

### Recommended plan scope (ordered)

1. **Path-keyed error access + per-field trigger** (backlog #6):
   `validate_fields()`, `field_errors()`, `error_for(<Name>FormPath::field())`
   — the foundation every later feature names errors by.
2. **Touched tracking + validation trigger modes + live validity** (untracked
   gap; extends #6): a runtime wrapper recording touched on entity
   blur/change events, exposing `is_valid` and mode-style triggers (on-submit
   default, onChange/onBlur opt-in).
3. **Field-level diff/dirty + patch** (backlog #9): per-field dirty flags
   keyed by `FieldPath` and typed patch output.
4. **Submit lifecycle** (backlog #7, expanded): `submit(validate → try_from →
   closure)`, `is_submitting`, `is_submitted`, `is_submit_successful`,
   `submission_attempts`, `last_error`, `can_submit` — plus manual
   `set_error`/`clear_errors` (untracked gap 3).
5. **Field arrays `Vec<T>`** (backlog #3): typed add/remove/(reorder-later) on
   the holder, list-item `FieldPath` composition, error paths into indices.
6. **Nested forms** (backlog #2): nested component, nested path composition,
   nested error propagation.
7. **Partial holders** (backlog #10) — PATCH payloads, now expressible via
   dirty fields + touched [gated by D10].
8. **Focus management**: `set_focus(path)` + focus-first-error on failed
   submit (gap 4).
9. **Field dependency metadata + conditional requiredness** (backlog
   #12/#13), folding in general cross-field validation rules (gap 5).
10. **Async validation** (gap 6) — design-gated on GPUI's executor; deliver
    `async validate` + `is_validating`/`validating_fields` hooks.
11. **Schema export** (backlog #14) and **wizard/step metadata** (backlog
    #11) — metadata consumers, cheap once the state model settles.
12. **Devtools/debug panel** (gap 8) and remaining text-domain validation
    (backlog #18's email/URL/slug/UUID) as opportunistic follow-ups.

Deliberately excluded per the MISSING-REJECT table: registration-lifecycle
concepts, native-HTML bridges, SSR merge, undo/redo, UI-state persistence.

---

## 2. Headless State & API Design

**Verified foundations this design builds on:** `rust-version = "1.95"`
(workspace `Cargo.toml` line 25, edition 2024) — async fn in traits (stable
since 1.75) is freely available, no `async_trait` dep needed.
`gpui-form-core` has zero dependencies by default (only optional
`phonenumber`), so any new core type must stay alloc+std only. `FormState<H>`
(`crates/gpui-form-core/src/state.rs`) keeps private `baseline`/`current`,
`H: Clone` for construction/resets, `H: PartialEq` for dirty checks; its
module docs already defer #9 to future work. Generated holders always derive
`Clone, Debug, PartialEq` and conditionally `Default`, `bon::Builder` (skipped
fields), `Koruma`/`KorumaAllDisplay` (koruma-gated), `Serialize`/`Deserialize`
(derive-crate `serde` feature). Koruma 0.9 generates per `#[derive(Koruma)]`:
a top-level `<Holder>KorumaValidationError` with one getter per non-`koruma(skip)`
field (even zero-validator fields), per-field error structs with typed
validator getters + `all() -> Vec<<Holder><Field>KorumaValidator>` (typed
enum, `Inner` variant for newtypes) + `element_errors: Vec<(usize, _)>` for
`Vec` fields, and `ValidateExt { type Error: ValidationError; fn validate(&self)
-> Result<(), Self::Error> }` where `ValidationError` is only
`is_empty()/has_errors()`. Koruma error types derive `Clone, Debug, Default`
only — **no `PartialEq`, no `serde`**.

**Core layering rule applied throughout:** `gpui-form-core` gets *generic,
plain-data primitives* that know nothing about individual forms; the derive
emits the *typed per-form glue* that maps `<Name>FormPath` to those
primitives. Holders stay plain data everywhere ([D2]).

### 2.1 Touched-field tracking

**Crate placement:** `TouchedFields` in `gpui-form-core` (new `src/fields.rs`);
per-form index mapping in
`crates/gpui-form-derive/src/derives/gpui_form/field_path.rs`; path-keyed
methods on the generated facade (§ "Combined facade"). Nothing in
`gpui-form-component` needed. [D4] scopes v1 blur wiring to the input family.

```rust
// crates/gpui-form-core/src/fields.rs — pure, GPUI-free, serde-free, alloc-only
/// Dense bitset of touched/visited fields, keyed by *declaration-order index*.
/// Core is form-agnostic; the derive supplies the path→index mapping (2.1b).
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct TouchedFields {
    bits: Box<[u64]>,
    field_count: usize,
}

impl TouchedFields {
    /// Sized from `<Name>FormPath::FIELD_COUNT` at the call site.
    pub fn new(field_count: usize) -> Self;
    pub fn field_count(&self) -> usize;
    /// Unknown indices are a no-op (guards hand-built paths).
    pub fn touch_index(&mut self, index: usize);
    pub fn is_touched_index(&self, index: usize) -> bool;
    pub fn clear_index(&mut self, index: usize);
    pub fn clear_all(&mut self);
    /// Iterates in ascending index order = declaration order (deterministic for UI).
    pub fn touched_indices(&self) -> impl Iterator<Item = usize> + '_;
    pub fn touched_count(&self) -> usize;
    pub fn is_any_touched(&self) -> bool;
}
```

**2.1b Generated path→index mapping** (extends `generate_field_path` in
`field_path.rs`, unconditional — mirrors FLAT v1's no-generics rule):

```rust
// generated next to the existing <Name>FormPath impl
impl SignupFormPath {
    /// All non-skipped field idents in declaration order (`stringify!` idents,
    /// same source as the existing per-field constructors).
    pub const FIELD_NAMES: &'static [&'static str] = &["username", "email", "age", "code"];
    pub const FIELD_COUNT: usize = 4;

    /// Every field path in declaration order.
    pub fn all_paths() -> impl Iterator<Item = Self>;
    /// Dense index for bitset-keyed state. `Some` only for FLAT v1
    /// single-segment paths matching FIELD_NAMES; `None` for hand-built
    /// multi-segment paths. Linear scan (fine at form scale; see OQ 6).
    pub fn field_index(&self) -> Option<usize>;
    /// Inverse mapping, for generic glue iterating raw indices.
    pub fn path_for_index(index: usize) -> Option<Self>;
}
```

**Interaction with the baseline model:** touched is *independent* of
`baseline`/`current` — it records user engagement, not value deltas. Decided
reset semantics (emitted on the facade):

- `reset_to_baseline()` → values revert, **touched cleared**, validation cache
  invalidated (a fresh canvas; error display should be suppressed after a full
  reset).
- `reset_to_baseline_keeping_touched()` → values revert, touched survives
  (undo-style revert; offering both resolves the "does reset clear touched?"
  debate).
- `sync_baseline()` → **touched untouched** (a successful save does not
  un-visit fields) and validation cache stays fresh (`current` did not
  change).
- `replace_current()` (programmatic load) → touched untouched, cache
  invalidated.

**Serde:** none. Touched state is transient; persistence stays holder-only per
shipped feature #1 ([D2]).

### 2.2 Per-field dirty / changed-fields / patch (backlog #9)

**Crate placement:** the bitset machinery stays private inside
`TouchedFields` (changed-state is computed, not stored). `<Name>FormPatch` is
generated by a new
`crates/gpui-form-derive/src/derives/gpui_form/form_state.rs` generator,
wired in `expansion.rs` next to `value_holder_tokens`/`field_path_tokens`.

Per-field comparison works with zero new bounds: the holder already derives
`PartialEq`, which requires every holder field type to be `PartialEq`, so
generated per-field comparisons (`from.username != to.username`) always
compile — including fields with `override_type`/`from`/`into` conversions,
because comparison happens on *holder* types, not source types.

```rust
// generated per form (unconditional; serde derives behind the derive crate's
// `serde` feature, same gate as the holder)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SignupFormPatch {
    // One field per non-skipped field: Option<<holder field type>>.
    // outer Some = changed; inner value is the holder-typed value, so
    // wrapped/originally-optional fields read as Option<Option<T>>
    // (Some(None) = "changed to cleared"). Skipped fields are absent.
    pub username: Option<Option<String>>,
    pub email: Option<Option<String>>,
    pub age: Option<Option<u32>>,
    pub code: Option<Option<InviteCode>>,
}

impl SignupFormPatch {
    /// Diff two holders field-by-field. Declaration order.
    pub fn between(from: &SignupFormValueHolder, to: &SignupFormValueHolder) -> Self;
    pub fn is_empty(&self) -> bool;
    pub fn changed_fields(&self) -> Vec<SignupFormPath>;
    /// Apply changed fields onto a holder; `None` fields are left untouched.
    pub fn apply_to(self, holder: &mut SignupFormValueHolder);
}
// #[cfg(feature = "serde")] on the derive crate adds:
//   #[derive(::serde::Serialize, ::serde::Deserialize)] with
//   #[serde(skip_serializing_if = "Option::is_none", default)] per field, so the
//   wire format is the PATCH tri-state: absent = unchanged, null = cleared, value = set.
```

Facade methods (everything compares `current` against `baseline`, so
`patch()` is empty right after `sync_baseline()` and full after edits, exactly
matching `is_dirty()`):

```rust
impl SignupFormState {                       // generated facade, § below
    pub fn is_dirty(&self) -> bool;                                  // delegates to FormState
    pub fn is_field_dirty(&self, field: &SignupFormPath) -> bool;    // false + debug_assert! for unknown paths
    pub fn changed_fields(&self) -> Vec<SignupFormPath>;             // baseline -> current, declaration order
    pub fn patch(&self) -> SignupFormPatch;                          // == SignupFormPatch::between(baseline, current)
}
```

**Boundary with backlog #10 (`partial_holder`):** `<Name>FormPatch` is a
*runtime diff value* derived from two holders; #10's partial holder is a
*separate compile-time type* generated from the source struct. Deliberately
kept distinct [D10]; `<Name>FormPatch` does not satisfy #10 and vice versa.
#13 conditional-requiredness has no effect here (dirty is value-equality,
rule-agnostic).

### 2.3 Per-field typed error access (backlog #6)

**Crate placement:** the trait `FormFieldErrors` in `gpui-form-core` (new
`src/validation.rs`); the error map, wrapper enum, and `validate_fields()`
generated in `value_holder.rs`'s koruma branch (only when
`needs_koruma_derive` is true — the same gate that puts `#[derive(Koruma)]` on
the holder, so the koruma error types referenced below are guaranteed to
exist; see also Pipeline hazard 6).

The constraint respected: `koruma::ValidationError` is a trait with only
`is_empty`/`has_errors`, and each field's concrete error type is distinct — so
a single path-keyed lookup cannot return `&ConcreteType` without erasure
([D3]). The design: **typed named accessors as the primary API** (zero
erasure, mirrors koruma's own style), plus a **path-addressable borrowing
enum** for generic/loop code (render all failing fields, attach errors to
path-keyed UI state).

```rust
// crates/gpui-form-core/src/validation.rs
/// Implemented by the generated per-form error map so GPUI glue and tests can
/// be generic over any form's errors using core's FieldPath.
pub trait FormFieldErrors {
    fn is_empty(&self) -> bool;
    /// Failing field paths in declaration order.
    fn failing_paths(&self) -> Vec<FieldPath>;
}
```

```rust
// generated per form, koruma-gated (same gate as #[derive(Koruma)])
#[derive(Clone, Debug, Default)]
pub struct SignupFormFieldErrors {
    // one private slot per non-skipped field; koruma(top-level error struct)
    // getters already exist per field, this re-keys them by typed path
    username: Option<SignupFormValueHolderUsernameKorumaValidationError>,
    email: Option<SignupFormValueHolderEmailKorumaValidationError>,
    age: Option<SignupFormValueHolderAgeKorumaValidationError>,
    code: Option<SignupFormValueHolderCodeKorumaValidationError>,
}

impl SignupFormFieldErrors {
    pub fn is_empty(&self) -> bool;
    pub fn has_errors(&self) -> bool;
    /// Fully typed accessors — the primary API (declaration order):
    pub fn username(&self) -> Option<&SignupFormValueHolderUsernameKorumaValidationError>;
    pub fn email(&self) -> Option<&SignupFormValueHolderEmailKorumaValidationError>;
    pub fn age(&self) -> Option<&SignupFormValueHolderAgeKorumaValidationError>;
    pub fn code(&self) -> Option<&SignupFormValueHolderCodeKorumaValidationError>;
    /// Path-addressable: Some only when `field` names a failing field.
    /// Unknown paths -> None.
    pub fn error_for<'a>(&'a self, field: &SignupFormPath) -> Option<SignupFormFieldError<'a>>;
    /// (path, error) pairs in declaration order, failing fields only.
    pub fn failing_fields(&self) -> impl Iterator<Item = (SignupFormPath, SignupFormFieldError<'_>)> + '_;
}

impl ::gpui_form::core::FormFieldErrors for SignupFormFieldErrors { /* is_empty, failing_paths */ }

/// One variant per field, borrowing that field's koruma error struct.
#[derive(Clone, Debug)]
pub enum SignupFormFieldError<'a> {
    Username(&'a SignupFormValueHolderUsernameKorumaValidationError),
    Email(&'a SignupFormValueHolderEmailKorumaValidationError),
    Age(&'a SignupFormValueHolderAgeKorumaValidationError),
    Code(&'a SignupFormValueHolderCodeKorumaValidationError),
}

impl<'a> SignupFormFieldError<'a> {
    pub fn path(&self) -> SignupFormPath;
    /// Typed passthrough to koruma's per-field `all()`:
    pub fn all(&self) -> Vec<SignupFormValueHolder{Field}KorumaValidator>; // per-variant
    /// i18n key per failed validator (e.g. "validation.required"), replacing the
    /// match arms forms-demo hand-writes today. Emitted under the SAME gate as the
    /// existing `validation_issue_key` helper (`koruma(fluent)`); see OQ 1.
    pub fn issue_keys(&self) -> Vec<&'static str>;
}

// generated on the holder, koruma-gated — pure method, plain data in/out:
impl SignupFormValueHolder {
    /// Run koruma validation once and re-key the result by typed paths.
    /// (Raw koruma `validate()` stays untouched; `map.is_empty()` == `is_ok()`.)
    pub fn validate_fields(&self) -> SignupFormFieldErrors;
}
```

**Baseline/interaction notes:** the map is a pure function of `current`; it
never touches baseline. "Show errors only for touched fields" is caller-side
composition (`touched.is_touched(path) && errors.error_for(path)`) — no
coupled API. #13 conditional requiredness slots in at the koruma-rule layer;
this map is rule-agnostic, so `failing_paths()` covers
conditionally-required fields automatically once those rules exist. **Scope
note (FLAT v1):** element-validator errors (`element_errors: Vec<(usize, _)>`
on `Vec` fields) are reachable only via the typed per-field accessors; a
per-item path surface (`tags[2]`-style) is blocked on backlog #3
repeated-field paths (OQ 7). **Serde:** none — koruma error types are
`Clone/Debug`-only, so the map cannot and should not be serializable (OQ 2)
[D2].

### 2.4 Submit lifecycle (backlog #7)

**Crate placement:** `SubmitTracker<E>` in `gpui-form-core` (new
`src/submit.rs`) as a standalone generic — deliberately **not** part of the
generated facade: (a) backlog #7 says "optional runtime glue, not required
generated state", and (b) `last_error`'s type `E` is the app's submit-failure
type, which the derive cannot know. The submit *hook* is a core trait (async
fn in trait, no runtime dep — see 2.6); *driving* it is GPUI glue in a later
phase. Per-form exposure is an opt-in `#[gpui_form(submit)]` type alias
[D7]; `is_submitted`/`is_submit_successful`/`can_submit` from parity gap 7 are
part of the surface.

```rust
// crates/gpui-form-core/src/submit.rs — plain data, no executor, no serde
#[derive(Clone, Debug)]
pub struct SubmitTracker<E> {
    submitting: bool,
    count: u32,
    last_error: Option<E>,
    generation: u64,        // [D7]/R12: guards stale async completions
}

impl<E> SubmitTracker<E> {
    pub const fn new() -> Self;
    pub fn is_submitting(&self) -> bool;
    pub fn submit_count(&self) -> u32;
    /// True once at least one attempt has reached a terminal state.
    pub fn is_submitted(&self) -> bool;
    /// True when the last terminal state was success.
    pub fn is_submit_successful(&self) -> bool;
    /// Submit-button gating: not currently submitting. Callers add their own
    /// validity (facade `is_valid()`) and dirty policy on top.
    pub fn can_submit(&self) -> bool;
    pub fn last_error(&self) -> Option<&E>;
    pub fn take_last_error(&mut self) -> Option<E>;
    /// Marks an attempt started; returns false when one is already in flight
    /// (re-entry guard — double-click protection is the caller's decision,
    /// but the data to enforce it lives here). Increments `count` only on
    /// Ok(true). Bumps `generation`.
    pub fn begin(&mut self) -> bool;
    /// Completes the attempt: clears `submitting`; `Ok(())` clears
    /// `last_error`, `Err(e)` records it. Stale completions (generation
    /// mismatch after `reset()`) are ignored.
    pub fn finish(&mut self, result: Result<(), E>);
    /// Clears in-flight state with no outcome (abandoned/cancelled submit).
    pub fn abandon(&mut self);
    /// Manual error injection/clearing for server-side errors (parity gap 3).
    pub fn set_error(&mut self, error: E);
    pub fn clear_error(&mut self);
    pub fn reset(&mut self);
}

impl<E> Default for SubmitTracker<E> { /* ... */ }

/// The submit hook contract. Executor-free: the implementor returns a future;
/// *driving* it (GPUI `cx.spawn`, tokio) is glue. No `Send` bound — GPUI
/// futures are typically non-Send (OQ 3).
pub trait SubmitAction<H> {
    type Output;
    type Error;
    /// Perform the submit for a form value snapshot.
    async fn submit(&mut self, holder: &H) -> Result<Self::Output, Self::Error>;
}
```

**Holder rule honored:** no submit fields on `<Name>FormValueHolder`; `E`
never appears in generated state [D2]. Holders keep flowing through
`SubmitAction::submit` as plain `&H`. Interaction with the baseline model is
the caller's save protocol: on `Ok`, the app calls `sync_baseline()` (that is
exactly what it exists for); on `Err`, nothing changes and `last_error`
carries the failure. The GPUI driver phase (component crate) will wrap
`begin()`/`SubmitAction::submit`/`finish()` in a `cx.spawn` closure mirroring
the existing `.spawn(cx, async move |cx| ...)` convention in `file_picker.rs`;
that glue is explicitly out of this headless design's scope [D5].

### 2.5 Validation state cache for live `isValid`

**Crate placement:** generic cache in `gpui-form-core/src/validation.rs`; the
koruma-typed instantiation on the facade.

**Invalidation rule — decided [D6]:** snapshot-compare, not staleness flags.
`FormState` hands out `&mut H` via `current_mut()`, so mutation can't be
intercepted; any flag-based scheme is either wrong or requires an
explicit-invalidate discipline on every edit path. Instead the cache stores
the holder snapshot at validation time and is fresh iff `current == snapshot`
— automatically correct for *every* mutation route (`edit`, `current_mut`,
`replace_current`, `reset_to_baseline`), zero API discipline. Cost: one
holder clone per `validate()` (form-scale data) and one `PartialEq` per cache
read.

```rust
// crates/gpui-form-core/src/validation.rs
/// Outcome cache keyed by value equality. `O` is typically a generated
/// `SignupFormFieldErrors` (or plain `bool`).
#[derive(Clone, Debug)]
pub struct ValidationCache<H, O> {
    stored: Option<(H, O)>,   // (snapshot validated, outcome)
}

impl<H: PartialEq, O> ValidationCache<H, O> {
    pub const fn new() -> Self;
    /// Fresh iff `current` equals the validated snapshot. Cheap: one PartialEq.
    pub fn get(&self, current: &H) -> Option<&O>;
    pub fn store(&mut self, validated: H, outcome: O);
    pub fn invalidate(&mut self);
}
impl<H, O> Default for ValidationCache<H, O> { /* ... */ }
```

```rust
// generated on the facade, koruma-gated (the struct field
// `validation: ValidationCache<SignupFormValueHolder, SignupFormFieldErrors>` exists only then)
impl SignupFormState {
    /// Re-run validation on `current`, cache, return the outcome.
    pub fn validate(&mut self) -> &SignupFormFieldErrors;
    /// Cache read: Some only when fresh (see invalidation rule above).
    pub fn validation(&self) -> Option<&SignupFormFieldErrors>;
    /// Always accurate: cached verdict when fresh; otherwise a fresh
    /// `current().validate_fields()` computed with `&self` (koruma validate is
    /// &self) WITHOUT caching. Never lies between cache generations.
    pub fn is_valid(&self) -> bool;
    /// Manual escape for callers who bypass the facade's mutators.
    pub fn invalidate_validation(&mut self);
    /// Cached-when-fresh, validate-and-cache on miss (takes &mut to cache).
    pub fn error_for(&mut self, field: &SignupFormPath) -> Option<SignupFormFieldError<'_>>;
}
```

Facade mutators keep the cache honest automatically:
`edit`/`replace_current`/`reset_to_baseline*` invalidate it; `sync_baseline`
does **not** (validity depends only on `current`, which didn't change — the
snapshot comparison yields this for free). `state_mut()` (raw `&mut
FormState<H>` escape hatch) is documented as bypassing invalidation, with
`invalidate_validation()` as the recourse. **Serde:** none — the cache is
transient by construction [D2].

### 2.6 Async validation contract

**Where the trait lives — decided: `gpui-form-core`, new
`src/async_validation.rs`, ungated.** Rationale against the component crate:
an async fn in trait adds no dependency and no executor (verified: `async fn`
in traits is stable at MSRV 1.95, and core is alloc-only), so the *contract*
belongs with the other headless primitives and is implementable by
server-side revalidation (tokio), tests, and GPUI glue alike. Rationale
against the pure-sync reading of core: core's purity rule is "no GPUI types,
no entities, no runtime state" — an executor-free future *contract* stores no
state and pulls nothing in; the module doc boundary is preserved. What is
*excluded* from core: anything that runs, times, or cancels a future.

```rust
// crates/gpui-form-core/src/async_validation.rs
use crate::FieldPath;

/// Executor-free async per-field validation contract. No `Send` bound so GPUI
/// (non-Send) futures satisfy it; tokio users who need to `spawn` pay for a
/// Send wrapper themselves (OQ 3).
pub trait AsyncFieldValidation<H> {
    type Error;
    /// Validate one field of the live holder. Return `Ok(())` for fields this
    /// validator does not handle, so one impl can serve a whole form.
    /// Path-keyed by the shared FieldPath (FLAT v1: single segment).
    async fn validate_field(&self, holder: &H, field: &FieldPath) -> Result<(), Self::Error>;
}
```

**Ownership of cancellation/debounce — decided:** the *driving* layer owns
them, i.e. the future GPUI glue in `gpui-form-component` (which already holds
all GPUI-facing runtime state, per AGENTS.md layering) [D5]. Concretely:
debounce timers and latest-wins cancellation (drop the in-flight future when
a newer edit/keystroke arrives — natural drop-based cancellation, no token in
the trait) live in the component-crate driver; the trait itself is stateless
per call and fire-and-forget. Core exposes no clock, no timer, no
cancellation token — that would violate its purity contract and would be dead
weight for headless consumers.

**Not generated:** the derive emits nothing for async validation —
`AsyncFieldValidation` impls are app-owned (they reference app services like
"username taken?" checks), so there is no generated-per-form surface here.
Async outcomes for v1 are stored by the driving glue (a per-path overlay
owned by the GPUI handle) and are deliberately **not merged into
`SignupFormFieldErrors`** in this phase; merge semantics (replace vs. append
per field, and how async errors interact with `is_valid`) is deferred to the
glue-phase plan (OQ 5).

### Combined facade vs standalone pieces

**Recommendation: generate a combined `<Name>FormState` facade that
*composes* the standalone core pieces — composition, not a monolith** [D1].
Concretely, a new
`crates/gpui-form-derive/src/derives/gpui_form/form_state.rs` emits:

```rust
// generated per form; struct shape is koruma-conditional only in the
// `validation` field + koruma-gated methods; everything else unconditional
pub struct SignupFormState {
    state: ::gpui_form::core::FormState<SignupFormValueHolder>,
    touched: ::gpui_form::core::TouchedFields,                     // sized FIELD_COUNT
    // koruma-gated:
    validation: ::gpui_form::core::ValidationCache<SignupFormValueHolder, SignupFormFieldErrors>,
}

impl SignupFormState {
    pub fn new(holder: SignupFormValueHolder) -> Self;
    pub fn state(&self) -> &::gpui_form::core::FormState<SignupFormValueHolder>;
    /// Escape hatch to the existing FormState API; documented as bypassing
    /// validation-cache invalidation (pair with invalidate_validation()).
    pub fn state_mut(&mut self) -> &mut ::gpui_form::core::FormState<SignupFormValueHolder>;
    // ...all methods specified in 2.1–2.5...
}
```

Why combined: the facade is the only place that knows `FIELD_COUNT`, can size
`TouchedFields`, can type the path-keyed accessors, and can couple reset
semantics to the right pieces; without it every user re-wires three objects
per form. It also fixes the existing smell in
`examples/forms-demo/src/main.rs` (a throwaway `FormState::new(default)` +
`replace_current` built inside `render` just to check dirty). Why composition
still matters: every primitive stays usable standalone (`TouchedFields`,
`SubmitTracker<E>`, `ValidationCache`, `FormState<H>`) for non-derived
holders, tests, and future backends; `FormState<H>` itself is **unchanged**
(its documented "any cloneable value" contract and `Clone`/`PartialEq` bounds
stay intact — no field-count knowledge forced into it [D1]); `SubmitTracker<E>`
stays outside the facade (`E` is app-defined; backlog #7 mandates optional
glue). Core re-exports: `pub use` the new types from
`crates/gpui-form-core/src/lib.rs`; facade adds the same names to
`crates/gpui-form/src/lib.rs` beside `FieldPath`/`FormState`.

**Where this state must NOT go (explicit [D2]):**

- `<Name>FormValueHolder` — stays plain data (`Clone/Debug/Default/PartialEq`
  [+ koruma/serde derives]); the only added member is the *pure method*
  `validate_fields()` (koruma-gated, a function of the value like `validate()`
  itself). No touched/dirty/error/submit fields, ever.
- `<Name>FormPath` — naming only; gains const tables and pure index mappings,
  no mutable state.
- `gpui-form-schema` (`FieldVariant`, `GpuiFormShape`) — compile-time
  metadata only.
- `FormState<H>` — remains a two-slot generic; no touched/validation fields
  inside it.
- Prototyping inventory — metadata/scaffold emission only.

### Open design questions

1. **`issue_keys()` gating:** the kind→key mapping currently exists only
   under `koruma(fluent)` (`generate_koruma_i18n_impl`). Emit
   `SignupFormFieldError::issue_keys()` unconditionally (duplicating the
   mapping at the derive layer via the existing `validation_issue_key_for_kind`
   helper) or keep it fluent-gated? Forms-demo hand-writes those match arms
   either way, so unconditional is the better DX, but it changes an existing
   gating boundary.
2. **Koruma error-type derives:** koruma error structs derive `Clone, Debug,
   Default` only — no `PartialEq`, no `serde`. Tests can assert via
   `is_empty()`/`Debug`, and the error map is deliberately non-serializable,
   but if patch/error persistence or `assert_eq!` on errors is ever wanted,
   that is an upstream koruma change or a gpui-form-side newtype. Left out of
   scope here.
3. **`Send` bounds on async traits:** `AsyncFieldValidation`/`SubmitAction`
   are un-`Send`ed for GPUI-executor friendliness. Tokio users need `Send`
   futures to `spawn`; decide later whether to add a `Send`-bound twin trait,
   a generic bound parameter, or leave block_on-only.
4. **Patch wire format:** `skip_serializing_if = "Option::is_none"` (absent =
   unchanged, null = cleared) is proposed for the PATCH tri-state; a team
   wanting explicit "unchanged" on the wire would drop the skip attribute.
   Convention, not technical.
5. **Async outcome merge semantics:** where async field errors land relative
   to sync `SignupFormFieldErrors` (overlay map owned by the GPUI glue vs.
   merging in), and how async-pending/erroring fields affect `is_valid()` —
   deferred to the GPUI-glue phase, flagged now.
6. **`field_index()` implementation:** linear scan over `FIELD_NAMES` is fine
   at form scale; if per-frame GPUI glue calls it on hot paths, regenerate as
   a `match` on segments (codegen-only change, API stable).
7. **Element-validator errors:** `error_for`/`issue_keys` cover field-level
   validators; per-item element errors (`element_errors: Vec<(usize, _)>`)
   need an item-indexed path surface that is blocked on backlog #3 (repeated
   fields / list paths). FLAT v1 scope note; see also D8.
8. **Facade naming:** `<Name>FormState` intentionally echoes core
   `FormState<H>` (the facade *is* a form state); alternative `<Name>Form` is
   shorter but vaguer. Recorded as a naming decision for review.

---

## 3. Codegen & Derive Implementation Plan

Baseline read: the pipeline is `#[derive(GpuiForm)]` →
`crates/gpui-form-derive/src/derives/gpui_form/expansion.rs` (orchestration)
→ darling parsing in `structs.rs` + component parsing in
`crates/gpui-form-codegen/src/components.rs` + per-component `field_tokens`
in `crates/gpui-form-codegen/src/implementations/*` → emitted items:
`<Name>FormValueHolder` (`value_holder.rs`), `<Name>FormPath`
(`field_path.rs`), `<Name>FormFields` + `<Name>FormComponents` (codegen
impls), `inventory::submit! { GpuiFormShape ... }` (schema `registry.rs`).
Koruma is re-parsed independently via `koruma_derive_core::parse_field` and
re-emitted as `#[koruma(...)]` attrs on holder fields. `FormState<H>`/
`FieldPath` live headless in `crates/gpui-form-core/src/state.rs` /
`path.rs`. Koruma's generated error struct is predictably named
`<HolderName>KorumaValidationError` (`koruma-derive-0.9.0/src/expand/derive.rs:22`),
with per-field getters named after source field idents (`e.username()`, per
`examples/forms-demo/src/main.rs:373-393`).

Note [D5] applies to everything below: the derive emits state, metadata, and
facade types; it does not emit event subscriptions. Prototyping scaffolds are
app-authored output and may wire events.

### Feature 1 — Touched-field tracking state + facade

Touched state must NOT live in the holder: it would pollute the unconditional
`PartialEq` derive that `FormState::is_dirty` relies on
(`tests/form_state_dirty_without_serde.rs` is the guardrail) and would leak
into serde round trips [D2]. State goes in core (§2.1); the derive emits the
per-form facade over typed paths.

| File | Change | Notes |
|---|---|---|
| `crates/gpui-form-core/src/fields.rs` (new) | `TouchedFields` bitset per §2.1 | Headless, no new deps, no feature flag |
| `crates/gpui-form-derive/src/derives/gpui_form/form_state.rs` (new) | `<Name>FormState` facade struct + `FIELD_NAMES`/`FIELD_COUNT`/`field_index()`/`path_for_index()` emission beside the existing `<Name>FormPath` impl | New module; registered in `derives/gpui_form/mod.rs`, spliced into both return sites of `expansion.rs` (see hazards) |
| `crates/gpui-form-derive/src/derives/gpui_form/expansion.rs` | Splice facade tokens next to `field_path_tokens` in **both** the empty-form early return (line 73) and the main `quote!` (line 360) | Empty path: emit type with zero per-field methods (mirrors `field_path.rs` contract) |
| `crates/gpui-form/src/lib.rs` | Re-export `TouchedFields` from `core` | Follows `FormState`/`FieldPath` precedent; no flag |

No new feature flag (headless, mirrors FLAT v1). Backward compat: only adds
new pub items; existing expansions byte-identical apart from the added
`<Name>FormState` item.

**Tests:** core unit tests in `fields.rs` (`touch/clear/is_touched`);
behavior test `crates/gpui-form-derive/tests/form_state_facade.rs` —
`FIELD_COUNT` matches non-skipped fields, skipped field has no constructor,
empty form emits a bare type (mirror `field_path.rs` structure and
`empty_form_still_emits_path_type`); token-level unit in
`derives/gpui_form/tests.rs` asserting the facade appears for
`#[gpui_form(empty)]`.

### Feature 2 — Per-field dirty/changed-fields + patch emission (backlog #9)

Diff must be **generated per field** because `FormState<H>` only has
`H: PartialEq`; whole-value equality cannot name fields. The generated
`<Name>FormPatch` + facade methods per §2.2.

| File | Change | Notes |
|---|---|---|
| `crates/gpui-form-derive/src/derives/gpui_form/form_state.rs` | Emit (a) `<Name>FormPatch` with one `Option<Option<T>>` slot per non-skipped field honoring `field_storage` tri-state (`OriginallyOptional` / `WrappedOption` / `Plain`); (b) facade `is_field_dirty`/`changed_fields`/`patch` | Patch values are **holder-typed** per §2.2; reuse the extracted conversion helpers from `value_holder.rs` (`apply_into_conversion` et al.) rather than re-deriving them — extract to a shared `pub(crate)` conversions module if needed |
| `derives/gpui_form/mod.rs`, `expansion.rs` | Register + splice in both return sites | Empty form: emit empty `<Name>FormPatch` with no fields |
| `crates/gpui-form/src/lib.rs` | Re-export nothing new (patch type is per-form) | No flag |

No new flag. Backward compat: additive items only; `PartialEq`/serde derive
list untouched [R15a: no new derives on holders].

**Tests:** behavior test `crates/gpui-form-derive/tests/patch_diff.rs` —
two-field holder, mutate one, assert `changed_fields() == vec![FormPath::age()]`,
patch contains only the changed slot, `WrappedOption` + `Plain` +
`OriginallyOptional` storages all covered, conversion fields (`type/from/into`
like `OverrideForm` in `tests/value_holder_conversions.rs`) patch correctly;
serde-flag matrix run (must compile under default features — mirror
`form_state_dirty_without_serde.rs`).

### Feature 3 — Per-field typed error access (backlog #6)

Emission gated on the already-computed `needs_koruma_derive` in
`generate_value_holder` — no flag. Design per §2.3 [D3].

| File | Change | Notes |
|---|---|---|
| `crates/gpui-form-core/src/validation.rs` (new) | `FormFieldErrors` trait | Re-export via `gpui_form::core` and facade |
| `crates/gpui-form-derive/src/derives/gpui_form/errors.rs` (new) | Emit `<Name>FormFieldErrors` map, `<Name>FormFieldError<'a>` borrowing enum (typed accessors, `error_for`, `failing_fields`, `issue_keys`), and holder `validate_fields()` — names derived from `wrapped_ident` (`format_ident!("{}KorumaValidationError", ...)`, confirmed at `koruma-derive-0.9.0/src/expand/derive.rs:22`) | Implementation matches on `path.path().segments().first()` and forwards to the koruma per-field getter; nested/newtype errors follow the `Inner` variant (see demo `main.rs:391-397` for the shape to bridge) |
| `value_holder.rs` | Thread `needs_koruma_derive` (already computed at line 489) into the new emitter call | Gate on `needs_koruma_derive`, NOT `koruma_options.is_some()` (hazard 6) |
| `expansion.rs` | Splice result next to `i18n_impl` | Empty form + koruma: no fields → `error_for` returns `None` arm only |
| `docs/` + root `README.md` | Document the `error_for(<Name>FormPath::username())` pattern | AGENTS.md sync rule: README section + `.agents/skills/use-gpui-form` |

Backward compat: methods appear **only** on holders whose forms already opt
into koruma — forms without koruma see zero diff. Matches candidate API in
`docs/feature-backlog.md` #6.

**Tests:** behavior test `crates/gpui-form-derive/tests/error_for.rs` —
mirror `koruma_validation_behavior.rs` structs; assert
`error_for(&HolderFormPath::seats())` surfaces the range failure and `None`
for a valid field, wrong-form path returns `None`; i18n interplay covered by
extending `tests/i18n_parity.rs` only if new keys are introduced (prefer
reusing `validation.*` keys); trybuild pass fixture
`tests/ui/error_for_koruma_pass.rs` added to `ui.rs` via a new
`tests.pass(...)` entry.

### Feature 4 — Submit lifecycle state type + opt-in generated alias (backlog #7)

Backlog is explicit: "optional runtime glue, not required generated state.
The holder should remain plain data." Headless type per §2.4 [D7].

| File | Change | Notes |
|---|---|---|
| `crates/gpui-form-core/src/submit.rs` (new) | `SubmitTracker<E>` + `SubmitAction<H>` per §2.4 (with generation guard, `set_error`/`clear_error`) | Unconditional re-export `gpui_form::core::submit`; no async runtime dependency |
| `crates/gpui-form-derive/src/derives/gpui_form/structs.rs` | Add `#[darling(default)] pub submit: bool` to the struct options (attr: `#[gpui_form(submit)]`) | Parse-time only |
| `crates/gpui-form-derive/src/derives/gpui_form/submit.rs` (new) | When `parsed.submit`: emit `pub type <Name>FormSubmission = ::gpui_form::core::SubmitTracker<<#wrapped_ident as ::koruma::Validate>::Error>;` when koruma enabled, else `SubmitTracker<::gpui_form::core::submit::BoxError>` (Box alias). Emit nothing else — GPUI button wiring is app/prototyping concern [D5] | Gated on the attr → zero diff for non-opted forms |
| `expansion.rs` | Splice in both return sites (submit on empty form is legal: alias still emits) | |
| `crates/gpui-form/src/lib.rs` | `pub use gpui_form_core::submit` | |

Feature-flag strategy: none (attr-gated). Backward compat: perfect — nothing
emitted without `#[gpui_form(submit)]`.

**Tests:** behavior `crates/gpui-form-derive/tests/submit_state.rs` — alias
exists, transitions begin→finish(Err)→finish(Ok) carry state, stale
generation completions ignored, holder unchanged by submit; trybuild pass
fixture `tests/ui/submit_attr_pass.rs` (form with `#[gpui_form(submit)]` +
koruma compiles); trybuild compile-fail `tests/ui/submit_attr_takes_no_args.rs`
+ `.stderr` (`#[gpui_form(submit = foo)]` → darling unexpected-expr error)
wired as a new `tests.compile_fail` entry in `tests/ui.rs`.

### Feature 5 — Validation-state cache for live isValid

Cache per §2.5 [D6]; emission conditional on `needs_koruma_derive` (no flag).

| File | Change | Notes |
|---|---|---|
| `crates/gpui-form-core/src/validation.rs` | `ValidationCache<H, O>` per §2.5 | Re-exported via facade |
| `crates/gpui-form-derive/src/derives/gpui_form/form_state.rs` | Facade `validation` field (koruma-gated) + `validate`/`validation`/`is_valid`/`invalidate_validation`/`error_for` methods per §2.5 | Snapshot-compare freshness — sound under `state_mut()` escapes [D6] |
| `expansion.rs`, `mod.rs` | Register + splice both return sites; only when `needs_koruma_derive` | Empty form: no cache field (nothing to validate) |
| `crates/gpui-form-prototyping-core/src/implementations/*.rs` | In each `generate_subscription` `Change` arm, after `update_current_data`, the emitted scaffold may call `state.invalidate_validation()` (or rely on snapshot-compare) — scaffold-authored, not derive-authored [D5] | Keeps prototyping scaffolds "live isValid" with zero app code |

Backward compat: emitted only for koruma forms — but that means **existing
koruma forms gain a new pub item** (same class of additive change as feature
1; document in root README). No flag.

**Tests:** behavior `crates/gpui-form-derive/tests/validation_cache.rs` —
default holder invalid; `is_valid()` caches (assert via the snapshot rule: a
mutation through `state_mut()` makes `validation()` return `None` while
`is_valid()` stays accurate); `reset_to_baseline` invalidates;
`sync_baseline` does not; extend prototyping test that change handlers emit
invalidate calls when koruma metadata present.

### Feature 6 — Async validation attribute surface (`validate(async_fn = ...)`) — parse + emission sketch

| File | Change | Notes |
|---|---|---|
| `crates/gpui-form-codegen/src/components.rs` | Nothing (this is field-level, not component-level) | |
| `crates/gpui-form-derive/src/derives/gpui_form/structs.rs` | New darling struct `AsyncValidate { pub async_fn: syn::Path, pub debounce: Option<syn::Expr> }` with `FromMeta`; add `#[darling(default)] pub validate: Option<AsyncValidate>` to `ComponentField`. Grammar: `#[gpui_form(validate(async_fn = my_check), component(input))]` | Reuse the `TypeOverride`/`DefaultExpr` custom-`FromMeta` pattern for path parsing |
| `crates/gpui-form-schema/src/registry.rs` | Add `with_async_validator(mut self, path: Option<&'static str>)` const builder + `pub async_validator: Option<&'static str>` field on `FieldVariant` (default `None`) — **emission sketch only**, emitted from `expansion.rs` field-variant chain when the attr is present | Keeps metadata-first contract; prototyping consumers can no-op. Must remain `const`-callable (hazard 8) |
| Feature flag | Reserve facade + derive feature name `async-validation` (default off) in both `Cargo.toml` files now; **emission of actual async wiring waits** — until then the attr parses and only metadata is recorded, so no runtime code depends on it | Additive; no existing expansion changes |
| `expansion.rs` | Parse errors must surface at the right place: darling accumulates them (existing `e.write_errors()` path at line 35) | |

Backward compat: new attr unknown to old derives → old darling behavior
(unknown-key error) is expected for users upgrading; opt-in otherwise.

**Tests:** trybuild compile-fail `tests/ui/async_validate_bad_args.rs` +
`.stderr` (`validate(async_fn = 123)`, `validate()` bare word rejected); unit
test in `derives/gpui_form/tests.rs` asserting the emitted FieldVariant chain
contains `with_async_validator(Some("my_check"))`; schema unit test in
`registry.rs` for the builder default.

### Feature 7 — Nested forms `component(nested)` (backlog #2) + repeated fields `component(list(item = ...))` (backlog #3)

**Gated on D8** (FieldPath segment representation decision). The full
cross-cutting change list.

**Parsing (darling):**

| File | Change | Notes |
|---|---|---|
| `crates/gpui-form-codegen/src/components.rs` | Add enum variants to `Components`: `Nested(NestedOptions)` and `List(ListOptions)`. `NestedOptions`: word-only marker (like `CheckboxOptions`) — the field's own type names the nested struct. `ListOptions { pub item: ListItemSpec, pub add_label: Option<String> }` where `ListItemSpec` is a new `FromMeta` enum restricted to scalar kinds (`input`, `number_input`, `select`, `checkbox`, ...) plus `nested`. Grammar per backlog: `component(list(item = input))`, `component(list(item = nested))` | `FromMeta` on `ListItemSpec` mirrors the existing `Components` enum style; reject `item = custom(...)` in v1 (darling custom error) |
| `crates/gpui-form-derive/src/derives/gpui_form/structs.rs` | No new `ComponentField` fields; `component` already routes through `Components`. Add validation helper `ComponentField::is_list(&self)` / `is_nested(&self)` | |

**Holder emission (`value_holder.rs` + `expansion.rs`):**

| File | Change | Notes |
|---|---|---|
| `expansion.rs` | For nested fields: base type = field's base type `Address` → holder field type `<Address>FormValueHolder` — resolve the nested holder ident **by naming convention** `format_ident!("{}FormValueHolder", base_type_last_segment)` (same convention prototyping already relies on at `code_gen.rs:63`). `wrap_in_option` for nested = **false** (Plain storage) so `koruma(nested)` (already parsed! `validation.is_nested` at `expansion.rs:124`) applies directly to the holder field. For list fields: holder field type `Vec<ItemStorage>` — scalar items store plain `Vec<BaseType>` (v1: no per-item Option) and nested items store `Vec<NestedFormValueHolder>` | Hazard: `field_storage` currently only knows Option tri-state — add `FieldStorage::Nested` / `FieldStorage::List` or a parallel enum; every `match field_storage(...)` arm in `value_holder.rs` (`to_wrapped`, `from_wrapped`, `try_from`, `present_fields_json`) needs a new arm |
| `value_holder.rs` | Conversions: nested `From<Original>`/`From<Holder>` reuse the **nested type's own generated From impls** (compose: `address: Address::from(from.address)` / `<Address>FormValueHolder::from(value)`), so `apply_from_conversion`/`apply_into_conversion` get a nested branch that wraps in the sibling type's conversion. List: `to_wrapped` maps element-wise; `try_from` maps with `?` collecting first error (error type gains an index: extend `<...>ConversionError` with `pub index: Option<usize>` — additive field, keep old construction sites compiling). Emit list mutation API on the holder: `pub fn add_<field>(&mut self, value: Item)`, `pub fn remove_<field>(&mut self, index: usize)`, `pub fn <field>_items(&self) -> &[Item]`; v1 reorder deferred (backlog says add/remove before drag) | `needs_required_validation` must be false for nested/list (they're not Option-wrapped; hazard 5) |
| `field_path.rs` + `crates/gpui-form-core/src/path.rs` | **Path composition [gated on D8].** Emit: (a) per nested field a consuming appender (`pub fn address(self) -> Self`) and `pub fn address_root() -> Self`; (b) per list field `pub fn contacts(index: usize) -> Self` producing segments `["contacts", "<index>"]`; (c) typed composition so `ContactsFormPath::contacts(2).name()` → `contacts.2.name`. D8 options: `enum PathSegment { Name(&'static str), Index(usize) }` shipped as the 0.6 core change, or a separate typed `<Name>FormListItemPath` newtype; `FieldPath` gains `push`/`join` additively either way. String formatting of indices is rejected | See R9 for why this cannot wait |

**Schema metadata:**

| File | Change | Notes |
|---|---|---|
| `crates/gpui-form-schema/src/components.rs` | `ComponentKind` += `Nested`, `List`; new Copy structs `NestedBehaviour { pub form_struct: &'static str }`, `ListBehaviour { pub item: ComponentKind, pub item_struct: Option<&'static str> }` (item_struct set only for `item = nested`); `ComponentsBehaviour` += `Nested(NestedBehaviour)`, `List(ListBehaviour)` — **stays `Copy`** because behaviours box nothing. Update `kind()`, `is_value_only_field` (nested/list → true: they render no leaf control), `subscribable` (false v1), `focusable` (false), `default_wraps_in_option` (false) | **Semver note:** adding variants to public `ComponentKind`/`ComponentsBehaviour` breaks exhaustive matches in downstream consumers of `gpui-form-schema`; call it out in CHANGELOG; do NOT add `#[non_exhaustive]` retroactively |
| `crates/gpui-form-schema/src/registry.rs` | No shape changes needed beyond the behaviour variants — `FieldVariant.value_type` keeps the full `Vec<String>` / `Address` string; add convenience `FieldVariant::is_list()` / `is_nested()` | Const-constructibility preserved (all builders stay `const fn` — inventory contract at `registry.rs` doc; hazard 8) |
| `crates/gpui-form-derive/src/derives/gpui_form/expansion.rs` | `behaviour_tokens` routing: `Components::Nested` → `::gpui_form::schema::components::ComponentsBehaviour::Nested(NestedBehaviour { form_struct: "<base type>" })`; `Components::List` → `List(ListBehaviour { ... })`. Nested fields must ALSO appear in `field_variant_construction_code` (they have `component`, so the existing filter at line 227 already includes them) | |
| `crates/gpui-form-codegen/src/components.rs` | `kind()`, `wraps_in_option()` (false for both), `generate_field_layout` (emit **no** `FormFields` entity field / `FormComponents` constructor for v1 — value-only, like checkbox/switch), `behaviour_tokens` arms | |

**Validation error paths:**

| File | Change | Notes |
|---|---|---|
| `errors.rs` (feature 3 emitter) | Nested: first segment `address` → delegate into `errors.address()` (koruma nested error getter) and recurse with the remaining segments against the nested form's `error_for` — requires the nested error type name `<AddressFormValueHolder>KorumaValidationError`, again predictable by convention. List: `contacts.2.name` → `errors.contacts()` element errors indexed at `2` (koruma element errors — `element_validators` already exist in `ValidationInfo`); expose as `Option<&ElementErrors>` or flatten | Verify koruma's nested/element getter names against `koruma-derive-0.9.0/src/expand/derive.rs` before pinning the sketch |

**Prototyping (`FieldCodeGenerator` mapping):**

| File | Change | Notes |
|---|---|---|
| `crates/gpui-form-prototyping-core/src/implementations/nested.rs` (new) | `FieldCodeGenerator` for nested: imports the nested form's generated types; `render_child` emits a bordered group that instantiates the nested scaffold or a simple per-field row set; `generate_subscription` → None (nested manages itself) | Registered in the `field_generator` match in `implementations/mod.rs` |
| `crates/gpui-form-prototyping-core/src/implementations/list.rs` (new) | List: render children as repeatable rows over `current_data.<field>_items()` with per-row controls + add/remove buttons wired to `current_data.add_<field>(...)` / `remove_<field>(index)` | Scaffold-authored event wiring [D5] |
| `implementations/mod.rs` | Extend `field_generator` match; `collect_i18n_keys` (line 422): index-free item keys — list scalar: `{form}.{field}_item_label`; list-of-nested: `{form}.{field}_label` for the group plus the nested form's own keys; add button key `{form}.{field}_add` | i18n shapes stay index-free: keys must never contain indices (hazard 11) |
| `code_gen.rs` | `FormParts` unchanged structurally; `render_children` section-heading logic must skip list/nested groups correctly (they carry their own layout) | |

**Feature-flag strategy for #7:** no new cargo features. Value-only rendering
means the GPUI runtime crates need nothing new in v1. Semver hazard on schema
enum variants is the cost; alternative (new `FieldVariant.list: Option<...>`
field instead of behaviour variants) avoids the break — **recommend the
behaviour-variant route** for consistency, documented in CHANGELOG, since
`gpui-form-schema` is public-integration and workspace-internal consumers
(derive, prototyping) are updated in the same change per AGENTS.md.

**Tests for #7:**

- Behavior: `crates/gpui-form-derive/tests/nested_forms.rs` — nested holder
  composition, `From` both directions, `koruma(nested)` fires through,
  `error_for` recursion; `crates/gpui-form-derive/tests/list_fields.rs` —
  add/remove/mutation API, element validator fires with index in error,
  `try_from` reports `index`; `crates/gpui-form-derive/tests/list_paths.rs` —
  `contacts(2).name()` displays as `contacts.2.name`, equality.
- Trybuild compile-fail: `tests/ui/list_requires_item.rs` + `.stderr` (bare
  `component(list)`), `tests/ui/list_item_unknown_kind.rs` + `.stderr`
  (`item = banana`), `tests/ui/list_item_custom_rejected.rs` + `.stderr` (v1
  rejection); pass fixture `tests/ui/nested_list_pass.rs` (backlog's exact
  `PostEditor` example compiles end-to-end with two forms + `SelectItem`
  derive).
- Unit: codegen `components.rs` tests for `ListOptions::from_list`; schema
  `components.rs` tests for new kinds' predicates; prototyping
  `list.rs`/`nested.rs` tests asserting row scaffolding tokens and index-free
  i18n keys (mirror `i18n_key_items_document_keys_and_skip_overridden_fields`
  in `code_gen.rs:945`).
- Example sync (AGENTS.md): extend `examples/some-lib` with a nested + list
  struct and a `forms-demo` screen; update root `README.md`
  supported-component table + `crates/gpui-form-derive/README.md` +
  `.agents/skills/use-gpui-form` + `docs/feature-backlog.md` statuses in the
  same change.

### Pipeline hazards (where current emission fights these features)

1. **Empty-form early return — `expansion.rs:44-82`.** Everything is emitted
   twice: once in the early return, once in the final `quote!` at line 360.
   `field_path.rs` and `value_holder.rs` already do this; every new emitter
   above (facade/touched, patch, errors, submit, validation cache) MUST splice
   into both, or `#[gpui_form(empty)]` forms silently lose the feature (only
   caught by tests like `empty_form_still_emits_path_type` in
   `tests/field_path.rs:112`).
2. **Skipped fields (`#[gpui_form(skip)]`).** Absent from holder fields,
   `FieldVariant`, and path constructors, but present as `into_original`
   params (`value_holder.rs:591-599`). Touched/patch/error surfaces must not
   reference them; `FormState` docs already note the `into_original`
   limitation (`core/src/state.rs:19-23`). The `has_skipped_fields` branch
   swaps `From` for `try_from`+bon `Builder` — patch emission must handle the
   skipped-params holder shape.
3. **Generic structs.** The holder carries `split_for_impl` generics
   (`value_holder.rs:536`), but `<Name>FormFields`, `<Name>FormComponents`,
   and `<Name>FormPath` are emitted with **no generics** (`expansion.rs:363`,
   `field_path.rs:20-24` documents the deliberate choice). Any new generated
   type must pick a side explicitly: state types (facade/touched/patch/
   validation cache) follow the `FormPath` no-generics rule; impls over the
   holder must reuse `split_for_impl` or they won't compile for generic forms.
4. **Holder field-storage tri-state + conversions.** `field_storage`
   (OriginallyOptional/WrappedOption/Plain), `type`/`from`/`into` overrides,
   and `default_expr` (which changes `to_wrapped` semantics at
   `value_holder.rs:223-252` and suppresses the `Default` derive, switching to
   `generate_default_impl`) mean every per-field feature has ≥6 code paths per
   field. Patch/dirty/touched emission must reuse the extracted conversion
   helpers, not re-derive them.
5. **Synthetic `RequiredValidation` double-counting.** `fields_requiring_required`
   is injected into FieldVariant `validations` (`expansion.rs:253-257`) and as
   a koruma builder on the holder (`value_holder.rs:447-451`). Error-map and
   validation-cache features must not add a second required check, and
   `needs_required_validation` must stay false for nested/list (they are
   Plain, non-Option).
6. **`enable_koruma` vs `effective_enable_koruma` vs `needs_koruma_derive`.**
   Three near-identical gates (`expansion.rs:106/210-211`,
   `value_holder.rs:486-489`). Feature 3/5 emission must gate on
   `needs_koruma_derive`, not `koruma_options.is_some()`, or
   `RequiredValidation`-only forms (e.g. `NameForm` in
   `tests/koruma_validation_behavior.rs`) silently lose error access.
7. **`FieldPath` segments are `&'static str`** (`core/src/path.rs:74`). List
   index segments (`contacts.2`) cannot be stored without either leaking
   (`intern_index`-style) or changing the public representation — this is
   exactly D8. Decide before #3 lands; the leak approach is additive but must
   be documented as bounded-by-usage; path persistence
   (`serde_round_trip.rs`-style) would otherwise be unaffected.
8. **Inventory const-constructibility.** `FieldVariant` builders are
   `const fn` because the derive emits `inventory::submit!` chains into the
   user crate (`registry.rs` module docs; `expansion.rs:318-336`). Every new
   builder (`with_async_validator`, nested/list behaviour payloads) must
   remain const-callable with `&'static str`/`Copy` payloads only — no
   `String`, no `Box`.
9. **`ComponentsBehaviour` is `Copy` and exhaustively matched.** Adding
   `Nested`/`List` variants touches `kind()`, `is_value_only_field`,
   `subscribable`, `focusable`, `default_wraps_in_option`, the derive's
   `behaviour_tokens`, and prototyping's `field_generator` match, all in one
   change — missing any arm is a compile error in-crate but a semver break
   downstream (`gpui-form-schema` is public-integration).
10. **Two parallel field parses.** Darling (`ComponentField`) and
    `koruma_derive_core::parse_field` (`expansion.rs:91-104`) parse
    independently; `component(nested)` interacting with `#[koruma(nested)]`
    must be reconciled at expansion time, not assumed — a nested field that is
    *not* a `#[derive(GpuiForm)]` type will fail only at name-resolution time
    in user code, so a trybuild pass fixture pinning the happy path is
    mandatory.
11. **Prototyping i18n keys assume flat fields.** `collect_i18n_keys`
    (`implementations/mod.rs:422`) emits `{form}.{field}_label/_description`
    unconditionally per field; list/nested must produce index-free keys or
    the emitted `*_I18N_KEYS` const will reference keys that can never resolve
    for per-item messages (parity-tested by `tests/i18n_parity.rs`).
12. **Unconditional new pub items collide with user names.** Features 1/2
    emit `<Name>FormState` / `<Name>FormPatch` into the user's module even
    when unused. Keep the `<Name>Form*` reserved-prefix convention documented
    in the root README, and prefer attr-gating (`#[gpui_form(submit)]`) over
    unconditional emission wherever the feature is genuinely opt-in (as done
    for #4).

---

## 4. Delivery Plan: Sequencing, Sync & Workflow

### 4.0 Starting state (from the repo)

Shipped primitives this plan builds on: `FormState<H>` (boolean-level
dirty/reset/diff, `crates/gpui-form-core/src/state.rs`), `FieldPath` FLAT v1
(`crates/gpui-form-core/src/path.rs`, generated `<Name>FormPath` from
`crates/gpui-form-derive/src/derives/gpui_form/field_path.rs`), layout
metadata v1 (`crates/gpui-form-schema/src/layout.rs`), and Koruma holder
mirroring (`crates/gpui-form-derive/src/derives/gpui_form/koruma.rs`).
`crates/gpui-form-core/docs/ARCHITECTURE.md` already names `FieldPath` as "the
shared naming foundation for the upcoming field-level validation (#6),
field-level diff/delta reporting (#9), schema export (#14), and nested/list
paths (#2/#3)" — that sentence is effectively the dependency spine of this
plan. Backlog "Suggested order" (docs/feature-backlog.md) confirms: error/
patch work before nested/repeated, partial before dependency metadata, schema
export near the end.

### 4.1 Dependency diagram

```
SHIPPED: #8 FieldPath/<Name>FormPath (FLAT v1)   #1 FormState (boolean dirty)   Koruma holder mirroring
                          │
        ┌─────────────────┴──────────────────────────────────────────┐
        │  M1  FIELD-LEVEL STATE MODEL  (keyed by FieldPath)         │
        │  (a) touched tracking   (b) #9 per-field dirty/patch       │
        │  (c) #6 per-field typed error access                       │
        └───────┬──────────────────────┬─────────────────────┬───────┘
                │                      │                     │
   errors need a home;            patch/touched define   M3 must compose
   isValid caches them            sparse updates         over multi-segment paths
                │                      │                     │
        ┌───────▼──────────┐           │            (design rule: M1 storage
        │ M2  VALIDATION & │           │             keys on FieldPath, never on
        │ SUBMIT LIFECYCLE │           │             flat ctor lists only)
        │ (e) cache/isValid│           │                     │
        │   ↓ gates        │           │                     │
        │ (d) #7 submit    │           │                     │
        │   ↓ settles      │           │                     │
        │ (f) async valid. │           │                     │
        └──────────────────┘           │                     │
                                       ▼                     ▼
                            ┌──────────────────────────────────────┐
                            │ M3  COMPOSITION  (codegen-heavy)     │
                            │ [gated on D8]                        │
                            │ (g) #2 nested  ──→  (h) #3 Vec<T>    │
                            │ (list-item paths are nested paths +  │
                            │  an index segment; #3 backlog:       │
                            │  "scalar item components before      │
                            │  nested item forms")                 │
                            └──────────────┬───────────────────────┘
                                           │ nested/list-aware paths
        ┌──────────────────────────────────┤
        │ (i) #10 partial holders [D10]    │ (j) #12 deps + #13 conditional req.
        │ consumes (a)/(b) touched/changed │ required_if/depends_on reference
        │ to pick included fields          │ sibling fields as FieldPaths; errors
        │                                  │ from (c) must be addressable
        └──────────────┬───────────────────┴───────────┬──────────┘
                       └────────────┬──────────────────┘
                                    ▼
                    ┌───────────────────────────────────┐
                    │ M5  (k) #14 SCHEMA EXPORT         │
                    │ exports the metadata model AFTER  │
                    │ nested/list/dependency shapes     │
                    │ exist, or v1 is immediately stale │
                    └───────────────────────────────────┘
    Follow-ups (unscheduled): focus management (gap 4), devtools panel
    (gap 8), text-domain validation (#18 remainder), wizard steps (#11)
```

Justification per edge:

- **(c) needs FieldPath**: backlog #6's stated value is "stable typed field
  paths ... so callers do not have to match validation output with ad hoc
  strings."
- **(b) needs FieldPath + the FormState baseline**: core ARCHITECTURE,
  FormState scope: "Field-level patch and delta reporting is backlog feature
  #9 and will build on the FieldPath naming primitive."
- **(e) needs (c)**: a live `isValid` is a derived query over stored per-field
  issues; without error storage there is nothing to cache.
- **(d) needs (e) + (c)**: submit gating consumes validity/pending state;
  `last_error` renders through typed error access. Backlog #7 constrains it
  to "optional headless workflow state ... The holder should remain plain
  data."
- **(f) needs (e) + (d)**: async validators introduce a Pending state in the
  cache and must settle before submit proceeds.
- **(g)/(h) need M1 designed path-keyed**: `FieldPath` is already
  multi-segment capable (`FieldPath::new(&["a", "b"])` works today), so M1
  must key touched/dirty/error storage on `FieldPath` semantics (the facade's
  index mapping is an implementation detail behind typed paths), never on
  ad-hoc string lists — otherwise M3 forces a storage migration. The index
  representation itself is exactly D8.
- **(h) needs (g)**: a list-item path is a nested path plus an index segment;
  backlog #3 explicitly sequences "scalar item components before nested item
  forms."
- **(i) needs (a)/(b)**: backlog #10's headline workflows are "edit screens
  that only send touched fields" and PATCH payloads — those consume the
  changed/touched field sets. Gated by D10.
- **(j) needs (c) + typed paths**: backlog #13: "should integrate with the
  validation layer rather than live as display-only metadata ... typed field
  paths are the likely foundation."
- **(k) needs M3 + M4 metadata**: the backlog's Suggested order places schema
  export after nested, repeated, and dependency metadata; exporting earlier
  freezes a schema shape that cannot describe nested, list, or
  conditional-required fields and gets reworked immediately.

### 4.2 Milestone 1 — Field-level state model (headless)

**Features:** (a) touched-field tracking, (b) per-field dirty/patch (#9),
(c) per-field typed error access (#6). **Effort: M.**

**Design rule (load-bearing for M3):** all three stores are keyed by
`FieldPath` semantics (the generated facade maps paths to dense indices as an
implementation detail), and iteration comes from a generated `all_paths()` on
`<Name>FormPath` — not from ad-hoc per-field string lists.

| Crate/File | Change | Why |
|---|---|---|
| `crates/gpui-form-core/src/fields.rs` (new) | `TouchedFields` bitset | Core's boundary is UI-neutral helpers |
| `crates/gpui-form-core/src/validation.rs` (new) | `FormFieldErrors` trait (error map is generated) | #6 needs a home for the generic contract |
| `crates/gpui-form-derive/src/derives/gpui_form/form_state.rs` (new) | `<Name>FormState` facade + `<Name>FormPatch` + path→index tables | D1 composition point |
| `crates/gpui-form-derive/src/derives/gpui_form/errors.rs` (new) | koruma-gated error map / `validate_fields()` / `error_for` | §2.3 |
| `crates/gpui-form-derive/src/derives/gpui_form/field_path.rs` | `FIELD_NAMES`/`FIELD_COUNT`/`all_paths()`/`field_index()`/`path_for_index()` | FLAT v1 additive extension; feeds iteration for touched/diff |
| `crates/gpui-form/src/lib.rs` | Facade re-exports of the new core types | Facade ARCHITECTURE: "if a lower-level crate adds a new public surface ... it usually needs a facade re-export" |
| `crates/gpui-form-core/src/lib.rs` | Export new types; keep headless | Core "Modules" section owns the export surface |
| New derive tests | Per §3 features 1–3 test lists, including a default-features compile test mirroring `form_state_dirty_without_serde.rs` | Derive "Tests" convention; R13 |

- **Doc/skill sync:** root README §Saving, Restoring, and Dirty Tracking
  (per-field diff/patch + touched), §Typed Field Paths (`all_paths`,
  `error_for`), §Validation With Koruma (typed access);
  `crates/gpui-form-core/README.md` and `crates/gpui-form-derive/README.md`;
  `.agents/skills/use-gpui-form/SKILL.md` Feature Gotchas (extend "Serde and
  dirty tracking" + "Typed field paths" subsections); `references/api-map.md`
  (Form-State Persistence pattern, Typed Field Paths pattern, Generated
  Names); `crates/gpui-form-core/docs/ARCHITECTURE.md` (new module +
  scope-boundary update), `crates/gpui-form-derive/docs/ARCHITECTURE.md`
  (holder responsibilities + tests list), `crates/gpui-form/docs/ARCHITECTURE.md`
  (re-export list); `docs/form-state-persistence.md`;
  `docs/feature-backlog.md` (mark #6/#9 shipped inline, update Suggested
  order); `examples/README.md` forms-demo blurb.
- **Validation (narrowest first):** `cargo test -p gpui-form-core` →
  `cargo test -p gpui-form-derive` → `cargo test -p gpui-form --features serde`
  → `cargo test -p forms-demo` → `just test` → `just clippy` +
  `cargo fmt --check` (CI parity).
- **Definition of done:** default-feature (no serde) touched/diff/error tests
  green; every koruma holder gains the accessors without name collisions
  (proven by some-lib's koruma forms compiling); docs synced as above;
  backlog updated.
- **forms-demo showcase:** footer graduates from `is_dirty` to a live
  changed/touched field list rendered via `FieldPath` Display, plus per-field
  error display using `error_for` alongside the existing `validation.*` `t!`
  rendering. Also removes the per-frame `validate()`/holder-clone smell
  (R3) by using the facade.

### 4.3 Milestone 2 — Validation and submit lifecycle (headless)

**Features:** (e) validation-state cache/live isValid, (d) submit lifecycle
(#7), (f) async validation contract (+ attr parse surface). **Effort: M.**

| Crate/File | Change | Why |
|---|---|---|
| `crates/gpui-form-core/src/validation.rs` | `ValidationCache` (snapshot-compare [D6]) + facade methods (generated in M1's `form_state.rs`) | (e) needs M1's error types; keeps one source of truth |
| `crates/gpui-form-core/src/submit.rs` (new) | `SubmitTracker` (with generation guard) + `SubmitAction`; `#[gpui_form(submit)]` alias emission | Backlog #7: "optional headless workflow state"; adds zero dependencies, keeping the CI `machete` job clean |
| `crates/gpui-form-core/src/async_validation.rs` (new) | `AsyncFieldValidation` trait (contract only) + `validate(async_fn = ...)` parse/metadata per §3 feature 6 | R19-compliant: contract, not engine |
| `crates/gpui-form/src/lib.rs` | Re-export `ValidationCache`, `SubmitTracker`, `SubmitAction`, `AsyncFieldValidation` | Facade re-export rule |
| `crates/gpui-form-core/docs/ARCHITECTURE.md` | Document modules, async contract, scope boundaries | "When To Update This Document": new helper modules |

- **Deliberately out:** no `gpui-form-component` changes — the backlog Scope
  boundary forbids turning gpui-form into a widget library, and no UI
  pending-spinner primitive is required for a headless lifecycle [D5].
- **Doc/skill sync:** root README new section "Submit Lifecycle and Live
  Validation" (README is the single user-facing surface per AGENTS.md);
  `crates/gpui-form-core/README.md`; SKILL.md Common Patterns entry +
  api-map.md Facade Imports and a new pattern block; core + facade
  ARCHITECTURE.md; backlog #7 status; `examples/README.md` forms-demo blurb.
- **Validation:** `cargo test -p gpui-form-core` → `cargo test -p gpui-form`
  (re-export compile proof) → `cargo test -p forms-demo` → `just test` →
  `just clippy`.
- **Definition of done:** submit gating demonstrably waits on pending async
  validation in unit tests; stale-completion generation guard tested; no new
  dependencies introduced (machete stays green); `--all-features` (CI ubuntu
  + macos jobs) unchanged-green.
- **forms-demo showcase:** a Submit button wired to `SubmitTracker` — button
  disables on `is_submitting`, `last_error` banner reuses the localized
  `validation.*` keys from M1's typed error access, plus an `isValid`
  indicator and a simulated async check on the email field. Any new visible
  strings are app-owned keys in
  `examples/forms-demo/locales/{en,fr-FR,zh-CN}.yml` (keys are crate-local).

### 4.4 Milestone 3 — Nested forms (#2) and repeated fields (#3)

**Features:** (g), (h). **Effort: L.** **Gate: D8 must be resolved first**
(PathSegment enum vs list-item path type — an 0.6 core decision). This is the
only milestone that changes generated code shape for new-syntax forms and
touches the codegen+schema+prototyping triad mandated by AGENTS.md ("When
Editing Components or Generated Form Metadata"). Full change tables in §3
feature 7.

Sequencing inside the milestone: scalar list items before nested item forms
(backlog #3's own rule); prototype the nested *holder + path + error bridge*
end-to-end on one example before finalizing attribute syntax (R16).

- **Doc/skill sync:** root README §Component Syntax (new syntax) + new
  section "Nested Forms and Repeated Fields";
  `crates/gpui-form-derive/README.md` (AGENTS.md: "Keep supported-component
  docs aligned across the root README.md and
  crates/gpui-form-derive/README.md"); SKILL.md Component Selection + Common
  Patterns; api-map.md Supported Component Syntax + Generated Names; codegen,
  schema, derive, prototyping-core, component (if touched), and facade
  ARCHITECTURE.md; `crates/gpui-form-prototyping-core/README.md`;
  examples/README.md (some-lib, forms-demo, prototyping sections); backlog
  #2/#3 statuses.
- **Validation:** `cargo test -p gpui-form-codegen` →
  `cargo test -p gpui-form-derive` → `cargo test -p gpui-form-schema` →
  `cargo run -p prototyping` + review emitted diff → `cargo test -p some-lib`
  → `cargo test -p forms-demo` → `just test` → `just clippy` →
  `just test-publish` (derive-surface change; dry-run the workspace publish).
- **Definition of done:** nested/list forms compile with typed error access
  from M1 working through composed paths (the payoff of the M1 keying rule);
  prototyping output regenerates cleanly; existing flat forms emit
  **byte-identical tokens** (regression-checked via an expansion snapshot
  test) so no published-API churn for existing users; docs synced.
- **forms-demo showcase:** an Address section (nested form) and a tags list
  with add/remove buttons; list count text goes through the app's
  pluralized-locale pattern already proven by the file picker.

### 4.5 Milestone 4 — Partial holders (#10) [gated D10], dependency metadata (#12), conditional requiredness (#13)

**Features:** (i), (j). **Effort: M.**

| Crate/File | Change | Why |
|---|---|---|
| `crates/gpui-form-derive/src/derives/gpui_form/structs.rs` + `expansion.rs` | `#[gpui_form(partial_holder)]` struct attribute; emit `<Name>FormPartialHolder` (all-`Option`) with tri-state semantics | Backlog #10; new generated type is additive. Build only if D10's gate says patch (#9) cannot express clear-vs-unchanged |
| `crates/gpui-form-derive/src/derives/gpui_form/value_holder.rs` | Holder↔partial conversions; `to_partial(touched)/apply_partial` consuming M1's touched/changed sets | Backlog #10 workflows: "Edit screens that only send touched fields" |
| `crates/gpui-form-codegen/src/components.rs` | Parse `depends_on = <field>` and `required_if = "<field>"`; propagate to metadata emission | Codegen owns parse-time attributes |
| `crates/gpui-form-schema/src/components.rs`/`registry.rs` | `FieldVariant` gains dependency payloads storing **`FieldPath` references** (const-constructible, `FieldLayout` precedent) | Backlog #12/#13: metadata that "generated or application code can consume"; #13: "typed field paths are the likely foundation" |
| `crates/gpui-form-core/src/validation.rs` | Pure conditional-required check (`required_when`) integrated with M1 error types | Backlog #13: "integrate with the validation layer rather than live as display-only metadata". Desugars to a generated koruma validator closure, NOT an expression language (R19) |
| `crates/gpui-form-prototyping-core/src/code_gen.rs` | Consume dependency metadata for scaffolded dependent-field placeholders | AGENTS.md prototyping triad rule |
| `crates/gpui-form/src/lib.rs` | Re-exports | Facade rule |
| `examples/some-lib`, `examples/forms-demo/src/main.rs` | Shapes + country→state `depends_on` (with reset-on-parent-change), `required_if = subscribe` email, "save only touched fields" via partial holder | Showcase + example-first docs |

- **Doc/skill sync:** root README new sections "Partial Form Holders" and
  "Field Dependencies and Conditional Requiredness";
  `crates/gpui-form-derive/README.md` (attribute inventory); SKILL.md Common
  Patterns + a Feature Gotchas subsection; api-map.md attribute list + new
  patterns; derive, schema, codegen, prototyping-core, core ARCHITECTURE.md;
  `crates/gpui-form-prototyping-core/README.md`; examples/README.md; backlog
  #10/#12/#13 statuses.
- **Validation:** `cargo test -p gpui-form-derive` →
  `cargo test -p gpui-form-core` → `cargo run -p prototyping` →
  `cargo test -p forms-demo` → `just test` → `just clippy`.
- **Definition of done:** conditional-required violation surfaces as a typed,
  path-addressable error (not display-only); partial holder round-trips a
  PATCH-style payload in tests; dependency reset behavior demoed in
  forms-demo; docs synced.
- **forms-demo showcase:** country select resetting the state select when
  changed; the subscribe switch flipping email between optional and required
  with the localized error; a "Save changes" action printing only the touched
  fields as a partial payload.

### 4.6 Milestone 5 — Schema export (#14)

**Feature:** (k). **Effort: S** — read-only projection over metadata that
M3/M4 already shaped.

| Crate/File | Change | Why |
|---|---|---|
| `crates/gpui-form-derive/src/derives/gpui_form/expansion.rs` (or new module) | Emit `<Name>FormSchema` with a `schema()` accessor built from the same `FieldVariant`/`FieldLayout` tokens the inventory path already builds | Backlog #14 candidate API `UserProfileFormSchema::schema()`; single metadata source (schema ARCHITECTURE: derive/codegen "should not invent separate parallel runtime models") |
| `crates/gpui-form-schema/src/registry.rs` | Export-friendly view types only if `FieldVariant` needs a stable consumer shape (nested/list/dependency entries included) | Schema owns the metadata boundary |
| `crates/gpui-form/src/lib.rs` | Re-export | Facade rule |
| `crates/gpui-form-prototyping-core` | Optionally consume the export for docs/fixture generation | Backlog #14: "documentation generation, test fixtures, prototyping tools" |
| `examples/forms-demo/src/main.rs` | Debug panel rendering the exported schema of the signup form | CI artifact; proves non-GPUI consumers can read it |

- **Doc/skill sync:** root README new section "Schema Export"; SKILL.md
  Common Patterns (schema export named in the typed-paths pattern's purpose
  list); api-map.md Generated Names (new generated type); derive, schema,
  facade ARCHITECTURE.md; examples/README.md forms-demo blurb; backlog #14
  status + Suggested order refresh.
- **Validation:** `cargo test -p gpui-form-derive` →
  `cargo test -p gpui-form-schema` → `cargo test -p forms-demo` →
  `just test` → `just test-publish`.
- **Definition of done:** exported schema round-trips through serde_json in a
  test (serde is already a facade dev-dependency), includes nested/list/
  dependency metadata from M3/M4, and docs are synced.
- **forms-demo showcase:** a footer/debug toggle printing the form's exported
  schema — a natural extension of the footer's existing typed-path display;
  doubles as the devtools seed (gap 8).

### 4.7 Semver and feature-flag strategy

- **Versioning facts:** workspace version `0.5.2`, `publish = true`,
  single-sourced in the root `Cargo.toml` `[workspace.package]`; `just
  test-publish` dry-runs `cargo publish --workspace`. On a 0.x line, additive
  public API is the safe case; the workspace convention should be **one minor
  bump per milestone that adds public surface** (M1 → 0.6.0, M2 → 0.7.0, M3 →
  0.8.0, M4 → 0.9.0, M5 → 0.10.0) — cheap because the version is defined
  once. Exceptions that consume the same bump: D8's `PathSegment` core change
  and M3's schema enum variants land with the M3 minor.
- **Feature flags:** follow the existing facade pattern (`derive`,
  `inventory`, `serde`, `phone` in `crates/gpui-form/Cargo.toml`). Add a flag
  **only when a new optional dependency appears**. None of M1–M5 requires
  one: M1/M2 are pure `core` additions (`core::future::Future` only — no
  tokio/futures, which also keeps the CI `machete` job green), and M3–M5 are
  derive-surface additions that are additive like the layout hints were. The
  reserved `async-validation` flag (§3 feature 6) is the one planned
  exception. If a future runtime-integration helper (e.g. a tokio-driven
  submit) is wanted, add it then as
  `gpui-form = { features = ["submit-async"] }` mirroring `phone`, and update
  the root README §Installation snippet and the `crates/gpui-form/README.md`
  stub feature list (the stub changes only then — AGENTS.md forbids
  re-growing it otherwise).
- **Ship independence:** every milestone is independently shippable without
  breaking published API. The two risk points are managed by design: (1) M1
  must key all state on `FieldPath` semantics so M3 never migrates storage;
  (2) M3/M4 must keep token output for existing forms byte-identical
  (new syntax only changes new-syntax forms) — enforce with a derive expansion
  snapshot test. Note the workspace lint `unexpected_cfgs = deny` (root
  `Cargo.toml`): any hand-written `cfg` in new feature-gated code must be
  declared, not improvised.
- **CI parity per milestone:** all five must keep
  `cargo test --workspace --all-features` green on both ubuntu and macos
  jobs, `cargo fmt --check`, `cargo clippy --workspace --all-features`, and —
  because CI builds `cargo build --release -p forms-demo` and uploads the
  `forms-demo-macos` artifact — the forms-demo showcase for each milestone
  must compile in release before the milestone closes.

### 4.8 Docs & skills sync matrix

| Milestone | Root README section(s) | Skills files | ARCHITECTURE.md (crates) | Examples |
|---|---|---|---|---|
| M1 (a, b, c) | §Saving, Restoring, and Dirty Tracking; §Typed Field Paths; §Validation With Koruma | SKILL.md Feature Gotchas (dirty-tracking + typed-paths subsections); api-map.md Form-State pattern, Typed Field Paths pattern, Generated Names | core, derive, facade | examples/README.md (forms-demo); docs/form-state-persistence.md |
| M2 (e, d, f) | new §Submit Lifecycle and Live Validation | SKILL.md Common Patterns; api-map.md Facade Imports + new pattern block | core, facade | examples/README.md (forms-demo) |
| M3 (g, h) | §Component Syntax; new §Nested Forms and Repeated Fields | SKILL.md Component Selection + Common Patterns; api-map.md Supported Component Syntax + Generated Names | codegen, derive, schema, prototyping-core, component (if touched), facade | examples/README.md (some-lib, forms-demo, prototyping); crates/gpui-form-derive/README.md; crates/gpui-form-prototyping-core/README.md |
| M4 (i, j) | new §Partial Form Holders; §Field Dependencies and Conditional Requiredness | SKILL.md Common Patterns + Feature Gotchas; api-map.md attribute list + new patterns | derive, codegen, schema, core, prototyping-core | examples/README.md (some-lib, forms-demo); crates/gpui-form-derive/README.md |
| M5 (k) | new §Schema Export | SKILL.md Common Patterns; api-map.md Generated Names | derive, schema, facade | examples/README.md (forms-demo) |

Every milestone also updates `docs/feature-backlog.md` inline statuses and
the "Suggested order" list in the same change, per AGENTS.md synchronization
rule 6 ("Land these updates in the same change unless there is a documented
reason not to").

---

## 5. Risk Register & Rejections

Scope of verification: all claims below were checked in the repo and in
dependency sources under
`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/` (`gpui-pre-0.3.7`,
`gpui-base-0.7.0`, `gpui-component-0.7.0`, `koruma-core-0.9.0`,
`koruma-derive-0.9.0`, `koruma-collection-0.9.0`). Items marked UNVERIFIED
list what to check.

**Headline finding:** the plan's biggest shared assumption — "per-field typed
error access" is something to build from scratch — is half-wrong. koruma
already generates a concrete, per-field, typed error struct per holder
(`SignupFormValueHolderKorumaValidationError` with `errors.username().all()`),
and it aggregates (never short-circuits). What is actually missing is (a) a
*cache/validity* story, (b) a *bridge* from those typed structs to
`FieldPath`, and (c) uniform blur hooks. Several planned features risk
duplicating koruma instead of wrapping it — hence [D3]. Second headline: the
derive is **not headless today** — it unconditionally emits a struct of
`Entity<InputState>` etc. (`<Name>FormFields`), so every new emission that
references a new crate adds a *direct dependency requirement* on every user —
hence [D2]/R14.

| ID | Risk | Evidence (file:line) | Likelihood | Impact | Mitigation |
|----|------|----------------------|------------|--------|------------|
| R1 | Blur→touched is viable for input-family only. `InputEvent::Blur` exists and is emitted by the shared input engine, so `cx.subscribe_in(&field_entity, …)` matching `InputEvent::Blur` works; `Context::on_focus_out(handle, …)` also exists at the view level. But parity across the 10 component kinds is unproven: checkbox/switch/select/date-picker/file-picker emit their own event enums; whether each can report blur is UNVERIFIED. Check `SelectEvent`/`DatePickerEvent`/`FilePickerEvent` variants and each gpui-kit component's focus-handle exposure before promising "touched tracking" for all kinds. | gpui-base-0.7.0/src/input/base/state.rs:123-128 (`Change/PressEnter/Focus/Blur`), `:3387` (`cx.emit(InputEvent::Blur)`); gpui-pre-0.3.7/src/app/context.rs:635-667 (`on_focus_out`), `:566` (`on_focus_in`), `:788` in elements/div.rs (`track_focus`); demo wiring examples/forms-demo/src/main.rs:152-161 | High | Medium | Ship touched tracking v1 for input/number_input/phone_input only [D4]; document other kinds as "touched on value change, not blur" until each event enum is verified. |
| R2 | Blur-event edge case: the input's `on_blur` early-returns (no `Blur` emitted) when a context menu / select popover is open, so "user clicked away to a select" never marks the field touched. Also `clamp_number_value` mutates the value *before* emitting Blur. | gpui-base-0.7.0/src/input/base/state.rs:3377-3379 (early return), `:3386` (clamp), `:3387` (emit after) | Medium | Low | Accept documented gap in v1; add regression note. Alternative (poll `focus_handle.is_focused(window)` in render) is worse. |
| R3 | Live-isValid cost is per-*render* today, not per-keystroke, and it is already heavy-ish: `FormsDemo::render` calls `self.data.validate()` every frame and constructs `SignupFormValueHolder::default()` + a full holder clone for the ad-hoc dirty check. koruma's `validate()` allocates the whole error struct up front and runs every validator; capturing validators **clone the field value on every call, even when passing** (`with_value_ref` → `with_value(value.clone())`; only `RequiredValidation` is `skip_capture`). At demo scale (~10 fields) this is tens of small allocations — fine; at nested/repeated scale it is O(fields × validators) clones per frame. | examples/forms-demo/src/main.rs:371, `:409-411`; koruma-derive-0.9.0/src/expand/derive.rs:1794-1808 (error struct constructed per call); koruma-derive-0.9.0/src/expand/validator.rs:176-182 (capture clones); koruma-collection-0.9.0/src/validators/general/required.rs:27-31 (skip_capture) | Medium | Medium | Cache must be invalidated-on-change (snapshot-compare [D6]), never recomputed in render. Do not "fix" by short-circuiting koruma — it structurally aggregates. Measure with a 50-field form before shipping the cache. |
| R4 | Validation cache soundness hole: `FormState::current_mut()` hands out `&mut H`, so app code can mutate the holder with no hook to invalidate a cached isValid/errors. Any flag-based cache built *inside* `FormState<H>` is unsound by design. | crates/gpui-form-core/src/state.rs:93-95 (`current_mut` returns `&mut H`) | High (if cache is put in FormState) | Medium | Snapshot-compare cache [D6] placed in the generated facade; `FormState<H>` stays frozen [D1]. Never expose `current_mut` on the caching type. |
| R5 | Re-entrancy: gpui **panics** on re-entrant entity update. Generated "helper" APIs that take `&Entity<View>` and call `.update()` from inside that view's own subscription handler (the natural shape of a generated on-change/on-blur hook) will hard-panic. Handlers currently run with `&mut Self` already leased (main.rs:185-200). | gpui-pre-0.3.7/src/app/entity_map.rs:164, `:231-232` ("cannot update {T} while it is already being updated"); examples/forms-demo/src/main.rs:185-200 | Medium | High | [D5] Keep the change-wiring app-authored (as today); any new glue must be methods on `&mut self` of the owning view, never entity-handle-based setters. |
| R6 | Notify discipline: existing change handlers mutate `self.data` **without** `cx.notify()`; error display refreshes only as a side effect of `subscribe_in` observation of the input entity. Generated hooks that forget `cx.notify()` produce stale error rendering that "works in the demo, breaks in real apps". | examples/forms-demo/src/main.rs:191-200 (no notify); input notifies internally at gpui-base-0.7.0/src/input/base/state.rs:2895-2896 | High | Medium | Any new glue documents/tests the notify contract; add a manual integration check in forms-demo (error text must update on each keystroke). |
| R7 | koruma is all-errors, no short-circuit, and `ValidationError` is only `is_empty/has_errors` — there is **no** map-like interface, no field iteration, and no path concept. A `HashMap<FieldPath, E>` fed "generically" from koruma output is impossible without generated per-form bridge code mapping `FormPath::username()` → `errors.username()`. Nested/newtype errors are recursive typed structs (`Inner` variant), not flat paths. | koruma-core-0.9.0/src/lib.rs:8-24, `:53-59`; koruma-derive-0.9.0/src/expand/derive.rs:1794-1808 (aggregate loop); demo error match on `Inner` at examples/forms-demo/src/main.rs:391-397; accessors at koruma-0.9.0/README.md:189-194 | High | Medium | Plan the bridge as *generated* code (per field: path const + closure reading the typed accessor) [D3]. Reject any String-keyed map (rejection 4). |
| R8 | Error-map-by-path collides with the error *struct's* lifetime: koruma error structs are owned snapshots (validators store cloned values). Caching them keyed by `FieldPath` is fine (FormPath is `Hash+Eq`), but caching *borrowed* references is a borrow trap. Also two error types exist per form (source struct's and holder's — demo derives Koruma on both, main.rs:79 and value_holder.rs:502); users will grab the wrong one. | examples/forms-demo/src/main.rs:79, `:382-397` (`SignupFormValueHolder*Validator` names); crates/gpui-form-derive/src/derives/gpui_form/value_holder.rs:501-506 | Medium | Low | Specify in docs: the holder's error type is canonical for form UIs. Cache owned error snapshots. |
| R9 | Typed paths cannot express repeated-field indices: `FieldPath` segments are `&'static str` (`Box<[&'static str]>`), constructors use `stringify!`. `tags.3` requires dynamic segments — changing the segment type breaks the published 0.5.2 primitive; keeping `&'static str` forces an index side-channel that will try to creep back as string formatting (`format!("tags.{}", i)`) — the exact stringly regression the paths feature was built to kill. | crates/gpui-form-core/src/path.rs:74-99; crates/gpui-form-derive/src/derives/gpui_form/field_path.rs:35-46 | High (when #3 starts) | High | [D8] Decide *now*, before #2/#3: either `enum PathSegment { Name(&'static str), Index(usize) }` shipped as a 0.6 breaking change of the core primitive, or a separate typed `ListItemPath` newtype. Do not format strings. |
| R10 | FormState god-object / breaking churn: adding touched+errors+validation-mode to `FormState<H>` changes `new()` and method bounds of a published type (`H: Clone`/`H: PartialEq` today). All 9 crates are published at 0.5.2, `publish = true`. | crates/gpui-form-core/src/state.rs:61-142; Cargo.toml:19-26 (version 0.5.2, publish = true) | Medium | High | [D1] New state = new types composed by the generated facade. FormState stays frozen; 0.6.0 only if composition proves awkward. |
| R11 | Serde coupling: touched/error/submit state must not live in the holder. Holder serde is opt-in value persistence (derives appended only under `#[cfg(feature = "serde")]`); persisting UI-session state corrupts round-trip semantics, and any new *field* on the holder breaks literal construction used pervasively in tests. | crates/gpui-form-derive/src/derives/gpui_form/value_holder.rs:507-520; literal construction at crates/gpui-form-derive/tests/koruma_validation_behavior.rs:50; docs/form-state-persistence.md:25-43 | High (if ignored) | High | [D2] Runtime state types live in core/component, are `#[derive(Clone)]`-only, never serde, never part of `<Name>FormValueHolder`. |
| R12 | Async validation / submit lifecycle: `cx.spawn_in` exists, but Task cancellation-on-drop semantics in this fork are UNVERIFIED (check `Task`/`BackgroundExecutor` in gpui-pre-0.3.7/src/executor.rs). Stale async results (validation or submit finishing after reset/second submit) will write wrong `isSubmitting`/errors without an epoch counter. Cross-thread note: async validators need `Send` + weak entity; `view.update` on a dropped entity is already handled via weak (`is_ok()` pattern at context.rs:580-583). | gpui-pre-0.3.7/src/app/context.rs:667+ (`spawn_in`); cancellation semantics UNVERIFIED | Medium | Medium | [D7] Require a generation/epoch counter in the submit/validation-state type from v1; treat Task-drop-cancellation as best-effort. |
| R13 | Feature-unification compile matrix: generated code shape already varies by feature (`PartialEq` unconditional, `Serialize/Deserialize` gated; `bon::Builder` only when skipped fields exist). Each new cfg-gated emission multiplies the matrix — the `form_state_dirty_without_serde.rs` regression test exists precisely because a past gating was wrong. Workspace builds unify features, so "it compiles with serde off" can silently become false in downstream workspaces. | crates/gpui-form-derive/src/derives/gpui_form/value_holder.rs:494-520; crates/gpui-form-derive/tests/form_state_dirty_without_serde.rs:1-19; facade features at crates/gpui-form/Cargo.toml [features] | Medium | Medium | Every new emission gets a default-features compile test mirroring form_state_dirty_without_serde.rs; feature flags threaded facade→derive with `gpui-form-derive?/x` (existing precedent in facade [features]). |
| R14 | Generated-code dependency creep: derive output already requires `::gpui`, `::gpui_kit` (the `<Name>FormFields` struct is emitted unconditionally), `::koruma` (holder derive), and `::rust_i18n` under `koruma(fluent)` — all must be *direct* deps of the user crate (proc-macros can't inject deps). some-lib lists gpui/gpui-kit as machete-ignored direct deps purely for generated code. Each new generated reference to a new crate adds an undocumented direct-dep requirement and a support burden. | crates/gpui-form-codegen/src/implementations/input.rs:19-40 (`Entity<InputState>` in emitted struct/fn); value_holder.rs:499, `:502`, `:775` (`::gpui_form::bon`, `::koruma::Koruma`, `::rust_i18n::t!`); examples/some-lib/Cargo.toml `[package.metadata.cargo-machete] ignored = ["gpui","gpui-kit"]` | High | Medium | [D2] New runtime state must be hand-written types in core/component (already compiled, already deps); generated glue may reference only `::gpui_form::…` paths. Keep a lint/test that generated output references no new extern crates. |
| R15 | Semver: additive-by-default is true for new generated types/methods (FormPath precedent), but three specific traps: (a) new bounds on existing `FormState`/`FieldPath` methods = breaking; (b) adding `Eq`/`Hash` derives to holders would break forms containing f64/`Decimal` fields (PartialEq deliberately without Eq today); (c) new inherent methods can silently shadow a user's extension-trait method (inherent wins — behavior swap, not compile error). | value_holder.rs:507-513 (PartialEq-not-Eq rationale); workspace Cargo.toml:26 (0.5.2); crates/gpui-form-core/src/path.rs:117-156 (published impls) | Low-Med | Medium | 0.6.0 for anything touching existing signatures [D9]; never add derives to holders beyond the current set; prefix new holder methods to avoid generic names (collision bait is low but document). |
| R16 | Nested forms (#2) vs koruma: koruma has `nested`/`newtype` support, but the holder would store `<Address>FormValueHolder` — conversions Address↔holder must compose two generated conversion lattices, `wrap_in_option`+`default_expr` semantics (currently defined per flat field: `None` means "equals default", value_holder.rs:223-266) get recursive and ambiguous (is an absent nested-holder "unchanged" or "cleared"?). Error access becomes `errors.address()` chains. | value_holder.rs:223-266, `:276-327` (default/None round-trip logic); Components enum has no Nested variant at crates/gpui-form-codegen/src/components.rs:318-331 | High | High | Prototype the nested *holder + path + error bridge* end-to-end on one example before designing attribute syntax; resolve absent-vs-default semantics explicitly; budget a docs pass (AGENTS.md sync rules make this 6+ surfaces per feature). |
| R17 | Repeated fields (#3) are the largest hidden-cost item: per-item `Entity<InputState>` means entity churn on add/remove (subscriptions must be rebuilt), touched/dirty keys need per-item paths (R9), koruma `each` validators exist but per-item error access is untyped-vec-shaped, and drag-reorder is explicitly widget work outside scope. Backlog itself says "conservative first" — plans tend to ignore that. | docs/feature-backlog.md:75-98; element_validators plumbing at value_holder.rs:440-460; entity holder at crates/gpui-form-derive/src/derives/gpui_form/expansion.rs:39, `:362-374` | High | High | Ship add/remove + scalar items only; forbid reorder and nested item forms in the first cut; keep per-item entity management app-authored (the derive emits metadata + holder ops, not subscriptions) [D5]. |
| R18 | Validation-mode (onChange/onBlur) config surface: modes interact with R1 (blur not uniform across kinds), R2 (per-render cost), and intermediate values the input layer deliberately permits ("`-`", `"0."` — phone/number empty-is-accepted precedent). Defaulting to onChange will flag half-typed fields. | gpui-base blur clamp comment state.rs:3388-3391; empty-accepted precedent in CHANGELOG 0.5.2 ("Empty input is accepted so a half-typed field is not flagged") | Medium | Medium | Default onBlur for touched-driven display and onChange only for cache invalidation (not error display); mirror the "empty is not an error while typing" rule. |
| R19 | Scope creep from backlog parity items: async_select provider (#5), field dependency (#12), conditional requiredness (#13), wizard steps (#11) each drift toward a rule/DSL/data layer that AGENTS.md + the backlog's own scope boundary exclude (derive-driven generation, not widget/rule library). Conditional requiredness via stringly `required_if = "subscribe"` (backlog line 288-294) contradicts the typed-path anti-goal and koruma's per-field validator model. | docs/feature-backlog.md:7-23 (scope boundary), `:134-152`, `:249-296`; AGENTS.md Workspace Map | Medium | Medium | Each must pass the test: "metadata + typed state only, no execution engine." Conditional requiredness should desugar to a generated koruma validator closure, not a gpui-form expression language. |
| R20 | Proc-macro expansion growth: derive already emits holder + conversion error type + From/try_from + `<Name>FormPath` + `<Name>FormFields` + `<Name>FormComponents` + optional inventory + i18n consts + `localized_validation_issue` (needs `recursion_limit = "1024"` in the demo, main.rs:1). Every feature adds per-field tokens; the derive depends on syn(full)+darling+koruma-derive-core, so parse cost is fixed but *codegen* cost scales with features. | crates/gpui-form-derive/src/derives/gpui_form/expansion.rs:356-374 (single quote! block); value_holder.rs:683-714, `:761-778`; examples/forms-demo/src/main.rs:1 | Medium | Low-Med | Keep emissions small: reference hand-written core/component APIs [D2]; gate luxury emissions (i18n consts, fluent) behind existing attribute/feature switches; add a token-count smoke test (expand a 10-field form, assert expansion size stays within a bound). |

### Deliberately NOT building

1. **`watch()` / React-style reactive subscriptions on the holder.** GPUI has
   explicit notify + entity observation; a watch layer duplicates
   `cx.subscribe_in` and hides re-render cost. Evidence: handlers are already
   the integration point (main.rs:152-161). Cost: a second invalidation
   system.
2. **`shouldUnregister` / mount-unmount field lifecycle.** There is no field
   mount/unmount: `<Name>FormFields` entities are created eagerly and owned
   by the app view (expansion.rs:363; main.rs:141-149). Nothing to hook; the
   concept translates to nothing.
3. **A `Controller`-style per-field rerender abstraction.** GPUI re-renders
   are entity-granular and coalesced; per-field render control is a React
   reconciliation concept.
4. **Error maps keyed by `String`.** Regresses the exact problem FieldPath
   was built to solve (path.rs:1-26); koruma errors are typed structs —
   bridge them, don't re-key them (R7).
5. **Touched/dirty/error/submit state inside the holder or its serde format.**
   Breaks round-trip semantics and literal construction (R11). Holder stays
   plain data — this is also backlog #7's own stated rule ("the holder should
   remain plain data", feature-backlog.md:180-182).
6. **Generic dynamic-segment `FieldPath` (arbitrary `String` segments).**
   Either typed `Index` segments or a dedicated list-item path type (R9/D8);
   free-form strings reintroduce ad-hoc naming.
7. **Drag-reorder for repeated fields (and nested item forms in v1).** Widget
   work the scope boundary excludes (feature-backlog.md:95-98 already draws
   the line; keep it).
8. **A cross-field rule/dependency engine** (`depends_on`, `required_if` as
   expressions, cross-field phone enforcement as a generic system). Metadata
   + generated koruma validators only (R19); a rule engine is a new product,
   not a form feature.
9. **Generated subscriptions/render wiring from the derive.** The derive
   emits state + metadata; apps wire events. Generating
   `cx.subscribe_in`/handlers would explode the API surface, bake re-entrancy
   traps (R5/R6) into generated code, and violate the headless-first layering
   (feature-backlog.md:11-22). This is the single most tempting "parity with
   react-hook-form" move and the most wrong for this architecture [D5].
10. **Partial holder (#10) as a separate holder type, for now.** It overlaps
    three existing mechanisms: the holder already stores non-optional fields
    as `Option<T>` (value_holder.rs:40-47), patch/delta (#9) produces sparse
    output, and the custom-component contract already defines tri-state
    set/clear/unchanged (crates/gpui-form-component/src/custom.rs:43-51).
    Ship #9 (patch) first; revisit #10 only if patch cannot express "clear vs
    unchanged" — which the tri-state enum suggests it can [D10].

**Biggest under-estimated items overall:** R7+R8 (the koruma error model
dictates the per-field-error design; it is struct aggregates, not maps), R9
(repeated fields break the `&'static str` path primitive — decide the 0.6
core change before any nested/list work), and R14/R17 (dependency creep and
entity churn, which are invisible in a 10-field demo).

---

## 6. Consolidated open questions

Carried from the design and codegen perspectives; each needs an owner before
its milestone starts. Items marked UNVERIFIED name what to check.

1. `issue_keys()` gating — unconditional vs `koruma(fluent)`-gated (§2 OQ 1).
   Recommended: unconditional; changes an existing gating boundary.
2. Koruma error-type derives (no `PartialEq`/`serde`) — accepted limitation;
   upstream change out of scope (§2 OQ 2).
3. `Send` bounds on `AsyncFieldValidation`/`SubmitAction` — decide when the
   GPUI glue phase starts (§2 OQ 3).
4. Patch wire format convention (absent-unchanged vs explicit) (§2 OQ 4).
5. Async outcome merge semantics into the error map (§2 OQ 5).
6. `field_index()` linear scan vs generated `match` (§2 OQ 6).
7. Element-error path surface — blocked on M3/D8 (§2 OQ 7).
8. Facade naming: `<Name>FormState` vs `<Name>Form` (§2 OQ 8). Recommended:
   `<Name>FormState`.
9. **UNVERIFIED (R1):** blur parity for select/date-picker/file-picker/
   checkbox event enums — check `SelectEvent`/`DatePickerEvent`/
   `FilePickerEvent` before claiming touched for those kinds.
10. **UNVERIFIED (R12):** gpui-pre `Task` drop-cancellation semantics — check
    `gpui-pre-0.3.7/src/executor.rs` before relying on drop-cancellation in
    the async glue.
11. **UNVERIFIED (R7 follow-up):** whether koruma-core supports struct-level
    validators the derive could mirror for cross-field rules — check before
    M4 designs `required_if`/`depends_on` lowering.
