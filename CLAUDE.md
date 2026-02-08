# Project Overview

`gpui-form` is a form-generation ecosystem written in **Rust**, built on top of `gpui` and `gpui-component`. It focuses on:

1. **Type Safety**: Derive macros generate strongly-typed form state and metadata at compile time.
1. **Ergonomics**: `#[derive(GpuiForm)]` minimize boilerplate.
1. **Developer Experience**: Inventory-based shape registry enables fast prototyping and codegen.

## Architecture Documentation Index

| Crate | Link to Architecture Doc | Purpose |
| --- | --- | --- |
| **Core** | | |
| `gpui-form` | [Architecture](crates/gpui-form/docs/ARCHITECTURE.md) | Facade crate; re-exports macros/core/components, numeric helpers. |
| `gpui-form-core` | [Architecture](crates/gpui-form-core/docs/ARCHITECTURE.md) | Component definitions, registry, codegen helpers. |
| `gpui-form-derive` | [Architecture](crates/gpui-form-derive/docs/ARCHITECTURE.md) | Proc macros for form derivation and select helpers. |
| **Components & Runtime** | | |
| `gpui-form-component` | [Architecture](crates/gpui-form-component/docs/ARCHITECTURE.md) | Runtime helpers (InfiniteSelect). |
| **Prototyping** | | |
| `gpui-form-prototyping-core` | [Architecture](crates/gpui-form-prototyping-core/docs/ARCHITECTURE.md) | Codegen from inventory shapes for prototyping. |
| **Internal** | | |
| `gpui-form-internal-macros` | [Architecture](crates/gpui-form-internal-macros/docs/ARCHITECTURE.md) | Internal proc macros used by core. |

## Crate Descriptions

### Core Layers

- **`gpui-form`**: User-facing facade. Re-exports derive macros, core metadata, and optional runtime components. Hosts numeric validation helpers.
- **`gpui-form-core`**: Shared metadata and component definitions used by macros and prototyping.
- **`gpui-form-derive`**: Proc macros that expand form structs into component fields, value holders, and optional inventory registrations.

### Components & Runtime

- **`gpui-form-component`**: Runtime helpers for advanced components (currently InfiniteSelect).

### Prototyping

- **`gpui-form-prototyping-core`**: Builds gpui form scaffolding by consuming `GpuiFormShape` inventory data.

### Internal

- **`gpui-form-internal-macros`**: Small derive macros used internally by core to reduce boilerplate.

## Examples

- `examples/i18n` - localization resources used by the examples.
- `examples/some-lib` - crate defining shared example types.
- `examples/some-lib-forms` - storybook-like gpui app showcasing generated forms. Run with `cargo run -p some-lib-forms`.
- `examples/prototyping` - prototyping generator that emits form scaffolding. Run with `cargo run -p prototyping`.

## Agent Notes

- Ignore all folders matching `**/__crate_paths/**` (generated files).
- When changing public APIs or behavior in a crate, update that crate's `docs/ARCHITECTURE.md`.
- When adding a component, update:
  - `gpui-form-core` `Components` + `ComponentLayout` implementation.
  - `gpui-form-prototyping-core` `FieldCodeGenerator` mapping.
