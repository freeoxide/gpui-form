---
name: use-gpui-form
description: "Use when Codex needs to build user-facing application forms with gpui-form, including adding #[derive(GpuiForm)] to app structs, choosing gpui_form component attributes, using generated form fields/components/value holders, wiring SelectItem or InfiniteSelect enums, and adding custom components."
---

# Use GPUI Form

## Scope Boundary

Treat this skill as a hosted public-usage guide for `gpui-form` consumers. Use
it only for user-facing application workflows: deriving forms on app models,
choosing component attributes, using generated form state, wiring select and
infinite-select enums, and adding app-owned custom components.

Do not use this skill as a contributor guide for `gpui-form` repository
internals. For build, test, format, lint, maintenance, release, or architecture
work, read the repository source, `AGENTS.md`, and the relevant crate
documentation directly.

## Core Workflow

Start from the user-facing facade. Most application code uses `gpui-form` for
derives, runtime helpers, and compatibility re-exports:

1. Identify the application struct or enum that should drive the form.
2. Check the app's existing `Cargo.toml` and form code for local patterns.
3. Use the facade crate in application code:

   ```rust
   use gpui_form::{GpuiForm, SelectItem};
   ```

4. Add `GpuiForm` to a normal Rust struct and annotate each visible field with
   a component.
5. Use generated types named from the source struct, such as
   `UserProfileFormFields`, `UserProfileFormComponents`,
   `UserProfileFormValueHolder`, and `UserProfileFormPath` (typed field paths,
   see [Typed Field Paths](#typed-field-paths)).
6. Use `#[gpui_form(default = ...)]` for initial form values,
   `#[gpui_form(skip)]` for model fields that should not render as widgets, and
   `#[gpui_form(type = ..., from = ..., into = ..., component(...))]` when the
   UI edits a form-side type that differs from the model field. Text input
   prototyping parses non-`String` form-side types with `FromStr`.
7. Use paths such as `gpui_form::date_picker`, `gpui_form::file_picker`, and
   `gpui_form::infinite_select` for helper state and facade compatibility modules.

## Reference Selection

Load only the reference needed for the task:

- `references/api-map.md`: installation shape, supported component syntax, and user-facing usage patterns.

Prefer current public docs or source examples over memory when details matter.

## Implementation Rules

Derive application forms from app-owned data types:

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

Common patterns:

- For selects, derive `SelectItem` on enum-like values and `EnumIter` when the app needs iteration-backed choices.
- For cascading or nested selects, derive `InfiniteSelect` and `PartialEq` on the enum tree and use `#[gpui_form(component(infinite_select))]`.
- For custom widgets, derive `CustomComponentState` on a state type or declare a reusable shape with `gpui_form::custom_component_shape!`.
- For value-bound custom widgets, implement `gpui_form::custom::CustomComponentValueAdapter<T>` on the shape and use `component(custom(shape = ..., value_binding))`.
- For save/restore and dirty tracking, enable the facade `serde` feature and wrap the holder in `gpui_form::FormState`.
- For typed field naming (validation, dirty tracking, focus, analytics, schema export), use the generated `<Name>FormPath` constructors such as `UserProfileFormPath::username()`; skipped fields have no constructor.
- Keep consumer code focused on app models, form state, rendering, and app-owned components.

## Saving, Restoring, and Dirty Tracking

When the app needs to persist a form or ask whether the user edited it, enable
the optional `serde` feature and use `gpui_form::FormState`. The feature is
additive: it adds `Serialize`, `Deserialize`, and `PartialEq` to the generated
`...FormValueHolder`, and the facade re-exports `FormState` (pure, GPUI-free
logic from `gpui-form-core`).

```toml
gpui-form = { version = "*", features = ["serde"] }
```

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

// Restore into a fresh state.
let restored: SettingsFormValueHolder = serde_json::from_str(&json).unwrap();
let mut state = FormState::new(restored);

// Track edits, reset, or mark clean after a save.
state.current_mut().username = Some("ada".into());
assert!(state.is_dirty());
state.sync_baseline();   // mark clean
assert!(!state.is_dirty());
```

Scope notes to keep in mind when recommending this feature:

- `FormState` stores holder **data** only, not runtime UI state (open menus,
  scroll, `InfiniteSelectState` snapshots). No undo/redo in this feature.
- Dirty/diff is **boolean-level** (`is_dirty()`, `diff_against(&other)`).
  Field-level diff is backlog feature #9 and will build on the typed field
  paths below.
- A holder with `#[gpui_form(skip)]` fields round-trips through serde on its
  own, but cannot fully reconstruct the source struct via `into_original`.
  Per-field serde passthrough (rename/skip) is backlog feature #15.
- `FormState` itself is available unconditionally from `gpui_form::FormState`;
  only the holder serde derives need the `serde` feature.

## Typed Field Paths

Every `#[derive(GpuiForm)]` form also emits a `<Name>FormPath` type — a
strongly-typed newtype around the shared headless primitive
`gpui_form::FieldPath` — so validation, dirty tracking, focus, analytics, and
schema export can refer to fields through ONE typed value instead of ad-hoc
strings. `FieldPath` is re-exported unconditionally (no feature flag, no GPUI,
no serde).

```rust
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
assert_eq!(username.to_string(), "username");

// `Deref`/`AsRef`/`into_path` all reach the shared primitive.
fn records(p: &FieldPath) -> &[&'static str] { p.segments() }
assert_eq!(records(username.as_ref()), &["username"]);
```

Scope notes for this feature (FLAT v1):

- Each constructor names a single flat field. Typed nested-path and
  list-item-path constructors arrive with backlog features #2 ("Nested forms")
  and #3 ("Repeated fields"). Hand-built multi-segment paths via
  `SettingsFormPath::new(&["a", "b"])` work today; typed composition is later.
- `#[gpui_form(skip)]` fields have NO constructor — they are absent from the
  holder too.
- `FieldPath` is the shared naming foundation for the upcoming field-level
  validation (#6), field-level diff/delta reporting (#9), and schema export
  (#14).
