# gpui-form-schema Architecture

`gpui-form-schema` owns the shared metadata model used by macro expansion,
inventory registration, and downstream prototyping.

## Purpose

This crate is the runtime-safe metadata boundary between:

- derive/codegen internals
- the user-facing facade
- downstream tooling such as `gpui-form-prototyping-core`

It should describe component behavior and discovered form shape information, but
not own proc-macro parsing or token emission.

## Modules

- `src/lib.rs`: exports `components`, `layout`, and `registry`
- `src/components.rs`: component identity and behavior payload types
- `src/layout.rs`: non-rendering layout hints (`FieldLayout`, `LayoutWidth`)
- `src/registry.rs`: `GpuiFormShape`, `FieldVariant`, `inventory`
  collection, and the M5 schema-export view types (`FormSchema`,
  `FormSchemaField`, `FormSchemaComponent`, `FormSchemaLayout`)

## Metadata Model

### `ComponentKind`

Static identity for built-in component categories. It centralizes shared traits
such as:

- snake-case component naming
- whether a component is subscribable
- whether a component is focusable
- whether generated holder fields wrap in `Option<T>` by default

### `ComponentsBehaviour`

Per-field runtime behavior metadata with payloads for:

- select behavior
- infinite-select behavior
- number-input behavior
- phone-input behavior (`PhoneInputBehaviour`, carrying the optional
  `country_field` reference for `phone_input(country = ...)`)
- file-picker behavior
- nested-form behavior (milestone M3: `NestedBehaviour { form_struct }`,
  naming the composed form struct for generators)
- list behavior (milestone M3: `ListBehaviour { item, item_struct,
  add_label }` — the scalar item component kind, the reserved
  post-v1 nested item slot, and the optional add-button label override)

Adding variants to `ComponentKind`/`ComponentsBehaviour` is a semver break
for exhaustive downstream matches; the M3 variants landed with the
milestone's minor bump and are called out in the CHANGELOG.
`FieldVariant::is_nested()`/`is_list()` are the convenience predicates for
composed fields, and both kinds are value-only (no leaf control, never
Option-wrapped by the component).

This is the metadata level that downstream consumers use; derive/codegen
internals should not invent separate parallel runtime models.

### `GpuiFormShape`

Inventory-registered description of one derived form source struct.

Important fields:

- `struct_name`
- `components`
- `source_path`
- `koruma_enabled`
- `has_skipped_fields`

### `FieldVariant`

Per-field metadata for one generated component entry.

Important fields:

- `field_name`
- `value_type`
- `source_value_type`
- `optional`
- `wraps_in_option`
- `behaviour`
- `validations`
- `default_expr`
- `from_expr`
- `into_expr`
- `custom_shape`
- `custom_component`
- `custom_value_binding`
- `layout` (`FieldLayout`): non-rendering hints — `section`, `label`,
  `description`, `placeholder` (all `Option<&'static str>`), and `width`
  (`LayoutWidth`)
- `async_validator` (`Option<&'static str>`): metadata-only async validator
  path (M2 emission sketch)
- `depends_on` (`Option<&'static str>`): the sibling field this field
  depends on (M4, backlog #12) — metadata-only; reset-on-parent-change is
  consumer-authored
- `required_if` (`Option<&'static str>`): the controlling sibling for this
  field's conditional-required rule (M4, backlog #13) — recorded for
  tooling; the rule itself is enforced by a generated koruma validator in
  the derive, not by this metadata

Both M4 payloads store the sibling's field name — the same `&'static str`
representation a `FieldPath` name segment carries — through const builders
(`with_depends_on` / `with_required_if`), keeping the `inventory::submit!`
contract (const-callable, `Copy`/`&'static str` payloads only).

### `NumberInputKind::classify`, the one numeric-family classifier

`gpui-form-codegen` classifies each number-input field once at parse time —
with the number-input `as = ...` override string when present, else the
field type — and feeds the same `NumberInputKind` value to both the
recorded `NumberInputBehaviour.kind` payload and the emitted input
validator branch, so recorded metadata and generated validation cannot
disagree.

Prototyping's step arithmetic is a separate, deliberate caller: it calls
`classify` on the field's recorded `value_type` string — the raw field type
(unless a field-level `type = ...` override applies). It never consults the
number-input `as = ...` override, because the holder stores the raw field
type and stepping must follow the stored value's semantics (an
`as = f64` Decimal still steps through Decimal arithmetic).

### Schema-export view types (M5, backlog #14)

`registry.rs` also owns the export-facing projection of the metadata model:
`FormSchema` (one per form), `FormSchemaField` (one per non-skipped field),
`FormSchemaComponent` (an owned mirror of `ComponentsBehaviour` payloads), and
`FormSchemaLayout` (an owned mirror of `FieldLayout`). Design rules:

