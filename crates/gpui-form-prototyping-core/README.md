# gpui-form-prototyping-core

Utilities for generating gpui form scaffolding from `GpuiFormShape` inventory data.

This crate is useful when you want to rapidly prototype forms from your struct
definitions without hand-writing the gpui widget wiring.

## Usage

Enable the `inventory` feature on `gpui-form` and iterate the registered shapes:

```rs
use gpui_form::schema::registry::{GpuiFormShape, inventory};
use gpui_form_prototyping_core::FormShapeAdapter;

for shape in inventory::iter::<GpuiFormShape>() {
    let parts = FormShapeAdapter::new(shape).parts();
    let _imports = parts.imports;
}
```

Use `generate_file(&impl FormLayout)` when you want the crate to assemble a full
`syn::File`. See `examples/prototyping` for a complete generator that writes
formatted files.

If you prefer calling `inventory::iter` directly, add `inventory` to your dependencies.

## Feature flags

- `fluent`: use `es-fluent` keys for labels and descriptions.
