[![Build Status](https://github.com/stayhydated/gpui-form/actions/workflows/ci.yml/badge.svg)](https://github.com/stayhydated/gpui-form/actions/workflows/ci.yml)
[![Docs](https://docs.rs/gpui-form/badge.svg)](https://docs.rs/gpui-form/)
[![Crates.io](https://img.shields.io/crates/v/gpui-form.svg)](https://crates.io/crates/gpui-form)

# gpui-form

`gpui-form` is a type-safe form-generation ecosystem for `gpui` and
[`gpui-component`](https://github.com/longbridge/gpui-component), centered on
`#[derive(GpuiForm)]`.

It is designed for three things:

1. Compile-time generation of strongly typed form state and helper types.
1. Concise field annotations on normal application structs.
1. Runtime helpers, metadata, and prototyping support around the derive-based
   workflow.

## Compatibility

| `gpui-form` | `gpui-component` | `gpui` |
| :---------- | :--------------- | :----- |
| **git** | | |
| `branch = "master"` | `branch = "main"` | `rev = "832c17e8192e2e1d472f0751e7cef2af84ded622"` |

## Installation

```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed", rev = "832c17e8192e2e1d472f0751e7cef2af84ded622" }
gpui-component = { git = "https://github.com/longbridge/gpui-component", branch = "main" }

gpui-form = "*"

# Optional: inventory registration for prototyping/code generation
# gpui-form = { version = "*", features = ["inventory"] }

# Optional: form-state persistence + dirty tracking (serde on the holder)
# gpui-form = { version = "*", features = ["serde"] }
```

## Quick Start

```rs
use gpui_form::{GpuiForm, SelectItem};
use strum::EnumIter;

#[derive(Clone, Debug, Default, EnumIter, PartialEq, SelectItem)]
pub enum Country {
    #[default]
    UnitedStates,
    France,
    Japan,
}

#[derive(Clone, Debug, Default, GpuiForm)]
pub struct UserProfile {
    #[gpui_form(component(input))]
    pub username: Option<String>,

    #[gpui_form(component(number_input))]
    pub age: Option<u32>,

    #[gpui_form(component(select), default = Country::France)]
    pub country: Country,

    #[gpui_form(component(checkbox))]
    pub subscribe: bool,
}
```

`#[derive(GpuiForm)]` generates the typed form support around that struct:

- `UserProfileFormFields` for GPUI entity state
- `UserProfileFormComponents` constructors for those fields
- `UserProfileFormValueHolder` for typed editing, defaults, validation, and
  conversion back into the original model
- `UserProfileFormPath` typed field paths so validation, dirty tracking,
  focus, analytics, and schema export share ONE typed way to name fields
  (see [Typed Field Paths](#typed-field-paths))

## Component Syntax

These component forms are currently supported:

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

Common field-level helpers:

- `#[gpui_form(default = <expr>)]` seeds the generated value holder and initial
  select-like choices.
- `#[gpui_form(skip)]` excludes a field from generated form widgets while still
  allowing prefill from the original model.
- `#[gpui_form(type = <form_type>, from = <expr>, into = <expr>)]` lets the
  generated form edit a type that differs from the original field type.
- `component(input)` parses non-`String` form-side value types with `FromStr`
  in prototyping output, so value objects can use `type`, `from`, and `into`
  while the source model keeps its storage type.
- Field-level `#[koruma(...)]` attributes are accepted by `GpuiForm` and copied
  onto the generated value holder, including fields that use `type`, `from`,
  and `into` to validate a form-side type.
- `#[gpui_form(section = ..., label = ..., description = ..., placeholder = ..., width = ...)]`
  attaches non-rendering layout hints. See
  [Layout and Section Hints](#layout-and-section-hints).

`component(infinite_select)` expects the field type to implement
`gpui_form::InfiniteSelect`, usually by deriving it on the enum tree. The enum
tree must also implement `PartialEq` because the backing `gpui-component`
select compares selected values.
Lower-level users can derive the same runtime contract from
`gpui-form-component` or `gpui-form-component-derive`; the macro resolves
whichever runtime crate is present, including renamed dependencies.

Common struct-level helpers:

- `#[gpui_form(empty)]` marks an intentional empty form.
- `#[gpui_form(koruma)]` enables Koruma-backed validation wiring.
- `#[gpui_form(koruma(fluent))]` enables Koruma validation plus fluent error
  rendering.

## Layout and Section Hints

Fields can declare non-rendering layout hints that generated and prototyped
forms consume. These are **metadata-only** in v1: they describe intent, they do
not drive any GPUI rendering. Application code and prototyping generators decide
how (or whether) to render each hint.

```rust
#[derive(gpui_form::GpuiForm)]
pub struct AccountSettings {
    #[gpui_form(section = "Account", label = "Username", component(input))]
    pub username: String,

    #[gpui_form(
        label = "Enable experiments",
        description = "Toggles unreleased features",
        component(switch)
    )]
    pub enable_experimental: bool,

    #[gpui_form(placeholder = "you@example.com", width = half, component(input))]
    pub email: String,
}
```

Supported hints (all optional):

- `section = "<str>"` — groups consecutive fields under a named section.
- `label = "<str>"` — preferred human-readable label. Defaults to the field
  name at consumption time when absent.
- `description = "<str>"` — help text / comment hint shown alongside the field.
- `placeholder = "<str>"` — placeholder text for inputs that support one.
- `width = full | half | third` — relative width hint. Accepts the bare ident
  (`width = half`) or a quoted string (`width = "half"`). This is a **hint, not
  a layout engine**: consumers may ignore it or map it onto their own grid.

Section grouping is **order-preserving**: consecutive fields with the same
`section` form one group, and fields are never reordered across the form. This
is the foundation richer layouts (columns, collapsible sections) can build on
later.

Layout hints are ignored on `#[gpui_form(skip)]` fields — skipped fields emit no
form metadata, so their hints never reach the schema. The generated form code
itself is unchanged; the hints ride along on the field metadata that
prototyping and tooling consume.

At runtime the hints are reachable through `gpui_form::schema::FieldVariant` as
a `gpui_form::schema::FieldLayout`, and the width enum is re-exported as
`gpui_form::LayoutWidth` for ergonomic matching. The prototyping generator
groups fields by `section`, prefers `label` over the field name, and emits
`description` where it already produces help text.

## Infinite Select Runtime

`component(infinite_select)` fields are backed by
`gpui_form::infinite_select::InfiniteSelectState`, which owns the root and
child `SelectState`s, exposes render-ready level snapshots, and emits a single
typed change event with the rebuilt nested value, both path forms, the
previous paths, and the changed depth.

```rs
use gpui_form::infinite_select::{InfiniteSelectEvent, InfiniteSelectState};

let location = cx.new(|cx| {
    InfiniteSelectState::new(Country::default(), window, cx)
});

cx.subscribe_in(
    &location,
    window,
    |_, _, event: &InfiniteSelectEvent<Country>, _, _| {
        let _value = event.value();
        let _path = event.path();
        let _key_path = event.key_path();
        let _previous_key_path = event.previous_key_path();
        let _changed_depth = event.changed_depth();
    },
);
```

Rendering code can stay on the runtime helper instead of combining select
handles with separate label lookups:

```rs
for field in location.read(cx).form_fields() {
    let _ = field;
}
```

The derive/runtime pair also exposes typed option labels, stable key paths, and
typed path errors:

- root option titles come from `variant_label()` instead of raw `variant_name()`
- `#[fluent_kv(keys = ["label", "description"], keys_this)]` emits
  `es-fluent` variant and type metadata for application-owned localizers;
  generated runtime labels use plain fallback names because the runtime trait
  contract is localizer-free
- `#[tuple_enum(key = "...")]` overrides persisted keys when enum names should
  stay decoupled from storage
- `selection_key_path()` / `build_from_key_path(...)` round-trip nested values
  without depending on enum ordering
- `InfiniteSelectKeyPath` supports `Display`, `FromStr`, and serde string
  round-trips for URLs and persisted config
- `build_from_path(...)`, `build_from_key_path(...)`, `set_path(...)`, and
  `set_key_path(...)` return `InfiniteSelectPathError` with the failing depth
  plus the invalid key/index segment
- `set_selected_index_at_depth(...)` / `set_selected_key_at_depth(...)` support
  incremental programmatic updates

## Validation With Koruma

`gpui-form` can mirror Koruma validation metadata into the generated value
holder so your form state and your domain model stay aligned.

```rs
use gpui_form::GpuiForm;
use koruma::{Koruma, KorumaAllFluent};
use koruma_collection::{
    collection::NonEmptyValidation,
    numeric::RangeValidation,
};

#[derive(Clone, Debug, GpuiForm, Koruma, KorumaAllFluent)]
#[gpui_form(koruma(fluent))]
pub struct Signup {
    #[gpui_form(component(input))]
    #[koruma(NonEmptyValidation::<_>::builder())]
    pub username: String,

    #[gpui_form(component(number_input))]
    #[koruma(RangeValidation::<_>::builder().min(18).max(120))]
    pub age: Option<u32>,
}
```

When validation is enabled:

- required-value semantics are preserved when the generated holder wraps fields
  in `Option<T>`
- builder-chain Koruma attrs are mirrored
- generated value-holder validation uses the same validator set as the source
  struct

## Saving, Restoring, and Dirty Tracking

Enable the optional `serde` feature to make generated forms saveable/restorable
and to know whether the user edited them. The feature is additive and opt-in.

```toml
gpui-form = { version = "*", features = ["serde"] }
```

With the feature on, `#[derive(GpuiForm)]` adds `Serialize` and `Deserialize`
to the generated `...FormValueHolder`, so the holder round-trips through any
serde format. The holder always derives `PartialEq` (not `Eq`, since field
types like `number_input(as = f64)` are not `Eq`), so `FormState`'s dirty
tracking works on default features too. Separately, the facade re-exports
`gpui_form::FormState` (from `gpui-form-core`) for dirty tracking, reset, and
diffing — pure logic with no GPUI dependency.

```rs
use gpui_form::{FormState, GpuiForm};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, GpuiForm, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    #[gpui_form(component(input))]
    pub username: Option<String>,

    #[gpui_form(component(number_input))]
    pub age: Option<u32>,
}

// Save: serialize the live holder.
let holder = SettingsFormValueHolder::default();
let json = serde_json::to_string(&holder).expect("serialize");

// Restore: deserialize into a fresh FormState.
let restored: SettingsFormValueHolder =
    serde_json::from_str(&json).expect("deserialize");
let mut state = FormState::new(restored);
assert!(!state.is_dirty());

// Edit, then ask whether the user changed anything.
state.current_mut().username = Some("ada".into());
assert!(state.is_dirty());

// Reset to discard edits, or sync after a save to mark clean.
state.reset_to_baseline();
assert!(!state.is_dirty());
```

Scope of this feature:

- `FormState` stores holder **data** only — not component runtime UI state such
  as open menus, scroll positions, or `InfiniteSelectState` snapshots.
- Dirty/diff is **boolean-level**: `is_dirty()` and `diff_against(&other)` tell
  you *whether* the holder changed, not which fields. Field-level diff is
  backlog feature #9 and will build on the [typed field paths](#typed-field-paths)
  foundation shipped as FLAT v1.
- A holder carrying `#[gpui_form(skip)]` fields round-trips through serde on its
  own, but cannot fully reconstruct the source struct via `into_original`, since
  skipped values are not on the holder. This mirrors the existing
  `has_skipped_fields` behavior. Per-field serde passthrough (rename/skip) is
  backlog feature #15.
- No undo/redo in this feature.

`FormState` is available unconditionally from `gpui_form::FormState` (or
`gpui-form-core` directly); only the holder serde derives require the `serde`
feature.

## Typed Field Paths

Every `#[derive(GpuiForm)]` form also emits a `<Name>FormPath` type — a
strongly-typed newtype around the shared headless primitive
`gpui_form::FieldPath` — so every consumer of a form (validation, dirty
tracking, focus, analytics, schema export) can refer to fields through ONE
typed value instead of ad-hoc strings.

```rs
use gpui_form::{FieldPath, GpuiForm};

#[derive(GpuiForm)]
pub struct Settings {
    #[gpui_form(component(input))]
    pub username: String,

    #[gpui_form(component(number_input))]
    pub age: Option<u32>,

    #[gpui_form(skip)]
    pub internal_id: u32,
}

// One constructor per non-skipped field, named identically to the field.
let username = SettingsFormPath::username();
let age = SettingsFormPath::age();
assert_eq!(username.to_string(), "username");
assert_ne!(username, age);

// `Deref`/`AsRef`/`into_path` all reach the shared primitive, so any code
// that takes a `&FieldPath` (the upcoming validation/diff/schema surfaces)
// also accepts the typed newtype.
fn records(path: &FieldPath) -> &[&'static str] {
    path.segments()
}
assert_eq!(records(username.as_ref()), &["username"]);
```

Scope of this feature (FLAT v1):

- Each constructor names a single flat field. Typed nested-path and
  list-item-path constructors arrive with backlog features #2 ("Nested forms")
  and #3 ("Repeated fields"). Hand-built multi-segment paths via
  `SettingsFormPath::new(&["a", "b"])` work today; typed composition is later.
- `#[gpui_form(skip)]` fields have NO constructor — they are absent from the
  holder too.
- `FieldPath` is a headless primitive: no GPUI, no `serde`, no feature flag.
  It is the shared naming foundation for the upcoming field-level validation
  (#6), field-level diff/delta reporting (#9), and schema export (#14).

`FieldPath` is available unconditionally from `gpui_form::FieldPath` (or
`gpui-form-core` directly).

## Custom Components

There are two supported custom-component workflows.

### 1. Derive directly on a state type

```rs
use gpui_form::{CustomComponentState, GpuiForm};

#[derive(Clone, Debug, CustomComponentState)]
#[gpui_form_custom(new = Self::new, component = TagsInput)]
pub struct TagsInputState;

#[derive(Clone, Debug, Default, GpuiForm)]
pub struct PostEditor {
    #[gpui_form(component(custom(state = TagsInputState, wraps_in_option = false)))]
    pub tags: Vec<String>,
}
```

### 2. Declare a reusable shape

```rs
gpui_form::custom_component_shape!(
    pub EmailInputShape,
    state = gpui_component::input::InputState,
    new = gpui_component::input::InputState::new,
    component = gpui_component::input::Input,
);

#[derive(Clone, Debug, Default, gpui_form::GpuiForm)]
pub struct ContactForm {
    #[gpui_form(component(custom(shape = EmailInputShape)))]
    pub email: String,
}
```

Custom components can also opt into generated value synchronization by
implementing `gpui_form::custom::CustomComponentValueAdapter<T>` on the shape
and adding `value_binding` to the custom component options. The adapter remains
application-owned; `gpui-form` only calls its generic seed and event-conversion
hooks.

Runtime helpers are available from both:

- `gpui_form::runtime`
- legacy compatibility re-exports such as `gpui_form::custom`,
  `gpui_form::date_picker`, `gpui_form::file_picker`,
  `gpui_form::infinite_select`, and
  `gpui_form::numeric`

## File Picker Runtime

For generated native path selection, use `#[gpui_form(component(file_picker))]`
on a `PathBuf` field or combine it with `type`, `from`, and `into` when the
model stores something derived from a path.

For manual native path selection, use `gpui_form::file_picker` or
`gpui_form::runtime::file_picker`. The runtime uses GPUI's
`PathPromptOptions` from the pinned Zed git dependency and renders the control
with `gpui-component` buttons, icons, sizing, and theme tokens.
Built-in defaults are plain English fallback copy. When a form needs localized
placeholder, prompt, or button text, render those messages through an
application-owned `es-fluent` localizer and pass the resulting strings through
`placeholder(...)`, `prompt(...)`, and `browse_label(...)`. The runtime ships
Fluent resources for callers that localize those messages explicitly.

```rs
use gpui_form::file_picker::{FilePicker, FilePickerEvent, FilePickerState};

let picker = cx.new(|cx| FilePickerState::new(window, cx));

cx.subscribe_in(&picker, window, |_, _, event: &FilePickerEvent, _, _| {
    if let FilePickerEvent::Change(paths) = event {
        let _paths = paths;
    }
});

FilePicker::new(&picker)
    .placeholder("Choose a file")
    .prompt("Choose a file")
    .cleanable(true);
```

## Date Conversion

`date_picker` fields can edit a different form-side type than the original model
field by combining `type`, `from`, and `into`:

```rs
#[derive(Clone, Debug, gpui_form::GpuiForm)]
pub struct User {
    #[gpui_form(
        type = chrono::NaiveDate,
        from = to_form_date,
        into = to_model_timestamp,
        component(date_picker)
    )]
    pub birth_date: Option<Timestamp>,
}
```

This pattern is useful when the model stores a domain-specific timestamp type
but the UI should edit a calendar date.
The default empty placeholder is plain English fallback copy; pass
`DatePicker::placeholder(...)` when a form needs localized or custom copy. The
selected-date label and calendar popover use ICU4X for localized month names,
weekday headers, day/year labels, and locale-specific week starts. Manual
runtime code can use `DateRangePicker` and `DateRangePickerState` for range
selection; generated `component(date_picker)` fields remain single-date fields.

## Prototyping

Enable the `inventory` feature when you want to generate scaffolding from
registered `GpuiFormShape` metadata:

```rs
use gpui_form::schema::registry::{GpuiFormShape, inventory};
use gpui_form_prototyping_core::FormShapeAdapter;

for shape in inventory::iter::<GpuiFormShape>() {
    let parts = FormShapeAdapter::new(shape)
        .parts()
        .expect("shape metadata should be valid");

    let _imports = parts.imports;
}
```

See [`examples/prototyping`](examples/prototyping) for a complete generator
that reads shape inventory, clears stale generated form modules, and writes
scaffolded GPUI form files. The Storybook form titles generated by that example
use the GPUI app context so they follow the active Storybook locale.

## Examples

[`examples/README.md`](examples/README.md) is the canonical index for runnable
workspace examples.

- `cargo run -p some-lib-forms`: browse generated forms in a storybook-style UI
- `cargo run -p gpui-form-component-story`: browse runtime component demos
- `cargo run -p prototyping`: generate scaffolded GPUI form files from shape
  inventory
