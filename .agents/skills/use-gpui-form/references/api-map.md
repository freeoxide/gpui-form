# gpui-form User API Map

Use this reference for application code that consumes `gpui-form`. It maps the
public API surface: facade imports, component/attribute inventory, generated
names, and pattern skeletons. Install snippets and full worked examples live in
the repository root `README.md` (single source) — do not transcribe them here.

## Compatibility

| `gpui-form` | `gpui-kit` | `gpui` |
| :---------- | :--------- | :----- |
| **git** | | |
| `branch = "master"` | `0.7.0` | `0.3.7` (`gpui-pre`) |

Install from git: `gpui-form = { git = "https://github.com/stayhydated/gpui-form" }`.
The crates.io `gpui-form` package is the upstream `freeoxide/gpui-form`
lineage, not this fork. Optional additive features: `inventory`, `serde`,
`phone` (see the facade README feature list or root `README.md`
§Installation).

## Facade Imports

Prefer imports from `gpui_form`:

```rust
use gpui_form::{CustomComponentState, GpuiForm, InfiniteSelect, SelectItem};
```

Useful facade paths:

- `gpui_form::custom`
- `gpui_form::date_picker`
- `gpui_form::file_picker`
- `gpui_form::infinite_select`
- `gpui_form::numeric`
- `gpui_form::path` (pure field-path module from `gpui-form-core`)
- `gpui_form::phone` (parser-backed phone validation helpers; `phone` feature)
- `gpui_form::state` (pure form-state module from `gpui-form-core`)
- `gpui_form::i18n` (rust-i18n locale bridge: `init`, `change_locale`,
  `localize_message`; built-in runtime copy ships en/fr-FR/zh-CN)
- `gpui_form::FieldPath` (typed field-path primitive; no feature flag)
- `gpui_form::FormFieldErrors` (generic per-form error-map contract; koruma forms)
- `gpui_form::FormState` (dirty tracking / reset / diff helper)
- `gpui_form::TouchedFields` (touched-field bitset composed by the generated
  `<Name>FormState` facade)
- `gpui_form::ValidationCache` (snapshot-compare validation-outcome cache
  composed by koruma-gated facades)
- `gpui_form::SubmitTracker`, `gpui_form::SubmitAction`,
  `gpui_form::submit` (headless submit lifecycle; `BoxError` default error)
- `gpui_form::AsyncFieldValidation` (executor-free async per-field
  validation contract; no `Send` bound)
- `gpui_form::custom_component_shape!`

## Supported Component Syntax

Components (`component(...)`):

- `input`
- `number_input`, `number_input(as = f64)`
- `phone_input`, `phone_input(country = <field>)` — `phone` feature
- `checkbox`, `switch`
- `select`, `select(searchable)`, `select(partial)`
- `infinite_select`, `infinite_select(searchable, max_depth = 3)`
- `date_picker`
- `file_picker`
- `custom(shape = my::Shape)`
- `custom(state = my::State)`
- `custom(shape = my::Shape, component = my::ui::Widget)`
- `custom(shape = my::Shape, wraps_in_option = false)`
- `custom(shape = my::Shape, value_binding)`
- `nested` — compose another `#[derive(GpuiForm)]` struct (milestone M3)
- `list(item = input)`, `list(item = <scalar kind>, add_label = "Add")` —
  editable `Vec<T>` (milestone M3; scalar item kinds only in v1:
  `input`, `number_input`, `phone_input`, `checkbox`, `switch`, `select`,
  `infinite_select`, `date_picker`, `file_picker`)

Field attributes: `default = <expr>`, `skip`, `type = <form_type>`,
`from = <expr>`, `into = <expr>`, the layout hints `section = "<str>"`,
`label = "<str>"`, `description = "<str>"`, `placeholder = "<str>"`,
`width = full | half | third`, `validate(async_fn = <path>)`
(metadata-only today: the path is recorded on `FieldVariant`, nothing
executes until the reserved `async-validation` wiring lands),
`depends_on = <field>` (sibling dependency metadata, milestone M4), and
`required_if = <field>` (conditional requiredness, milestone M4 — desugars
to a generated koruma validator, so violations surface through
`error_for`/`issue_keys` with the shared `validation.required` key). Both
dependency attributes name a sibling field of the same struct
(existence-checked at derive time) and require a `component(...)` on the same
field.

