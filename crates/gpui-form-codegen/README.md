# gpui-form-codegen

Internal code-generation support for `gpui-form-derive`.

This crate is not the normal entry point for application code. Most users
should use [`gpui-form`](../gpui-form/README.md), and proc-macro consumers
should usually use [`gpui-form-derive`](../gpui-form-derive/README.md).

## What It Does

- parses `#[gpui_form(component(...))]` into a typed component model
- emits per-component field layout tokens for generated forms
- translates parse-time component options into runtime metadata tokens aligned
  with `gpui-form-schema`

## Who This Crate Is For

- maintainers extending the derive system
- tooling authors experimenting with `gpui-form` token generation internals

## Most Users Should Use Instead

- [`gpui-form`](../gpui-form/README.md) for application development
- [`gpui-form-derive`](../gpui-form-derive/README.md) for proc-macro entry
  points
- [`gpui-form-schema`](../gpui-form-schema/README.md) for runtime metadata
