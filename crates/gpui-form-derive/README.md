# gpui-form-derive

Procedural macros behind the `gpui-form` ecosystem.

Most users should depend on [`gpui-form`](../gpui-form/README.md) and derive
from the facade crate. Use this crate directly when you want the proc-macro
layer without the facade's runtime and metadata re-exports.

## What This Crate Provides

- `#[derive(GpuiForm)]`
- `#[derive(SelectItem)]`
- `#[derive(CustomComponentState)]`

`#[derive(InfiniteSelect)]` does not live in this crate. It is provided by
[`gpui-form-component-derive`](../gpui-form-component-derive/README.md) and
re-exported by the facade as `gpui_form::InfiniteSelect`.

## `#[derive(GpuiForm)]`

Turns a struct into typed form state plus helper types for editing and
submission.

```rs
use gpui_form::GpuiForm;

#[derive(Clone, Debug, Default, GpuiForm)]
pub struct UserProfile {
    #[gpui_form(component(input))]
    pub username: Option<String>,

    #[gpui_form(component(number_input))]
    pub age: Option<u32>,
}
```

Supported component forms:

- `#[gpui_form(component(input))]`
- `#[gpui_form(component(number_input))]`
- `#[gpui_form(component(number_input(as = f64)))]`
- `#[gpui_form(component(phone_input))]` (requires the `gpui-form` `phone` feature)
- `#[gpui_form(component(phone_input(country = <field>)))]` (requires the `gpui-form` `phone` feature)
- `#[gpui_form(component(checkbox))]`
- `#[gpui_form(component(switch))]`
- `#[gpui_form(component(select))]`
- `#[gpui_form(component(select(searchable)))]`
- `#[gpui_form(component(select(partial)))]`
- `#[gpui_form(component(infinite_select))]`
- `#[gpui_form(component(infinite_select(searchable, max_depth = 3)))]`
- `#[gpui_form(component(date_picker))]`
- `#[gpui_form(component(file_picker))]`
- `#[gpui_form(component(custom(shape = my::Shape)))]`
- `#[gpui_form(component(custom(state = my::State)))]`
- `#[gpui_form(component(custom(shape = my::Shape, component = my::ui::Widget)))]`
- `#[gpui_form(component(custom(shape = my::Shape, wraps_in_option = false)))]`
- `#[gpui_form(component(custom(shape = my::Shape, value_binding)))]`
- `#[gpui_form(component(nested))]` — compose another `#[derive(GpuiForm)]`
  struct (milestone M3; the holder stores the nested form's
  `<Name>FormValueHolder` and converts through its `From` impls)
- `#[gpui_form(component(list(item = input)))]` — an editable `Vec<T>` with a
  scalar item component (milestone M3; optional `add_label = "..."` overrides
  the add-control label). v1 rejects `item = nested` and `item = custom(...)`

Supporting field attributes:

- `#[gpui_form(default = <expr>)]`
- `#[gpui_form(skip)]`
- `#[gpui_form(type = <form_type>)]`
- `#[gpui_form(from = <expr>)]`
- `#[gpui_form(into = <expr>)]`
- `#[gpui_form(section = "<str>")]` — non-rendering section grouping hint
- `#[gpui_form(label = "<str>")]` — preferred display label (defaults to the
  field name at consumption time when absent)
- `#[gpui_form(description = "<str>")]` — help text / comment hint
- `#[gpui_form(placeholder = "<str>")]` — placeholder text for inputs
- `#[gpui_form(width = full | half | third)]` — relative width hint; accepts a
  bare ident or a quoted string
- `#[gpui_form(depends_on = <field>)]` — records a sibling-field dependency as
  schema metadata (reset-on-parent-change consumers read it; the derive emits
  no event wiring)
- `#[gpui_form(required_if = <field>)]` — conditional requiredness; desugars to
  a generated koruma validator (`<Name>FormRequiredIfValidation`) so the
  violation surfaces through `validate_fields()`/`error_for` with the shared
  `validation.required` key. The sibling reference is existence-checked at
  derive time and must hold a `bool` or `Option` value; the target's holder
  field must store `Option<T>`

Supporting struct attributes:

