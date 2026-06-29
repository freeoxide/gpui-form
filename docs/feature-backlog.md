# Feature backlog

Status: **living** — candidate and shipped features for `gpui-form` work. Shipped items are marked inline.

## Scope boundary

`gpui-form` should stay centered on derive-driven, type-safe form generation.
It is not a general GPUI widget library.

The workspace currently has three related layers:

1. **Headless form metadata and generated value state** in the derive, schema,
   codegen, and core crates.
2. **GPUI integration glue** that maps generated form fields onto
   `gpui-component` controls.
3. **Small form-specific runtime helpers** in `gpui-form-component`, such as
   date picker, file picker, infinite select state, and custom component
   contracts.

Prefer features that improve generated state, metadata, validation, conversion,
and prototyping. Add new rendered controls only when they are form-specific
runtime primitives or thin mappings to existing `gpui-component` widgets.

## Highest-value candidates

### 1. Form-state persistence and dirty tracking

> **Status: shipped (feature #1).** Opt-in `serde` (de)serialization of the generated holder plus
> the pure, GPUI-free `gpui_form::core::FormState<H>` wrapper (is_dirty / reset_to_baseline /
> diff_against) are live. See `form-state-persistence.md`.

See [`form-state-persistence.md`](form-state-persistence.md).

Generated forms should support save/restore workflows and know whether the user
changed the holder from its initial value. This is the most immediately useful
headless feature because it benefits settings panes, editors, wizards, and
long-lived form sessions without expanding the widget surface.

Candidate API shape:

```rs
let state = gpui_form::core::FormState::new(initial_holder);

state.is_dirty();
state.reset_to_baseline();
state.diff_against(&initial_holder);
```

### 2. Nested forms

Support composing generated forms from other generated forms.

```rs
#[derive(gpui_form::GpuiForm)]
pub struct User {
    #[gpui_form(component(input))]
    pub name: String,

    #[gpui_form(component(nested))]
    pub address: Address,
}
```

This belongs primarily in the derive, schema, codegen, and prototyping layers.
The runtime should only need enough metadata for generated render code to place
the nested fields.

Key design questions:

- Should nested forms flatten field names or preserve a nested path?
- How should validation errors expose nested paths?
- Should nested field groups have optional labels, sections, or collapsible
  rendering metadata?

### 3. Repeated fields

Support editable collections such as `Vec<T>`.

```rs
#[derive(gpui_form::GpuiForm)]
pub struct PostEditor {
    #[gpui_form(component(list(item = input)))]
    pub tags: Vec<String>,

    #[gpui_form(component(list(item = nested)))]
    pub contacts: Vec<Contact>,
}
```

This is a form-generation feature more than a widget feature. The generated
holder needs typed add/remove/reorder behavior, and prototyping needs to emit
repeatable row scaffolding.

Keep the first version conservative:

- `Vec<T>` only.
- Add/remove support before drag reordering.
- Scalar item components before nested item forms.

### 4. Layout and section metadata

> **Status: shipped (METADATA-FIRST v1).** The `section`, `label`,
> `description`, `placeholder`, and `width` hints are live. They attach a
> `gpui_form::schema::FieldLayout` to each `FieldVariant` for generators and
> prototyping to consume (the prototyping generator groups by `section`,
> prefers `label`, and emits `description` where it already produces help text).
> The generated form code itself is unchanged; `width` is a hint, not a layout
> engine. Richer layout (columns, collapsible sections) builds on this later.

Add non-rendering hints that generated/prototyped forms can consume.

```rs
#[derive(gpui_form::GpuiForm)]
pub struct AccountSettings {
    #[gpui_form(section = "Account", component(input))]
    pub username: String,

    #[gpui_form(section = "Advanced", component(switch))]
    pub enable_experimental: bool,
}
```

This should stay metadata-first. Application code and prototyping generators can
choose how to render sections, columns, help text, and field grouping.

Useful first attributes:

- `section = "..."`
- `label = "..."`
- `description = "..."`
- `placeholder = "..."`
- `width = full | half | third`

### 5. Async/select provider contracts

Add a typed provider abstraction for choices that are not known at compile time.

```rs
#[gpui_form(component(async_select(provider = UserSearchProvider)))]
pub assignee: Option<UserId>,
```

This should be designed as a provider contract plus generated state wiring, not
as a new standalone select widget. The rendered control can still be backed by
`gpui-component` where possible.

Key design questions:

- Is the provider sync, async, or both?
- Does search live in the provider trait or the component options?
- How are selected values restored when only an ID is persisted?
- How are loading and error states exposed to generated render code?

### 6. Validation and error metadata improvements

Koruma integration already mirrors validator attributes onto generated value
holders. The next step is better generated error access for UI code.

Candidate APIs:

```rs
holder.validate_fields();
holder.field_errors();
holder.error_for(UserProfileFormFields::username());
```

The main value is stable typed field paths, including future nested/list paths,
so callers do not have to match validation output with ad hoc strings.

### 7. Submit lifecycle helpers

Add optional headless workflow state for common submit flows.

```rs
form_submission.is_submitting();
form_submission.last_error();
form_submission.submit(|value| async move { save(value).await });
```

This should be optional runtime glue, not required generated state. The holder
should remain plain data.

### 8. Typed field paths and field IDs

> **Status: shipped (FLAT v1).** The shared primitive
> `gpui_form_core::FieldPath` (re-exported as `gpui_form::FieldPath`) and the
> generated `<Name>FormPath` newtype per form are live. Flat fields only:
> typed nested-path and list-item-path constructors remain, tracked under
> backlog #2 ("Nested forms") and #3 ("Repeated fields").

Expose stable generated identifiers for every field.

```rs
UserProfileFormPath::username();
UserProfileFormPath::new(&["address", "city"]);
```

The generated `<Name>FormPath` is a newtype over the headless, GPUI-free,
serde-free `FieldPath`, giving validation, dirty tracking, focus management,
analytics, and schema export one shared way to name fields without ad hoc
strings. It ships with one same-named constructor per non-skipped field (no
generics, even on generic source structs) and is the naming foundation for the
upcoming field-level validation (#6), field-level diff (#9), schema export
(#14), and nested/list paths (#2/#3).

The FLAT v1 surface covers single-field constructors; hand-built multi-segment
paths via `<Name>FormPath::new(&["a", "b"])` work today, and typed nested/list
composition expands under #2/#3.

### 9. Patch and delta generation

Generate changed-field output from a holder and a baseline.

```rs
let patch = form_state.patch();
let changed_fields = form_state.changed_fields();
```

This should build on the same baseline model as dirty tracking. It is useful for
PATCH APIs, settings writes, audit logs, and "save only what changed" workflows.

Keep this headless: the output should be typed data or typed field paths, not UI
events.

### 10. Partial form holders

Generate an optional-field holder for partial updates.

```rs
#[derive(gpui_form::GpuiForm)]
#[gpui_form(partial_holder)]
pub struct UserUpdate {
    #[gpui_form(component(input))]
    pub display_name: String,
}
```

This is distinct from the normal value holder. The normal holder represents the
editable form state; the partial holder represents a sparse update payload.

Useful workflows:

- PATCH endpoints.
- Progressive forms.
- Edit screens that only send touched fields.
- Admin tools that need tri-state "unchanged / clear / set" behavior.

### 11. Wizard and step metadata

Add headless grouping for multi-step forms.

```rs
#[derive(gpui_form::GpuiForm)]
pub struct Signup {
    #[gpui_form(step = "Account", component(input))]
    pub email: String,

    #[gpui_form(step = "Profile", component(input))]
    pub display_name: String,
}
```

The derive should expose metadata for steps and field ordering. Applications and
prototyping generators can decide how to render progress indicators, navigation,
and per-step validation.

### 12. Field dependency metadata

Represent relationships between fields without owning the UI behavior.

```rs
#[gpui_form(component(select))]
pub country: Country,

#[gpui_form(component(select), depends_on = country)]
pub state: Option<State>,
```

This should produce metadata that generated or application code can consume for
conditional loading, dependent validation, and reset-on-parent-change behavior.

### 13. Conditional requiredness

Expose validation metadata for fields that are required only when another field
has a particular state.

```rs
#[gpui_form(component(input), required_if = "subscribe")]
pub email: Option<String>,
```

This should integrate with the validation layer rather than live as display-only
metadata. The design should avoid stringly typed expressions long term; typed
field paths are the likely foundation.

### 14. Schema export

Export a structured description of a form's fields, component kinds, defaults,
validation metadata, sections, steps, and custom component hints.

```rs
let schema = UserProfileFormSchema::schema();
```

This would help documentation generation, test fixtures, prototyping tools, and
non-GPUI consumers that want to inspect form shape without running the GPUI
runtime.

### 15. Serde attribute passthrough

If holder persistence is added, allow generated holder fields to preserve
serialization customizations.

```rs
#[gpui_form(component(input), serde(rename = "displayName"))]
pub display_name: String,
```

This should be scoped to generated holder serialization, not source struct
serialization. The derive should make that boundary explicit in docs and error
messages.

### 16. Custom component introspection

Expose richer metadata for `component(custom(...))` so prototyping can emit
better placeholders.

Candidate metadata:

- component display name
- expected value binding
- whether the component is clearable
- suggested imports
- app-owned render wrapper hints

This should not require custom components to adopt a heavyweight trait unless
the extra metadata is needed.

### 17. Numeric validation coverage

Numeric input validation is currently structured around explicit character
checks plus `FromStr` parsing for normal numeric fields. That is the right
direction: it is not regex-driven and should stay type-backed.

The important invariant is that `number_input(as = ...)` parses against the
validation type. That option is meant to let custom numeric types use a standard
numeric family for input semantics without falling back to shape-only checks.

Target behavior:

```rs
#[gpui_form(component(number_input(as = f64)))]
pub price: rust_decimal::Decimal,
```

- Character-level checks still allow useful intermediate input such as `""`,
  `"-"`, and `"0."`.
- Completed values parse against the validation type, such as `f64`.
- Conversion into the field type remains owned by the generated value-holder
  conversion path.
- Repeated decimal points, misplaced signs, overflow for integer validation
  types, and other malformed values are rejected by parsing, not by a growing
  pile of special cases.

### 18. Phone, email, URL, and typed text validation

Some values look simple but should not use numeric components. Phone numbers are
the clearest example: they can contain `+`, spaces, punctuation, extensions,
country-specific rules, and leading zeros. Treat them as strings or domain value
objects, not integers.

```rs
#[gpui_form(component(input), validate(phone(region = "US")))]
pub phone_number: Option<String>,
```

Dynamic regions should also be supported for forms where country is selected in
another field:

```rs
#[gpui_form(component(select))]
pub country: Country,

#[gpui_form(component(input), validate(phone(region_from = country)))]
pub phone_number: Option<String>,
```

This should be a text-validation feature, not a `number_input` feature.

Recommended direction:

- Keep the rendered control as `component(input)` unless a real phone-specific
  UI wrapper is needed later.
- Add structured validation metadata for common text domains such as phone,
  email, URL, slug, and UUID.
- For phone numbers, prefer a real parser/normalizer such as a libphonenumber
  implementation instead of regex-only validation.
- Support both static phone regions and dynamic regions derived from typed field
  paths, after field-path metadata exists.
- Store normalized output separately from display formatting when possible, for
  example E.164 for phone numbers.
- Keep application-owned domain types supported through `type`, `from`, and
  `into`.

## Component mapping candidates

These are useful only if implemented as mappings to existing controls or small
form-specific runtime wrappers. They should not turn `gpui-form` into a broad
component library.

### Textarea mapping

```rs
#[gpui_form(component(textarea))]
pub bio: Option<String>,
```

This is a good low-risk candidate if `gpui-component` already exposes the
needed multiline input behavior.

### Radio group mapping

```rs
#[gpui_form(component(radio))]
pub visibility: Visibility,
```

This should reuse `SelectItem`-like metadata and target enum choices where a
dropdown is too heavy.

### Slider/range mapping

```rs
#[gpui_form(component(slider(min = 0, max = 100, step = 5)))]
pub volume: u8,
```

This should build on numeric metadata and validation boundaries rather than
creating a parallel validation system.

## Suggested order

1. Form-state persistence and dirty tracking — **shipped (feature #1)**.
2. Numeric validation hardening.
3. Typed field paths and field IDs — **shipped (FLAT v1)**; nested/list
   composition expands under #2/#3.
4. Layout and section metadata — **shipped (METADATA-FIRST v1)**; richer
   layout (columns, collapsible sections) builds on this later.
5. Validation/error metadata improvements.
6. Patch and delta generation.
7. Nested forms.
8. Repeated fields.
9. Partial form holders.
10. Wizard and step metadata.
11. Field dependency metadata and conditional requiredness.
12. Schema export.
13. Async/select provider contracts.
14. Phone, email, URL, and typed text validation.
15. Thin component mappings such as textarea, radio, and slider.

This order keeps the early work mostly headless and additive, then moves into
larger code-generation features after the metadata and state model are stronger.
