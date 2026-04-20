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

Most application code should start with [`gpui-form`](crates/gpui-form/README.md).

## Compatibility

| `gpui-form` | `gpui-component` | `gpui` |
| :---------- | :--------------- | :----- |
| **crates.io** | | |
| `0.5.1` | `0.5.1` | `0.2.2` |
| **git** | | |
| `branch = "master"` | `branch = "main"` | `rev = "15d8660748b508b3525d3403e5d172f1a557bfa5"` |

## Installation

```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed", rev = "15d8660748b508b3525d3403e5d172f1a557bfa5" }
gpui-component = { git = "https://github.com/longbridge/gpui-component", branch = "main" }

gpui-form = { version = "*", features = ["derive"] }

# Optional: inventory registration for prototyping/code generation
# gpui-form = { version = "*", features = ["derive", "inventory"] }
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

Common struct-level helpers:

- `#[gpui_form(empty)]` marks an intentional empty form.
- `#[gpui_form(koruma)]` enables Koruma-backed validation wiring.
- `#[gpui_form(koruma(fluent))]` enables Koruma validation plus fluent error
  rendering.

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

See [`examples/prototyping`](examples/prototyping) for a complete generator that
walks inventory, produces `syn::File`, formats it with `prettyplease`, and
writes scaffolded GPUI form files.

## Examples

[`examples/README.md`](examples/README.md) is the canonical index for runnable
workspace examples.

- `cargo run -p some-lib-forms`: browse generated forms in a storybook-style UI
- `cargo run -p prototyping`: generate scaffolded GPUI form files from shape
  inventory