- `#[gpui_form(empty)]`
- `#[gpui_form(koruma)]`
- `#[gpui_form(koruma(fluent))]`

Behavior notes:

- `select` expects enum-like values that can populate a `gpui_kit` select
- `component(infinite_select)` expects the field type to implement
  `gpui_form::InfiniteSelect`
- `component(file_picker)` stores a selected `PathBuf` in the generated value
  holder and can be paired with `type`/`from`/`into` for model fields that store
  bytes or other path-derived data
- `default = ...` also seeds the initial selection for `select` and
  `infinite_select`
- `custom(..., wraps_in_option = false)` keeps the generated value-holder field
  as `T` instead of `Option<T>`
- `custom(..., value_binding)` records that the custom shape implements
  `gpui_form::custom::CustomComponentValueAdapter<T>` for generated
  prototyping subscriptions
- `type`/`from`/`into` let the generated holder edit a type that differs from
  the original model field
- `component(input)` prototyping code parses form-side non-`String` values with
  `FromStr` instead of assigning raw `String`s
- field-level `#[koruma(...)]` attributes are accepted by `GpuiForm` and copied
  onto the generated value holder, which allows validating form-side override
  types without deriving `Koruma` on the original model
- when skipped fields are present, the generated value holder keeps builder
  support and exposes `into_original(...)` instead of an unconditional reverse
  conversion
