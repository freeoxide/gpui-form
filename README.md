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
| `branch = "master"` | `branch = "main"` | `rev = "f7d46cf7d02c88d3d71ec495a31d7f19bd5eb96b"` |

## Installation

```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed", rev = "f7d46cf7d02c88d3d71ec495a31d7f19bd5eb96b" }
gpui-component = { git = "https://github.com/longbridge/gpui-component", branch = "main" }

gpui-form = "*"

# Optional: inventory registration for prototyping/code generation
# gpui-form = { version = "*", features = ["inventory"] }
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
- `#[gpui_form(component(custom(shape = my::Shape)))]`
- `#[gpui_form(component(custom(state = my::State)))]`
- `#[gpui_form(component(custom(shape = my::Shape, component = my::ui::Widget)))]`
- `#[gpui_form(component(custom(shape = my::Shape, wraps_in_option = false)))]`

Common field-level helpers:

- `#[gpui_form(default = <expr>)]` seeds the generated value holder and initial
  select-like choices.
- `#[gpui_form(skip)]` excludes a field from generated form widgets while still
  allowing prefill from the original model.
- `#[gpui_form(type = <form_type>, from = <expr>, into = <expr>)]` lets the
  generated form edit a type that differs from the original field type.

`component(infinite_select)` expects the field type to implement
`gpui_form::InfiniteSelect`, usually by deriving it on the enum tree.
Lower-level users can derive the same runtime contract from
`gpui-form-component` or `gpui-form-component-derive`; the macro resolves
whichever runtime crate is present, including renamed dependencies.

Common struct-level helpers:

- `#[gpui_form(empty)]` marks an intentional empty form.
- `#[gpui_form(koruma)]` enables Koruma-backed validation wiring.
- `#[gpui_form(koruma(fluent))]` enables Koruma validation plus fluent error
  rendering.

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
    #[koruma(NonEmptyValidation<_>)]
    pub username: String,

    #[gpui_form(component(number_input))]
    #[koruma(RangeValidation<_>(min = 18, max = 120))]
    pub age: Option<u32>,
}
```

When validation is enabled:

- required-value semantics are preserved when the generated holder wraps fields
  in `Option<T>`
- shorthand Koruma attrs and builder-chain attrs are both mirrored
- generated value-holder validation uses the same validator set as the source
  struct

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

Runtime helpers are available from both:

- `gpui_form::runtime`
- legacy compatibility re-exports such as `gpui_form::custom`,
  `gpui_form::date_picker`, `gpui_form::infinite_select`, and
  `gpui_form::numeric`

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
that reads shape inventory and writes scaffolded GPUI form files.

## Examples

[`examples/README.md`](examples/README.md) is the canonical index for runnable
workspace examples.

- `cargo run -p some-lib-forms`: browse generated forms in a storybook-style UI
- `cargo run -p prototyping`: generate scaffolded GPUI form files from shape
  inventory