Struct attributes: `empty`, `koruma`, `koruma(fluent)`, `submit` (bare
word; emits the `<Name>FormSubmission` tracker alias).

Layout hints are metadata-only: they attach a `gpui_form::schema::FieldLayout`
to each field for generators/prototyping to consume and do not change
generated form rendering. `label` defaults to the field name at consumption
time; `width` is a hint, not a layout engine (enum re-exported as
`gpui_form::LayoutWidth`). Details: root `README.md` §Component Syntax and
§Layout and Section Hints.

## Component Selection

- Use `input` for text-like fields.
- Use `number_input` for numeric fields; `number_input(as = f64)` when the
  field editor should parse through a different numeric representation.
- Use `checkbox` or `switch` for `bool` fields.
- Use `select` for a single enum-like choice; derive `SelectItem`.
  `select(searchable)` for searchable option sets, `select(partial)` for
  partial-selection semantics.
- Use `infinite_select` for nested/cascading enum trees; derive
  `InfiniteSelect`.
- Use `date_picker` for single-date editing, `file_picker` for native path
  selection.
- Use `custom(...)` when the app owns the state/widget contract.

For country-aware phone inputs, enable the `phone` feature and use the shared
helpers in `gpui_form::phone` instead of duplicating parser and
selected-country checks in every UI: `validate_phone_number`,
`validate_phone_number_for_country_label`,
`validate_optional_phone_number` / `validate_required_phone_number` (plus
`_for_country_label` variants), `validate_phone_number_for`, and the
`PhoneCountry` trait. Details: root `README.md` §Phone Number Validation.

## Generated Names

For a source struct named `UserProfile`, expect generated types named:

```rust
UserProfileFormFields
UserProfileFormComponents
UserProfileFormValueHolder
UserProfileFormPath
UserProfileFormState
UserProfileFormPatch
UserProfileFormFieldErrors   // koruma forms only
UserProfileFormSubmission    // #[gpui_form(submit)] only
```