- the derive also emits a `<Name>FormPath` type — a strongly-typed newtype
  around `gpui_form::core::FieldPath` — with one same-named constructor per
  NON-skipped field (`SettingsFormPath::username()`). Skipped fields have NO
  constructor (mirroring the holder). `<Name>FormPath` carries no generics,
  reaches the shared primitive via `Deref`/`AsRef`/`into_path`, and also
  exposes the declaration-order tables `FIELD_NAMES` / `FIELD_COUNT` /
  `all_paths()` / `field_index()` / `path_for_index()` that field-level state
  keys on (dense indices exist only for FLAT v1 single-segment paths naming a
  real field; hand-built multi-segment or unknown paths return `None`). It is
  the typed naming foundation for the field-level validation (#6) and
  field-level diff (#9) surfaces below, and for the upcoming schema export
  (#14) and nested/list paths (#2/#3). FLAT v1: each
  constructor names a single field; typed nested/list composition arrives with
  #2/#3 (hand-built multi-segment paths via `new(&["a","b"])` work today).
  `FieldPath` is unconditional — the path type is emitted without any feature
  flag
- every form also emits a `<Name>FormState` facade and a `<Name>FormPatch`
  type (field-level state model): the facade composes
  `gpui_form::core::FormState` + `gpui_form::core::TouchedFields` behind
  typed path accessors (`touch`, `is_touched`, `touched_fields`,
  `is_field_dirty`, `changed_fields`, `patch`, plus reset semantics where
  `reset_to_baseline()` clears touched while
  `reset_to_baseline_keeping_touched()` and `sync_baseline()` keep it), and
  the patch holds one `Option<holder-field-type>` slot per non-skipped field
  (`between`/`is_empty`/`changed_fields`/`apply_to`; with the derive crate's
  `serde` feature it also serializes as the PATCH tri-state). Holders stay
  plain data — all state lives on the facade. See the root README
  §Saving, Restoring, and Dirty Tracking
- koruma-enabled forms (any route: struct-level `#[gpui_form(koruma)]` or
  field-level `#[koruma(...)]` attrs) additionally gain typed error access:
  `holder.validate_fields()` returns a `<Name>FormFieldErrors` map re-keying
  koruma's typed per-field error structs by `<Name>FormPath`, with
  `error_for(&path)`, per-field `issue_keys()` (`validation.*` keys), typed
  accessors, and the `gpui_form::core::FormFieldErrors` trait for generic
  code. See the root README §Validation With Koruma
- `section`, `label`, `description`, `placeholder`, and `width` are
  **metadata-only** layout hints. They do not change generated form rendering;
  they attach a `FieldLayout` to each emitted `FieldVariant` so downstream
  tooling (e.g. `gpui-form-prototyping-core`) can consume them. `label`
  defaults to the field name at consumption time when absent. `width` is a
  hint, not a layout engine. Hints on `#[gpui_form(skip)]` fields are ignored
  because no `FieldVariant` is emitted for skipped fields. Section grouping is
  order-preserving (consecutive same-section fields).

### `koruma(fluent)` localization

The attribute grammar is unchanged. Under `koruma(fluent)` the generated holder
derives `KorumaAllDisplay` and adds:

- `const <FORM>_I18N_KEY_PREFIX: &str = "<snake_form>"`
- `const <FIELD>_LABEL_KEY: &str = "<snake_form>.<field>_label"` per non-skipped
  field
- `fn validation_issue_key(kind: &str) -> &'static str` mapping validator kinds
  to `validation.<snake>` keys (`RequiredValidation` -> `validation.required`,
  with `validation.invalid` as the fallback arm)
- `fn localized_validation_issue(kind: &str) -> Cow<'static, str>` resolving
  through `rust_i18n::t!`

The emitted `t!` resolves against the CONSUMER crate's `rust-i18n` backend:
apps using `koruma(fluent)` must depend on `rust-i18n`, call
`rust_i18n::i18n!("locales", fallback = "en")` at their crate root, and own the
`validation.*` and `<form>.<field>_label` keys. This crate's
`locales/{en,fr-FR,zh-CN}.yml` ship the canonical `validation.*` strings as the
template.

## `#[derive(SelectItem)]`

Implements `::gpui_kit::component::select::SelectItem` for enums.

```rs
use gpui_form::SelectItem;

#[derive(Clone, Debug, SelectItem)]
pub enum Country {
    USA,
    France,
}
```

Optional attribute:

- `#[select_item(fluent)]` allows enums that do not implement `Display`.
  `title()` resolves `"<enum_snake>.<variant_snake>"` (e.g.
  `preferred_language.english`) through `rust_i18n::t!` and falls back to the
  variant name when the key is missing. The consuming crate must depend on
  `rust-i18n`, call `rust_i18n::i18n!` at its crate root, and own those keys.

## `#[derive(CustomComponentState)]`

Implements `gpui_form::custom::CustomComponentShape` directly for a state type.

```rs
use gpui_form::CustomComponentState;

#[derive(Clone, Debug, CustomComponentState)]
#[gpui_form_custom(
    new = crate::state::build,
    component = crate::ui::TagsInput
)]
pub struct TagsState;
```

By default, the generated implementation calls `Self::new(window, cx)`.

## Feature Flags

- `inventory`: enables `GpuiFormShape` registration for `#[derive(GpuiForm)]`
- `serde`: adds `Serialize`, `Deserialize`, and `PartialEq` to the generated
  `...FormValueHolder` so it round-trips through any serde format and can be
  compared for dirty tracking. `PartialEq` (not `Eq`) is emitted deliberately,
  because `number_input(as = f64)` and similar non-`Eq` field types would
  otherwise fail to compile. Most users enable this through the facade's
  `serde` feature rather than this crate directly.

When the `serde` feature is on, the holder becomes suitable for form-state
persistence and dirty tracking via `gpui_form::FormState`:

```rs
use gpui_form::{FormState, GpuiForm};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, GpuiForm, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    #[gpui_form(component(input))]
    pub username: Option<String>,
}

let holder = SettingsFormValueHolder::default();
let json = serde_json::to_string(&holder).expect("serialize");
let restored: SettingsFormValueHolder =
    serde_json::from_str(&json).expect("deserialize");

let mut state = FormState::new(restored);
state.current_mut().username = Some("ada".into());
assert!(state.is_dirty());
```

Scope notes: the holder with `#[gpui_form(skip)]` fields round-trips through
serde on its own, but cannot fully reconstruct the source struct via
`into_original` (skipped values are absent from the holder). Per-field serde
passthrough (rename/skip) is backlog feature #15.

## Most Users Should Use Instead

- [`gpui-form`](../gpui-form/README.md) for the main facade
- [`gpui-form-component-derive`](../gpui-form-component-derive/README.md) for
  `#[derive(InfiniteSelect)]`
- [`gpui-form-schema`](../gpui-form-schema/README.md) when you need metadata
  rather than derives
