# gpui-form-core

UI-neutral helper logic for the `gpui-form` ecosystem.

Most application code should use [`gpui-form`](../gpui-form/README.md) instead.
Reach for this crate directly only when you want the helper logic without the
GPUI runtime layer.

## What It Provides

Today this crate is intentionally small and focused:

- `numeric::validate_signed_numeric`
- `numeric::validate_unsigned_numeric`
- `phone::validate_phone_number` and
  `phone::validate_phone_number_for_country_label` behind the optional `phone`
  feature, plus `validate_optional_phone_number` / `validate_required_phone_number`
  (and `_for_country_label` variants) for explicit empty-handling, the
  `validate_phone_number_for` helper, and the `PhoneCountry` mapping trait
- `FormState<H>` for dirty tracking, reset, and diffing of form holder values
- `TouchedFields` for touched-field tracking (a dense, form-agnostic bitset
  the generated `<Name>FormState` facade sizes from `<Name>FormPath::FIELD_COUNT`)
- `FormFieldErrors`, the generic contract the generated
  `<Name>FormFieldErrors` maps implement for path-keyed error access
- `ValidationCache<H, O>` for snapshot-compare validation-outcome caching
  (the live `is_valid` primitive the koruma-gated facade composes)
- `required_when(requires, present)`, the pure conditional-required
  predicate every generated `required_if` koruma validator delegates to
  (the derive owns the desugaring; core owns no rule engine)
- `SubmitTracker<E>` + `SubmitAction<H>` for the headless submit lifecycle
  (in-flight guard, attempt count, last error, generation-guarded completions)
- `AsyncFieldValidation<H>`, the executor-free async per-field validation
  contract (`async fn` in a trait, path-keyed, no `Send` bound)
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
gpui-form-core = { git = "https://github.com/stayhydated/gpui-form", features = ["phone"] }
```

```rs
use gpui_form_core::phone::{
    country,
    validate_phone_number,
    validate_phone_number_for_country_label,
};

let general = validate_phone_number("+1 415 550 2222", Some(country::FR));
assert!(general.is_valid());

let result = validate_phone_number_for_country_label(
    "+1 415 550 2222",
    country::FR,
    "France",
);

assert!(!result.is_valid());
```

Apps that want explicit empty semantics can use
`validate_optional_phone_number` (empty input is acceptable) or
`validate_required_phone_number` (empty input is rejected with
`PhoneNumberValidationError::Required`). Implement `PhoneCountry` on an
application enum to map it to a `country::Id` and label once, then validate with
`validate_phone_number_for`:

```rs
use gpui_form_core::phone::{country, validate_phone_number_for, PhoneCountry};

enum Region {
    UnitedStates,
}

impl PhoneCountry for Region {
    fn phone_country_id(&self) -> country::Id {
        match self {
            Self::UnitedStates => country::US,
        }
    }

    fn phone_country_label(&self) -> String {
        "United States".to_string()
    }
}

assert!(validate_phone_number_for("415 555 2671", &Region::UnitedStates).is_valid());
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
`H: PartialEq`. Dirty/diff on `FormState` itself is boolean-level — *whether*
the value changed, not which fields (the field-level diff, touched tracking,
and per-form facade live in the derive's generated `<Name>FormState`, which
composes this type). `FormState` stores holder data only, never component
runtime UI state; it carries no GPUI dependency and no new crate dependencies.

## TouchedFields

`TouchedFields` is a dense bitset of touched/visited fields keyed by
*declaration-order index*. It is form-agnostic on purpose: the generated
`<Name>FormState` facade supplies the path-to-index mapping (via
`<Name>FormPath::field_index`) and sizes the bitset from
`<Name>FormPath::FIELD_COUNT`. Touched state records user engagement (a field
was blurred or edited), not value deltas — it is independent of `FormState`'s
baseline/current model and is never serialized.

```rs
use gpui_form_core::TouchedFields;

let mut touched = TouchedFields::new(3);
touched.touch_index(1);
assert!(touched.is_touched_index(1));
assert_eq!(touched.touched_indices().collect::<Vec<_>>(), vec![1]);

// Unknown indices are a no-op, never a panic.
touched.touch_index(99);
assert!(!touched.is_touched_index(99));
```

The facade re-exports it as `gpui_form::TouchedFields`.

## FormFieldErrors

`FormFieldErrors` is the form-agnostic contract implemented by the derive's
generated `<Name>FormFieldErrors` maps (one per koruma form), so GPUI glue,
tests, and application code can be generic over any form's errors through the
shared `FieldPath` primitive:

