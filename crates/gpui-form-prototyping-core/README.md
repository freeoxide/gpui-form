# gpui-form-prototyping-core

Scaffolding utilities built on top of `GpuiFormShape` inventory data.

Use this crate when you want to generate GPUI form code from the metadata
emitted by `#[derive(GpuiForm)]` instead of wiring forms by hand.

Most application code should still start with
[`gpui-form`](../gpui-form/README.md).

## Quick Example

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

## Main API

- `FormShapeAdapter::parts()` returns validated identifiers, imports, and form
  fragments for one shape
- `FormShapeAdapter::generate_file(&impl FormLayout)` renders a full file with
  your chosen layout
- `FormLayout` lets callers define the overall file structure
- `PrototypingError` reports malformed metadata without panicking

## Example Workflow

The workspace example in [`examples/prototyping`](../../examples/prototyping)
shows the normal flow:

1. enable `gpui-form`'s `inventory` feature
1. iterate `inventory::iter::<GpuiFormShape>()`
1. adapt each shape with `FormShapeAdapter`
1. render a file through a custom `FormLayout`
1. clear stale generated modules and write the generated form files

The example's layout emits each form as a plain GPUI component: a `pub struct`
with a `pub fn new(window, cx)` constructor and a `Render` implementation, with
no story-app or `Story` trait glue. Route the form title through the consuming
crate's i18n helper (for example `localize(cx, "<form>_label")`) so generated
titles follow the active locale.

Generated infinite-select and file-picker fields use the same runtime helpers
that hand-written forms use. Generated text inputs use the form-side
`FieldVariant::value_type` and parse non-`String` values with `FromStr` instead
of assuming every text field stores `String`.

The adapter also consumes non-rendering layout hints from
`FieldVariant::layout` (metadata-first, feature #4). It groups consecutive
fields by `section` (emitting a section heading via the `field()` builder when
the section changes, order-preserving), emits `layout.label` /
`layout.description` verbatim when present and otherwise `rust_i18n::t!`
lookups of `<form>.<field>_label` / `<form>.<field>_description` keys that
fall back to the title-cased field name on a miss, and surfaces `description`
where it already emits help text.
`placeholder` is reachable through `ResolvedField::layout().placeholder` for
consumers that own a richer input builder; the v1 generator does not render it
itself. Layout hints on skipped fields are ignored (no `FieldVariant` is
emitted for them).

Custom fields remain inert by default. If a field's shape opts into
`value_binding`, the adapter emits generic seed and subscription hooks through
`gpui_form::custom::CustomComponentValueAdapter<T>`.

## Localization

Generated labels, descriptions, and form titles resolve through `rust-i18n`.
Generated files call `rust_i18n::t!(...)`, so the consuming crate must depend
on `rust-i18n`, call `rust_i18n::i18n!("locales", fallback = "en")` at its
crate root, and own the `<form>.<field>_label` / `_description` keys —
`layout.label` / `layout.description` overrides replace the lookup with a
verbatim literal. `FormParts` carries `i18n_key_items` (emitted as a
doc-commented `<FORM>_I18N_KEYS` const) and `collect_i18n_keys(&GpuiFormShape)`
lists every key a shape needs, so generators can scaffold the locale files.
This crate's own `locales/{en,fr-FR,zh-CN}.yml` double as the reference
template.

## Most Users Should Use Instead

- [`gpui-form`](../gpui-form/README.md) for hand-written forms plus derives
- [`gpui-form-schema`](../gpui-form-schema/README.md) if you only need the
  metadata layer
