# gpui-form-component-story

Storybook gallery for the runtime helpers in
[`gpui-form-component`](../gpui-form-component/README.md).

This package keeps demo UI and the launcher binary outside the runtime library
crate. Most users should depend on [`gpui-form`](../gpui-form/README.md) or
`gpui-form-component`, not this package.

Run the gallery with:

```sh
cargo run -p gpui-form-component-story
```

Story titles, descriptions, diagnostics, and other demo chrome are in-place
English strings. Text passed into the demo components is rust-i18n-backed via
this package's `locales/{en,fr-FR,zh-CN}.yml`: the `infinite_select` namespace
covers demo enum metadata, while `date_picker` and `file_picker` cover
component placeholders, prompts, and action labels. Locale switching goes
through the `gpui-form-i18n` bridge re-exported in `src/i18n.rs`
(`init`, `change_locale`).