- **Read-only projection.** `From<&GpuiFormShape>`/`From<&FieldVariant>`
  convert the `&'static` inventory model into owned `String` data; there is no
  builder surface and no path back. The derive's `<Name>FormSchema::schema()`
  hosts the one `FieldVariant`/`FieldLayout` chain (the inventory submission
  registers that same const) and converts at runtime, so the export cannot
  drift from the registered shape (the layers do not invent a parallel
  runtime model).
- **Serializable unconditionally.** The view types (and `ComponentKind` /
  `NumberInputKind` / `LayoutWidth`) derive `Serialize`/`Deserialize`
  unconditionally — `serde` is a plain dependency of this crate, NOT an
  optional one, so schema export round-trips with no feature flag anywhere in
  the chain. This is deliberately different from the holder's opt-in serde
  story: the export is tooling-facing metadata, not persisted user values.
  Wire style: snake_case component tags matching `ComponentKind`'s `Display`
  (`{"component":"number_input",...}`), absent `Option` hints omitted with
  `skip_serializing_if` + `default` so documents stay readable and round-trip.
- **Coverage.** Field entries include everything `FieldVariant` carries:
  component payload details (select flags, phone country sibling, number
  validation type, nested form struct, list item kind), validations, value
  and conversion types, defaults, custom component hints, layout, the async
  validator path, and the M4 dependency payloads. Element validators
  (`each(...)`) stay element-scoped and do not appear in `validations` — the
  export projects what the inventory records, no more.

### `FieldLayout` and `LayoutWidth`

Non-rendering layout hints attached to each `FieldVariant` (METADATA-FIRST v1,
backlog feature #4). Key properties:

- **Metadata-only.** Nothing here describes how a field is rendered, painted,
  or laid out by GPUI. Consumers (prototyping generators, application code)
  decide how to interpret each hint.
- **`&'static str` boundary.** All string hints are `Option<&'static str>`
  because the derive emits them as string literals in the user crate (matching
  `default_expr`/`from_expr`).
- **Const-constructible.** Both `FieldLayout` and `LayoutWidth` are
  `const`-constructible (all builders are `const fn`) so the derive can build
  them inside `inventory::submit! { ... }` blocks via
  `FieldVariant::new(...).with_layout(FieldLayout::new()...)`.
- **Order-preserving sections.** `section` grouping is order-preserving:
  consecutive fields with the same section form one group; consumers must not
  reorder fields across the form.
- **Defaults.** `LayoutWidth` defaults to `Full`; an all-`None`/`Full` layout
  is `is_empty()` so consumers can skip emitting anything for it. `label`
  defaults to the field name at consumption time (not in the schema).
- **Serialization.** `LayoutWidth` derives `Display`/`EnumString` with
  `#[strum(serialize_all = "snake_case")]` so `width = half` round-trips
  consistently with `ComponentKind`.

## Data Flow

1. `gpui-form-codegen` parses field component syntax and turns it into typed
   component definitions.
1. `gpui-form-codegen` emits `ComponentsBehaviour` tokens for each field.
1. `gpui-form-derive` embeds those behavior tokens into generated
   `FieldVariant` metadata, alongside a `FieldLayout` built from the field's
   layout hints (`section`/`label`/`description`/`placeholder`/`width`).
1. When inventory registration is enabled, `gpui-form-derive` submits a
   `GpuiFormShape`. Independently of that feature, the derive's
   `<Name>FormSchema::schema()` converts the identical shape chain into a
   `FormSchema` document (M5) for docs, fixtures, and non-GPUI consumers.
1. `gpui-form-prototyping-core` reads that metadata (including layout hints)
   and generates scaffolded code; its `schema_markdown` helper renders the
   exported `FormSchema` as a Markdown reference table.

## Boundary Rules

This crate should own:

- runtime-safe metadata
- shared component identity
- inventory registration types

This crate should not own:

- parsing of `#[gpui_form(...)]` attributes
- proc-macro error diagnostics
- direct GPUI runtime implementations

## Coordination Rules

When adding or changing a component:

1. update `ComponentsBehaviour` and related payload structs here
1. update `gpui-form-codegen` so the derive layer emits the new metadata
1. update `gpui-form-prototyping-core` so the generator consumes the new
   behavior correctly
1. update `gpui-form-component` if runtime support is required

## When To Update This Document

Update this file when:

- `GpuiFormShape` or `FieldVariant` fields change
- `FieldLayout` / `LayoutWidth` shape, builders, or serialization change
- component behavior payloads change
- inventory ownership or registration semantics change
- schema-export view types, their serde wire format, or their projection
  coverage change