```rs
use gpui_form_core::FormFieldErrors;

fn failing_names<E: FormFieldErrors>(errors: &E) -> Vec<String> {
    errors
        .failing_paths()
        .iter()
        .map(|path| path.to_string())
        .collect()
}
```

The concrete map — koruma's typed per-field error structs re-keyed by typed
paths, with `error_for(path)` lookup and per-field `issue_keys()` — is
generated by `gpui-form-derive`; see the root README §Validation With Koruma.

## ValidationCache

`ValidationCache<H, O>` is the outcome cache behind a live `is_valid`. It
stores the holder snapshot that was validated next to the outcome, and a
read is fresh **iff `current == snapshot`** — so it stays correct under every
mutation route (including `&mut` escapes) with zero invalidation discipline:

```rs
use gpui_form_core::ValidationCache;

let mut cache: ValidationCache<MyHolder, bool> = ValidationCache::new();
let snapshot = my_holder.clone();
cache.store(snapshot.clone(), my_holder_is_valid(&snapshot));

assert_eq!(cache.get(&snapshot), Some(&true));  // fresh while equal
// ...the caller edits the holder...
assert_eq!(cache.get(&edited), None);           // stale — recompute
cache.invalidate();                             // manual reset-style escape
```

The koruma-gated generated `<Name>FormState` facade composes it (the
`validation` slot plus `validate`/`validation`/`is_valid`/
`invalidate_validation`/`error_for`); see the root README §Submit Lifecycle
and Live Validation. `stored()` is the exact-read companion that skips the
comparison (a holder containing NaN never compares equal to its own clone).

## SubmitTracker and SubmitAction

`SubmitTracker<E>` is the headless submit lifecycle (backlog #7): in-flight
state, attempt count, last error, and success verdicts — plus the
generation/epoch guard that ignores completions arriving after a `reset()`
or with no attempt in flight:

```rs
use gpui_form_core::submit::SubmitTracker;

let mut tracker: SubmitTracker<MyError> = SubmitTracker::new();
if tracker.begin() {                       // false while a submit is in flight
    let result = do_save().await;          // your code drives the future
    tracker.finish(result.map(|_| ()));    // Ok clears / Err records last_error
}
assert!(tracker.can_submit());
tracker.set_error(server_error);           // manual server-error injection
```

`SubmitAction<H>` is the matching executor-free hook contract (an async fn
in a trait, no `Send` bound). Neither runs futures: driving them (GPUI
`cx.spawn`, tokio) is application or glue code. The derive exposes the
tracker per form through the opt-in `#[gpui_form(submit)]` attribute as
`<Name>FormSubmission`; `submit::BoxError` is the default error type for
forms without koruma validation.

## AsyncFieldValidation

`AsyncFieldValidation<H>` is the executor-free async per-field validation
contract: `async fn validate_field(&self, holder: &H, field: &FieldPath) ->
Result<(), Self::Error>`, with no `Send` bound (GPUI futures are typically
non-Send). One impl can serve a whole form by returning `Ok(())` for fields
it does not handle. Deliberately absent: clocks, debounce timers, and
cancellation tokens — the driving layer (GPUI glue or the app) owns them.
See the root README §Submit Lifecycle and Live Validation.

## FieldPath

`FieldPath` is a headless primitive: an ordered sequence of static string
segments naming a form field, so that every consumer of a form (validation,
dirty tracking, focus, analytics, schema export) can share ONE typed way to
refer to fields instead of ad-hoc strings. It is the shared naming foundation
for the field-level validation (#6) and field-level diff/delta reporting (#9)
surfaces shipped on the generated `<Name>FormState` facade, and for the
upcoming schema export (#14) and nested/list paths (#2/#3).

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
  `gpui-kit`
- You want `FormState` dirty/reset/diff logic without the GPUI runtime layer
  (the facade re-exports it as `gpui_form::FormState` for convenience)
- You want the `FieldPath` naming primitive for analytics, focus tracking, or
  a custom validation layer (the facade re-exports it as
  `gpui_form::FieldPath`)
- You want the M2 validation/submit primitives for non-derived holders:
  `ValidationCache` for snapshot-compare outcome caching, `SubmitTracker`/
  `SubmitAction` for the submit lifecycle, `AsyncFieldValidation` for async
  per-field checks (all re-exported by the facade at the same names)

## Most Users Should Use Instead

- [`gpui-form`](../gpui-form/README.md) for normal application development
- [`gpui-form-component`](../gpui-form-component/README.md) when you need the
  GPUI runtime helpers
