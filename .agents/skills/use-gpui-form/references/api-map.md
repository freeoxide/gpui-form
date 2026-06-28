# gpui-form User API Map

Use this reference for application code that consumes `gpui-form`.

## Install Shape

Use `gpui-form` as the public entry point. Match `gpui` and `gpui-component`
versions to the compatibility guidance for the `gpui-form` version in use.

```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed", rev = "832c17e8192e2e1d472f0751e7cef2af84ded622" }
gpui-component = { git = "https://github.com/longbridge/gpui-component", branch = "main" }
gpui-form = "*"
```

Optional feature flags (additive):

```toml
# inventory registration for prototyping/code generation
# gpui-form = { version = "*", features = ["inventory"] }

# form-state persistence + dirty tracking (serde + PartialEq on the holder)
# gpui-form = { version = "*", features = ["serde"] }
```

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
- `gpui_form::state` (pure form-state module from `gpui-form-core`)
- `gpui_form::FormState` (dirty tracking / reset / diff helper)
- `gpui_form::custom_component_shape!`

## Supported Component Syntax

```rust
#[gpui_form(component(input))]
#[gpui_form(component(number_input))]
#[gpui_form(component(number_input(as = f64)))]
#[gpui_form(component(checkbox))]
#[gpui_form(component(switch))]
#[gpui_form(component(select))]
#[gpui_form(component(select(searchable)))]
#[gpui_form(component(select(partial)))]
#[gpui_form(component(infinite_select))]
#[gpui_form(component(infinite_select(searchable, max_depth = 3)))]
#[gpui_form(component(date_picker))]
#[gpui_form(component(file_picker))]
#[gpui_form(component(custom(shape = my::Shape)))]
#[gpui_form(component(custom(state = my::State)))]
#[gpui_form(component(custom(shape = my::Shape, component = my::ui::Widget)))]
#[gpui_form(component(custom(shape = my::Shape, wraps_in_option = false)))]
```

Common field attributes:

```rust
#[gpui_form(default = <expr>)]
#[gpui_form(skip)]
#[gpui_form(type = <form_type>)]
#[gpui_form(from = <expr>)]
#[gpui_form(into = <expr>)]
```

Common struct attributes:

```rust
#[gpui_form(empty)]
```

## Component Selection

- Use `input` for text-like fields.
- Use `number_input` for numeric fields; use `number_input(as = f64)` when the
  field editor should parse through a different numeric representation.
- Use `checkbox` or `switch` for `bool` fields.
- Use `select` for a single enum-like choice; derive `SelectItem`.
- Use `select(searchable)` when the option set should be searchable.
- Use `select(partial)` when partial selection semantics are needed.
- Use `infinite_select` for nested/cascading enum trees; derive
  `InfiniteSelect`.
- Use `date_picker` for single-date editing.
- Use `file_picker` for native path selection.
- Use `custom(...)` when the app owns the state/widget contract.

## Generated Names

For a source struct named `UserProfile`, expect generated types named:

```rust
UserProfileFormFields
UserProfileFormComponents
UserProfileFormValueHolder
```

Use the generated value holder for editable form data, defaults, and conversion
back into the original model.

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

Add `#[select_item(fluent)]` when the enum derives `EsFluent` and the app will
handle localized labels outside the `SelectItem::title()` call.

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

Helper state is available from `gpui_form::infinite_select`.

## Type Conversion Pattern

Use `type`, `from`, and `into` when the UI edits a different type than the
model stores:

```rust
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

This is useful for dates, paths, numeric newtypes, and other domain-specific
wrappers.

## Custom Component Patterns

Derive directly on a state type:

```rust
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

Or declare a reusable shape:

```rust
gpui_form::custom_component_shape!(
    pub EmailInputShape,
    state = gpui_component::input::InputState,
    new = gpui_component::input::InputState::new,
    component = gpui_component::input::Input,
);
```

## Form-State Persistence and Dirty Tracking Pattern

Enable the `serde` feature to make generated holders saveable/restorable and to
use `gpui_form::FormState` for dirty tracking. The feature adds `Serialize`,
`Deserialize`, and `PartialEq` to the generated `...FormValueHolder`. `FormState`
itself is available unconditionally; only the holder serde derives need the
feature.

```rust
use gpui_form::{FormState, GpuiForm};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, GpuiForm, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    #[gpui_form(component(input))]
    pub username: Option<String>,
}

// Save.
let json = serde_json::to_string(&SettingsFormValueHolder::default()).unwrap();

// Restore into a fresh state and track edits.
let restored: SettingsFormValueHolder = serde_json::from_str(&json).unwrap();
let mut state = FormState::new(restored);
state.current_mut().username = Some("ada".into());
assert!(state.is_dirty());

// Reset to discard edits, or sync after a save to mark clean.
state.reset_to_baseline();
state.sync_baseline();
```

Scope: `FormState` stores holder data only (no runtime UI state), dirty/diff is
boolean-level (field-level diff is backlog #9), a holder with `#[gpui_form(skip)]`
fields round-trips through serde but cannot fully reconstruct the source struct
(per-field serde passthrough is backlog #15), and there is no undo/redo.
