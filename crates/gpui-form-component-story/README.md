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
English strings. The only `es-fluent` resources in this package are the
`infinite_select` namespace messages used by the demo enum metadata, so the
storybook can exercise fluent-backed `InfiniteSelect` labels and descriptions.
Those demo metadata resources ship in English, French (`fr-FR`), and
Simplified Chinese (`zh-CN`).