Use the generated value holder for editable form data, defaults, and conversion
back into the original model. Use `UserProfileFormPath` (one same-named
constructor per non-skipped flat field, `<field>_root()` + `fn <field>(self)`
appenders for nested fields, `<field>_root()` + `fn <field>(index: usize)`
for list fields, plus `join` for cross-form composition,
`FIELD_NAMES`/`FIELD_COUNT`/`all_paths()`/`field_index()`/`path_for_index()`)
for typed field naming across validation, dirty tracking, focus, analytics,
and schema export. Segments are the typed `gpui_form::PathSegment` enum
(`Name(&'static str)` / `Index(usize)`) since milestone M3 — list-item
indices are stored typed, never formatted into strings. Use
`UserProfileFormState` for field-level state (touched tracking, per-field
dirty, `patch()`); it composes `gpui_form::FormState` + `gpui_form::TouchedFields`
and leaves the holder plain data. Koruma forms also emit
`UserProfileFormFieldErrors` + `validate_fields()`/`error_for(&path)` for
path-keyed typed error access, and their facades gain the cached-validation
methods (`validate`/`validation`/`is_valid`/`invalidate_validation`).
Milestone M3 adds: `<field>_errors()` recursion accessors on nested fields
(re-key into the nested form's own error map), `<field>_element_error(index)`
on list fields with `each(...)` validators, `add_<field>` /
`remove_<field>` / `<field>_items` holder operations on list fields, and the
additive `index: Option<usize>` slot on `<Name>FormValueHolderConversionError`.
`#[gpui_form(submit)]` adds `UserProfileFormSubmission`, an alias over
`gpui_form::SubmitTracker` keyed on the koruma error type (`BoxError`
without koruma).

## Select Pattern

```rust
use gpui_form::SelectItem;
use strum::EnumIter;

#[derive(Clone, Debug, Default, EnumIter, PartialEq, SelectItem)]
pub enum Country {
    #[default]
    UnitedStates,
    France,
}
```

Add `#[select_item(fluent)]` when the enum does not implement `Display`:
`title()` then resolves `<enum_snake>.<variant_snake>` through the app's
`rust-i18n` locales and falls back to the variant name on a miss.

## Infinite Select Pattern

```rust
use gpui_form::{GpuiForm, InfiniteSelect};
use strum::EnumIter;

#[derive(Clone, Debug, Default, EnumIter, InfiniteSelect, PartialEq)]
pub enum City {
    #[default]
    Paris,
    Lyon,
}

#[derive(Clone, Debug, EnumIter, InfiniteSelect, PartialEq)]
pub enum Country {
    France(City),
}

#[derive(Clone, Debug, Default, GpuiForm)]
pub struct LocationForm {
    #[gpui_form(component(infinite_select))]
    pub location: Country,
}
```

The enum tree must implement `PartialEq`. Helper state is available from
`gpui_form::infinite_select`. Runtime details (events, key paths, path
errors): root `README.md` §Infinite Select Runtime.

## Type Conversion Pattern

Use `type`, `from`, and `into` when the UI edits a different type than the
model stores — dates, paths, numeric newtypes, and other domain-specific
wrappers. Skeleton:

```rust
#[gpui_form(
    type = chrono::NaiveDate,
    from = to_form_date,
    into = to_model_timestamp,
    component(date_picker)
)]
pub birth_date: Option<Timestamp>,
```

Full examples: root `README.md` §Date Conversion and §Component Syntax.

## Custom Component Patterns

Two workflows: derive `CustomComponentState` directly on a state type
(`#[gpui_form_custom(new = ..., component = ...)]`, consumed via
`component(custom(state = ..., wraps_in_option = false))`), or declare a
reusable shape and consume it via `component(custom(shape = ...))`:

```rust
gpui_form::custom_component_shape!(
    pub EmailInputShape,
    state = gpui_kit::component::input::InputState,
    new = gpui_kit::component::input::InputState::new,
    component = gpui_kit::component::input::Input,
);
```

Value-bound custom widgets implement
`gpui_form::custom::CustomComponentValueAdapter<T>` on the shape and add
`value_binding`. Full examples: root `README.md` §Custom Components.

## Form-State Persistence and Dirty Tracking Pattern

Enable the `serde` feature to make generated holders saveable/restorable and
to use `gpui_form::FormState` for dirty tracking. The feature adds
`Serialize`, `Deserialize`, and `PartialEq` to the generated
`...FormValueHolder` (and the PATCH tri-state serde form of
`<Name>FormPatch`). `FormState` itself is available unconditionally; only
the holder serde derives need the feature. Scope: holder data only (no
runtime UI state), boolean-level dirty/diff on `FormState` itself, holders
with `#[gpui_form(skip)]` fields round-trip through serde but cannot fully
reconstruct the source struct (per-field serde passthrough is backlog #15),
and there is no undo/redo. For field-level state — touched tracking,
`is_field_dirty(path)`, `changed_fields()`, `patch()` — use the generated
`<Name>FormState` facade, which composes `FormState` +
`gpui_form::TouchedFields`; call `touch(&path)` from app-authored blur/change
handlers. Worked example: root `README.md` §Saving, Restoring, and Dirty
Tracking.

## Typed Field Paths Pattern

Every `#[derive(GpuiForm)]` form emits a `<Name>FormPath` newtype around the
shared headless primitive `gpui_form::FieldPath` (no feature flag, no GPUI, no
serde), so every consumer of a form (validation, dirty tracking, focus,
analytics, schema export) can refer to fields through ONE typed value instead
of ad-hoc strings. One constructor per non-skipped field, named identically to
the field; `#[gpui_form(skip)]` fields have no constructor. `Deref`/`AsRef`/
`into_path` reach the shared primitive, and the same type carries
`FIELD_NAMES`/`FIELD_COUNT`/`all_paths()`/`field_index()`/`path_for_index()`
for declaration-order iteration and dense index mapping. Hand-built
multi-segment paths via `<Name>FormPath::new(&["a", "b"])` work today; typed
nested/list composition arrives with backlog #2 ("Nested forms") and #3
("Repeated fields"). Koruma forms pair the paths with typed error access:
`holder.validate_fields()` → `errors.error_for(&path)` →
`field_error.issue_keys()` (`validation.*` keys; inner newtype/nested
failures report `validation.invalid`). Worked example: root `README.md`
§Typed Field Paths and §Validation With Koruma.

## Submit Lifecycle and Live Validation Pattern

Opt in per form, drive the futures yourself (the derive emits no
subscriptions), and keep the holder plain data:

```rust
#[derive(GpuiForm, koruma::Koruma)]
#[gpui_form(submit)]
pub struct Signup {
    #[gpui_form(component(input))]
    #[koruma(koruma_collection::collection::NonEmptyValidation::<_>::builder())]
    pub username: String,
}

// Live validity: refresh the cache from change handlers, read it in render.
let mut state = SignupFormState::new(SignupFormValueHolder::default());
state.validate();
let valid = state.is_valid();

// Submit: gate, begin, drive, finish; success re-syncs the baseline.
let mut submission: SignupFormSubmission = gpui_form::SubmitTracker::new();
if state.is_valid() && submission.begin() {
    let result = save(&state.current()).await;
    submission.finish(result.map(|_: ()| ()));
    if submission.is_submit_successful() {
        state.sync_baseline();
    }
}
```

Behavior notes: `begin()` is the re-entry guard (false while in flight) and
the only thing that counts an attempt; stale completions (after `reset()`,
or post-`abandon()`) are ignored via the generation guard; `set_error`/
`clear_error` inject server-side errors without rewriting terminal
verdicts. Async checks implement `gpui_form::AsyncFieldValidation<Holder>`
(one impl per form, `Ok(())` for unhandled fields); debounce and
latest-wins cancellation are app-owned. Worked example: root `README.md`
§Submit Lifecycle and Live Validation.

## Field Dependencies and Conditional Requiredness Pattern

Milestone M4: name sibling fields (never condition strings), and reset
dependents yourself from the parent's change handler.

```rust
#[derive(GpuiForm)]
pub struct Signup {
    #[gpui_form(component(select(searchable)))]
    pub country: Option<Country>,

    // Metadata: consumers read `depends_on` for reset-on-parent-change.
    #[gpui_form(component(select), depends_on = country)]
    pub state: Option<UsState>,

    #[gpui_form(component(switch))]
    pub subscribe: bool,

    // Enforced: desugars to a koruma validator on the holder field.
    #[gpui_form(component(input), required_if = subscribe)]
    pub email: Option<String>,
}

// Typed, path-addressable conditional failure — no display-only metadata.
let errors = holder.validate_fields();
let error = errors.error_for(&SignupFormPath::email()).unwrap();
assert_eq!(error.issue_keys(), vec!["validation.required"]);

// Reset-on-parent-change is app-authored ([D5]): clear holder AND widget.
// self.form.current_mut().state = None;
// state_select.update(cx, |s, cx| s.set_selected_index(None, window, cx));

// "Save only touched fields": filter the patch by the touched set (the
// patch tri-state already expresses cleared-vs-unchanged — plan decision
// D10; no separate partial-holder type).
let touched: HashSet<String> = state
    .touched_fields()
    .iter()
    .map(|path| path.to_string())
    .collect();
let mut patch = state.patch();
if !touched.contains("state") {
    patch.state = None;
}
```

Constraints: the `required_if` target's holder field stores `Option<T>`;
the controlling sibling holds `bool` (required when `true`), `Option<bool>`
(required when `Some(true)`), or another `Option<T>` (required when set); a
`required_if` field never also gets the synthetic unconditional
`RequiredValidation`. The pure predicate behind the desugaring is
`gpui_form::required_when`. Worked example: root `README.md` §Field
Dependencies and Conditional Requiredness.
