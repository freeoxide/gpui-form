# gpui-form-i18n

GPUI global-state i18n bridge for the `gpui-form` workspace, backed by
[`rust-i18n`](https://crates.io/crates/rust-i18n).

Most application users should depend on [`gpui-form`](../gpui-form) instead
and consume i18n through `gpui_form` re-exports; this crate is the shared
locale plumbing underneath it.

## Who this crate is for

- Form crates and examples that need **one locale** shared between GPUI Kit
  widgets (`gpui_kit::component`) and generated form components.
- Consumers migrating off `es-fluent` that still want the
  `init` / `change_locale` / `localize_message` / `localize_label` helper
  names.

## What it does

- Installs an `I18n` gpui `Global` holding the active locale.
- `init`, `init_with_language`, `replace_with_language`, and `change_locale`
  switch that locale and propagate it to the shared `rust-i18n` locale, which
  `gpui_kit::component::set_locale` also writes to (with the `component`
  feature).
- `localize_message` / `localize_label` resolve keys embedded from
  `locales/*.yml` via `rust_i18n::i18n!("locales", fallback = "en")`; a
  missing translation falls back to `humanize_key`.
- `locale` files currently cover the runtime component strings
  (`date_picker.*`, `file_picker.*`) for `en`, `fr-FR`, and `zh-CN`.

## Usage

```rust
use gpui_form_i18n::{change_locale, init, localize_message};

fn setup(cx: &mut gpui::App) {
    init(cx);
    change_locale(cx, unic_langid::langid!("fr-FR")).unwrap();
    assert_eq!(
        localize_message(cx, "file_picker.browse"),
        "Parcourir"
    );
}
```

Keys follow `namespace.snake_case` naming; a missing translation humanizes
instead of leaking the raw key:

```rust
use gpui_form_i18n::I18n;

let i18n = I18n::new_with_language(unic_langid::langid!("zh-CN"));
assert_eq!(i18n.localize_message("date_picker.select_date"), "选择日期");
assert_eq!(i18n.localize_message("unknown_key"), "Unknown Key");
```
