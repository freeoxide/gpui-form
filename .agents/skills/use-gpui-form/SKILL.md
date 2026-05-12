---
name: use-gpui-form
description: "Use when Codex needs to build user-facing application forms with gpui-form, including adding #[derive(GpuiForm)] to app structs, choosing gpui_form component attributes, using generated form fields/components/value holders, wiring SelectItem or InfiniteSelect enums, and adding custom components."
---

# use-gpui-form

## Overview

Use this skill to build application forms with `gpui-form`. Treat
`gpui_form` as the normal public entry point and keep work focused on consumer
code: app models, form state, rendering, and app-owned components.

## First Moves

1. Identify the application struct or enum that should drive the form.
2. Check the app's existing `Cargo.toml` and form code for local patterns.
3. Use the facade crate in application code:

   ```rust
   use gpui_form::{GpuiForm, SelectItem};
   ```

4. Load `references/api-map.md` when you need exact component syntax or a
   compact pattern reference.

## Application Form Workflow

Add `GpuiForm` to a normal Rust struct and annotate each visible field with a
component:

```rust
use gpui_form::{GpuiForm, SelectItem};
use strum::EnumIter;

#[derive(Clone, Debug, Default, EnumIter, PartialEq, SelectItem)]
pub enum Country {
    #[default]
    UnitedStates,
    France,
}

#[derive(Clone, Debug, Default, GpuiForm)]
pub struct UserProfile {
    #[gpui_form(component(input))]
    pub username: Option<String>,

    #[gpui_form(component(number_input))]
    pub age: Option<u32>,

    #[gpui_form(component(select), default = Country::France)]
    pub country: Country,
}
```

The derive generates types named from the source struct:

- `UserProfileFormFields`
- `UserProfileFormComponents`
- `UserProfileFormValueHolder`

Use `#[gpui_form(default = ...)]` for initial form values, `#[gpui_form(skip)]`
for model fields that should not render as widgets, and
`#[gpui_form(type = ..., from = ..., into = ..., component(...))]` when the UI
edits a form-side type that differs from the model field.

## Common Patterns

- For selects, derive `SelectItem` on enum-like values and `EnumIter` when the
  app needs iteration-backed choices.
- For cascading/nested selects, derive `InfiniteSelect` on the enum tree and
  use `#[gpui_form(component(infinite_select))]`.
- For custom widgets, derive `CustomComponentState` on a state type or declare
  a reusable shape with `gpui_form::custom_component_shape!`.
- For helper state and facade compatibility modules, use paths such as
  `gpui_form::date_picker`, `gpui_form::file_picker`, and
  `gpui_form::infinite_select`.

## References

- `references/api-map.md`: installation shape, supported component syntax, and
  user-facing usage patterns.
