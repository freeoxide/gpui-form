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

    let _field_imports = parts.imports;
}
```

## Main API

- `FormShapeAdapter::parts()` computes the validated, reusable fragments for one
  shape
- `FormShapeAdapter::generate_file(&impl FormLayout)` builds a complete
  `syn::File`
- `FormLayout` lets callers define the overall file structure
- `PrototypingError` reports malformed metadata without panicking

## Example Workflow

The workspace example in [`examples/prototyping`](../../examples/prototyping)
shows the normal flow:

1. enable `gpui-form`'s `inventory` feature
1. iterate `inventory::iter::<GpuiFormShape>()`
1. adapt each shape with `FormShapeAdapter`
1. render a file through a custom `FormLayout`
1. format the `syn::File` with `prettyplease`

Generated infinite-select fields target the runtime `InfiniteSelectState`
surface directly, including `form_fields()` for rendering and the richer
`InfiniteSelectEvent<T>` payload for state updates.

## Feature Flags

- `fluent`: use `es-fluent` keys for generated labels and descriptions

## Most Users Should Use Instead

- [`gpui-form`](../gpui-form/README.md) for hand-written forms plus derives
- [`gpui-form-schema`](../gpui-form-schema/README.md) if you only need the
  metadata layer
