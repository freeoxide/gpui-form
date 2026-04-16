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
    let parts = FormShapeAdapter::new(shape)
        .parts()
        .expect("shape metadata should be valid");
    let _imports = parts.imports;
}
```

Use `generate_file(&impl FormLayout)` when you want the crate to run your layout
over the computed parts and return a full `syn::File`. See
`examples/prototyping` for a complete generator that writes formatted files.

Both `parts()` and `generate_file(...)` return `Result<_, PrototypingError>` so
custom tooling gets a structured error instead of a panic when shape metadata is
malformed.

If you prefer calling `inventory::iter` directly, add `inventory` to your dependencies.

## Feature flags

- `fluent`: use `es-fluent` keys for labels and descriptions.
