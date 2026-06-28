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

Supporting struct attributes:

- `#[gpui_form(empty)]`
- `#[gpui_form(koruma)]`
- `#[gpui_form(koruma(fluent))]`

Behavior notes:

- `select` expects enum-like values that can populate a `gpui_component` select
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
  reaches the shared primitive via `Deref`/`AsRef`/`into_path`, and is the
  typed naming foundation for future field-level validation (#6), field-level
  diff (#9), schema export (#14), and nested/list paths (#2/#3). FLAT v1: each
  constructor names a single field; typed nested/list composition arrives with
  #2/#3 (hand-built multi-segment paths via `new(&["a","b"])` work today).
  `FieldPath` is unconditional — the path type is emitted without any feature
  flag
- `section`, `label`, `description`, `placeholder`, and `width` are
  **metadata-only** layout hints. They do not change generated form rendering;
  they attach a `FieldLayout` to each emitted `FieldVariant` so downstream
  tooling (e.g. `gpui-form-prototyping-core`) can consume them. `label`
  defaults to the field name at consumption time when absent. `width` is a
  hint, not a layout engine. Hints on `#[gpui_form(skip)]` fields are ignored
  because no `FieldVariant` is emitted for skipped fields. Section grouping is
  order-preserving (consecutive same-section fields).

## `#[derive(SelectItem)]`

Implements `gpui_component::select::SelectItem` for enums.

```rs
use gpui_form::SelectItem;

#[derive(Clone, Debug, SelectItem)]
pub enum Country {
    USA,
    France,
}
```

Optional attribute:

- `#[select_item(fluent)]` allows enums that derive `EsFluent` to avoid a
  `Display` bound, but `SelectItem::title()` has no localizer argument. Render
  localized select labels in the application layer when localization is needed.

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
