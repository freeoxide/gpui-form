# gpui-form-core

UI-neutral helper logic for the `gpui-form` ecosystem.

Most application code should use [`gpui-form`](../gpui-form/README.md) instead.
Reach for this crate directly only when you want the helper logic without the
GPUI runtime layer.

## What It Provides

Today this crate is intentionally small and focused:

- `numeric::validate_signed_numeric`
- `numeric::validate_unsigned_numeric`
- `phone::validate_phone_number_for_country_label` behind the optional
  `phone` feature
- `FormState<H>` for dirty tracking, reset, and diffing of form holder values
- `path::FieldPath` for typed field naming (a headless, GPUI-free, serde-free
  primitive)

The numeric helpers match the text-entry rules used by `gpui-form` number
inputs. `FormState` is the pure, GPUI-free side of form-state persistence and
dirty tracking (feature #1); it is re-exported by the facade as
`gpui_form::FormState`.

The phone helpers wrap the `phonenumber` parser and add selected-country
matching. This avoids repeating the same boilerplate in every UI that has a
country select plus phone input.

```toml
gpui-form-core = { version = "*", features = ["phone"] }
```

```rs
use gpui_form_core::phone::{
    country,
    validate_phone_number_for_country_label,
};

let result = validate_phone_number_for_country_label(
    "+1 415 550 2222",
    country::FR,
    "France",
);

assert!(!result.is_valid());
```

## FormState

`FormState<H>` wraps any cloneable value (typically a generated
`...FormValueHolder`) and snapshots a *baseline* copy at construction time so
the caller can answer "did the user edit this form?" without keeping a second
copy by hand.

```rs
use gpui_form_core::FormState;

#[derive(Clone, Debug, PartialEq, Default)]
struct Holder { name: String }

let mut state = FormState::new(Holder { name: "a".into() });
assert!(!state.is_dirty());

state.current_mut().name = "b".into();
assert!(state.is_dirty());

state.reset_to_baseline();   // discard edits
assert!(!state.is_dirty());

state.current_mut().name = "c".into();
state.sync_baseline();       // mark clean after a save
assert!(!state.is_dirty());
```

Construction and the mutating helpers (`reset_to_baseline`, `sync_baseline`)
require `H: Clone`. `is_dirty()` and `diff_against(&other)` require
`H: PartialEq`. Dirty/diff is boolean-level — *whether* the value changed, not
which fields (field-level diff is backlog feature #9, building on the
[`FieldPath`](#fieldpath) foundation). `FormState` stores holder data only,
never component runtime UI state; it carries no GPUI dependency and no new
crate dependencies.

## FieldPath

`FieldPath` is a headless primitive: an ordered sequence of static string
segments naming a form field, so that every consumer of a form (validation,
dirty tracking, focus, analytics, schema export) can share ONE typed way to
refer to fields instead of ad-hoc strings. It is the shared naming foundation
for the upcoming field-level validation (#6), field-level diff/delta reporting
(#9), schema export (#14), and nested/list paths (#2/#3).

The derive crate wraps this primitive per form as `<Name>FormPath`, reachable
through the facade as `gpui_form::FieldPath`; the raw primitive is also
available directly as `gpui_form_core::FieldPath`.

```rs
use gpui_form_core::FieldPath;
use std::collections::HashSet;

let city = FieldPath::new(&["address", "city"]);
let name = FieldPath::new(&["name"]);

assert_eq!(city.to_string(), "address.city");   // Display joins with "."
assert_eq!(name.to_string(), "name");
assert_ne!(city, name);

// Equality and hashing are by segment sequence.
let mut seen = HashSet::new();
seen.insert(city.clone());
assert!(seen.contains(&FieldPath::new(&["address", "city"])));
assert!(!seen.contains(&name));
```

Scope of this primitive (FLAT v1): a path is a list of static segments —
typically a single field name. Typed nested-path and list-item-path
constructors arrive with backlog features #2 ("Nested forms") and #3
("Repeated fields"); hand-built multi-segment paths via `FieldPath::new(&["a",
"b"])` work today, typed composition is later. `FieldPath` is unconditional (no
feature flag), carries no GPUI dependency, and has no `serde` dependency of its
own.

## Example

```rs
use gpui_form_core::numeric::{
    validate_signed_numeric,
    validate_unsigned_numeric,
};

assert!(validate_signed_numeric::<i32>("-42", true));
assert!(validate_unsigned_numeric::<u32>("42", true));

assert!(!validate_signed_numeric::<i32>("01", true));
assert!(!validate_unsigned_numeric::<u32>("-1", true));
```

## When To Use This Crate Directly

- You are building your own numeric input wrapper and want the same text-entry
  rules as `gpui-form`
- You want the numeric or phone validation helpers without pulling in `gpui` or
  `gpui-component`
- You want `FormState` dirty/reset/diff logic without the GPUI runtime layer
  (the facade re-exports it as `gpui_form::FormState` for convenience)
- You want the `FieldPath` naming primitive for analytics, focus tracking, or
  a custom validation layer (the facade re-exports it as
  `gpui_form::FieldPath`)

## Most Users Should Use Instead

- [`gpui-form`](../gpui-form/README.md) for normal application development
- [`gpui-form-component`](../gpui-form-component/README.md) when you need the
  GPUI runtime helpers
