# gpui-form-core

UI-neutral helper logic for the `gpui-form` ecosystem.

Most application code should use [`gpui-form`](../gpui-form/README.md) instead.
Reach for this crate directly only when you want the helper logic without the
GPUI runtime layer.

## What It Provides

Today this crate is intentionally small and focused:

- `numeric::validate_signed_numeric`
- `numeric::validate_unsigned_numeric`
- `FormState<H>` for dirty tracking, reset, and diffing of form holder values

The numeric helpers match the text-entry rules used by `gpui-form` number
inputs. `FormState` is the pure, GPUI-free side of form-state persistence and
dirty tracking (feature #1); it is re-exported by the facade as
`gpui_form::FormState`.

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
which fields (field-level diff is backlog feature #9). `FormState` stores holder
data only, never component runtime UI state; it carries no GPUI dependency and
no new crate dependencies.

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
- You want the validation helpers without pulling in `gpui` or
  `gpui-component`
- You want `FormState` dirty/reset/diff logic without the GPUI runtime layer
  (the facade re-exports it as `gpui_form::FormState` for convenience)

## Most Users Should Use Instead

- [`gpui-form`](../gpui-form/README.md) for normal application development
- [`gpui-form-component`](../gpui-form-component/README.md) when you need the
  GPUI runtime helpers
