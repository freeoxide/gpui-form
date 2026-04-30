# gpui-form-component-story

Storybook gallery for the runtime helpers in
[`gpui-form-component`](../gpui-form-component/README.md).

This package keeps demo UI, story-only localized copy, and the launcher binary
outside the runtime library crate. Most users should depend on
[`gpui-form`](../gpui-form/README.md) or `gpui-form-component`, not this
package.

Run the gallery with:

```sh
cargo run -p gpui-form-component-story
```

The story copy is split across `date_picker`, `file_picker`, and
`infinite_select` `es-fluent` namespaces under this package's `i18n/` assets.
