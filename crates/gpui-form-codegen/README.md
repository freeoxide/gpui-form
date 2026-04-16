# gpui-form-codegen

Proc-macro-adjacent parsing and token-generation support for `gpui-form-derive`.

## What it provides

- Parse-time component option parsing for `#[gpui_form(component(...))]`
- Per-component field layout emitters
- Runtime metadata token generation aligned with `gpui-form-schema`

## Notes

- This is primarily an internal workspace crate.
- Public consumers should usually depend on `gpui-form`, `gpui-form-core`,
  `gpui-form-runtime`, or `gpui-form-schema` instead.
