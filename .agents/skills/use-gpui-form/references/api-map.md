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
`phone`, `mcp` (see the facade README feature list or root `README.md`
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
- `gpui_form::FormState` (dirty tracking / reset / diff helper)
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

Field attributes: `default = <expr>`, `skip`, `type = <form_type>`,
`from = <expr>`, `into = <expr>`, and the layout hints `section = "<str>"`,
`label = "<str>"`, `description = "<str>"`, `placeholder = "<str>"`,
`width = full | half | third`.

Struct attributes: `empty`, `koruma`, `koruma(fluent)`.

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
```

Use the generated value holder for editable form data, defaults, and conversion
back into the original model. Use `UserProfileFormPath` (one same-named
constructor per non-skipped field) for typed field naming across validation,
dirty tracking, focus, analytics, and schema export.

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
`...FormValueHolder`. `FormState` itself is available unconditionally; only
the holder serde derives need the feature. Scope: holder data only (no
runtime UI state), boolean-level dirty/diff (field-level diff is backlog #9,
built on the typed field paths below), holders with `#[gpui_form(skip)]`
fields round-trip through serde but cannot fully reconstruct the source struct
(per-field serde passthrough is backlog #15), and there is no undo/redo.
Worked example: root `README.md` §Saving, Restoring, and Dirty Tracking.

## Typed Field Paths Pattern

Every `#[derive(GpuiForm)]` form emits a `<Name>FormPath` newtype around the
shared headless primitive `gpui_form::FieldPath` (no feature flag, no GPUI, no
serde), so every consumer of a form (validation, dirty tracking, focus,
analytics, schema export) can refer to fields through ONE typed value instead
of ad-hoc strings. One constructor per non-skipped field, named identically to
the field; `#[gpui_form(skip)]` fields have no constructor. `Deref`/`AsRef`/
`into_path` reach the shared primitive. Hand-built multi-segment paths via
`<Name>FormPath::new(&["a", "b"])` work today; typed nested/list composition
arrives with backlog #2 ("Nested forms") and #3 ("Repeated fields").
`FieldPath` is the shared naming foundation for upcoming field-level
validation (#6), field-level diff/delta reporting (#9), and schema export
(#14). Worked example: root `README.md` §Typed Field Paths.
