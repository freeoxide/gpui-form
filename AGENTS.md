# Project Overview

`gpui-form` is a form-generation ecosystem written in **Rust**, built on top of `gpui` and `gpui-component`. It focuses on:

1. **Type Safety**: Derive macros generate strongly-typed form state and metadata at compile time.
1. **Ergonomics**: `#[derive(GpuiForm)]` minimize boilerplate.
1. **Developer Experience**: Inventory-based shape registry enables fast prototyping and codegen.

## Architecture Documentation Index

| Crate | Link to Architecture Doc | Purpose |
| --- | --- | --- |
| **Core** | | |
| `gpui-form` | [Architecture](crates/gpui-form/docs/ARCHITECTURE.md) | Facade crate; re-exports macros plus `core` / `runtime` / `schema`. |
| `gpui-form-core` | [Architecture](crates/gpui-form-core/docs/ARCHITECTURE.md) | Pure helper logic such as numeric validation. |
| `gpui-form-schema` | [Architecture](crates/gpui-form-schema/docs/ARCHITECTURE.md) | Runtime/schema metadata and inventory registry types. |
| `gpui-form-derive` | [Architecture](crates/gpui-form-derive/docs/ARCHITECTURE.md) | Proc macros for form derivation and select helpers. |
| **Components & Runtime** | | |
| `gpui-form-component` | [Architecture](crates/gpui-form-component/docs/ARCHITECTURE.md) | GPUI-facing runtime helpers, re-exported by the facade as `gpui_form::runtime`. |
| **Prototyping** | | |
| `gpui-form-prototyping-core` | [Architecture](crates/gpui-form-prototyping-core/docs/ARCHITECTURE.md) | Codegen from inventory shapes for prototyping. |
| **Internal** | | |
| `gpui-form-codegen` | [Architecture](crates/gpui-form-codegen/docs/ARCHITECTURE.md) | Proc-macro-adjacent parse-time component parsing and token generation. |

## Crate Descriptions

### Core Layers

- **`gpui-form`**: User-facing facade. Re-exports derive macros, core metadata, and optional runtime components. Hosts numeric validation helpers.
- **`gpui-form-core`**: Pure, non-GPUI helpers used by generated code.
- **`gpui-form-schema`**: Shared metadata and registry definitions used by macros and prototyping.
- **`gpui-form-derive`**: Proc macros that expand form structs into component fields, value holders, and optional inventory registrations.

### Components & Runtime

- **`gpui-form-component`**: GPUI-facing runtime helper implementations, re-exported by the facade as `gpui_form::runtime`.

### Prototyping

- **`gpui-form-prototyping-core`**: Builds gpui form scaffolding by consuming `GpuiFormShape` inventory data.

### Internal

- **`gpui-form-codegen`**: Parse-time component parsing and token generation used by `gpui-form-derive`.

## Examples

- `examples/i18n` - localization resources used by the examples.
- `examples/some-lib` - crate defining shared example types.
- `examples/some-lib-forms` - storybook-like gpui app showcasing generated forms. Run with `cargo run -p some-lib-forms`.
- `examples/prototyping` - prototyping generator that emits form scaffolding. Run with `cargo run -p prototyping`.

## Agent Notes

- Ignore all folders matching `**/__crate_paths/**` (generated files).
- When changing public APIs or behavior in a crate, update that crate's `docs/ARCHITECTURE.md`.
- When adding a component, update:
  - `gpui-form-codegen` `Components` + `ComponentLayout` implementation.
  - `gpui-form-schema` runtime behavior metadata.
  - `gpui-form-prototyping-core` `FieldCodeGenerator` mapping.
