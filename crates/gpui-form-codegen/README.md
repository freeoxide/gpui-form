# gpui-form-codegen

Internal support crate for the workspace derive system.

This crate is for maintainers working on `gpui-form` itself and for advanced
tooling experiments around the derive layer. It is not the normal entry point
for application code.

## Who This Crate Is For

- maintainers extending `gpui-form` derive support
- advanced tooling authors working alongside the workspace internals

## What It Does

Provides the shared support layer behind the workspace's form derives.

If you are looking for the public derive API, start with
[`gpui-form`](../gpui-form/README.md) or
[`gpui-form-derive`](../gpui-form-derive/README.md) instead.

## Most Users Should Use Instead

- [`gpui-form`](../gpui-form/README.md) for normal application development
- [`gpui-form-derive`](../gpui-form-derive/README.md) for the public derive
  crate
- [`gpui-form-schema`](../gpui-form-schema/README.md) for metadata and
  inventory types
