# gpui-form-prototyping-core

Utilities for generating gpui form scaffolding from `GpuiFormShape` inventory data.

This crate is useful when you want to rapidly prototype forms from your struct
definitions without hand-writing the gpui widget wiring.

## Usage

Enable the `inventory` feature on `gpui-form` and iterate the registered shapes:

```rs
use gpui_form::core::registry::{GpuiFormShape, inventory};
use gpui_form_prototyping_core::code_gen::FormShapeAdapter;
use gpui_form_prototyping_core::implementations::ComponentShape as _;

for shape in inventory::iter::<GpuiFormShape>() {
    let adapter = FormShapeAdapter::new(shape);
    let _children = adapter.child_elements();
}
```

See `examples/prototyping` for a full generator that writes formatted files.

If you prefer calling `inventory::iter` directly, add `inventory` to your dependencies.

## Feature flags

- `fluent`: use `es-fluent` keys for labels and descriptions.
