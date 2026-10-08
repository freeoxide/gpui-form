# Examples

This directory is the canonical index for runnable `gpui-form` workspace
examples.

## some-lib

Source structs and enums that demonstrate the supported derive surface:

- basic input, number input, checkbox, switch, and select fields
- searchable select, date-picker conversion, and skipped-field workflows
- nested `InfiniteSelect` enums, including index-path and key-path round trips
  plus typed path-error reporting, custom persisted keys, and key-path string
  serialization
- custom component state, local shapes, and cross-crate shapes
- Koruma validation wiring
- newtype-backed numeric validation
- `cfg_attr`-gated derive usage
- empty forms and date conversion

## some-lib-custom-components

External custom component state and UI types used by the example forms.

This crate demonstrates the cross-crate custom-component shape workflow.

## Removed story apps

`examples/some-lib-forms` and `crates/gpui-form-component-story` (story-style
GPUI browsers for generated forms and runtime components) were removed from
this fork: they targeted the retired longbridge/gpui-component dock API and do
not build against gpui-kit 0.7.0. Recover them from git history if needed.

## prototyping

Generator example that walks `GpuiFormShape` inventory data and emits scaffolded
form files into `examples/prototyping/output`. Generated form titles use the
example app's active locale, and generated labels/descriptions resolve
through `rust_i18n::t!` keys listed in the emitted `<FORM>_I18N_KEYS` consts.

Run it with:

```sh
cargo run -p prototyping
```

## forms-demo

Standalone GPUI app built entirely on the current stack,
packing the edge cases an application form actually hits. One `#[derive(GpuiForm)]`
signup form covers: required text input, optional field whose validators only
fire when present (email), number input with range validation, searchable and
plain selects over `SelectItem(fluent)` enums, checkbox and switch bool fields,
a newtype-backed field with inner koruma validation (`#[koruma(newtype)]` invite
code whose `Inner` error is mapped to a localized key), a form-bound
`component(date_picker)` field, and a `#[gpui_form(skip)]` field that is absent
from the holder and typed paths (shown live in the footer). Standalone runtime
components add a date picker and a multiple-mode file picker whose pluralized
count text follows the locale. Validation errors render through `validation.*`
`t!` keys — including newtype `Inner` errors — with unit tests asserting the
French/Chinese strings, plus per-crate key-parity tests. EN/FR/中文 buttons
drive the shared `gpui-form-i18n` locale; the footer shows the live locale,
`FormState::is_dirty`, picked paths count, and the typed field paths.

Run it with:

```sh
cargo run -p forms-demo
```

CI builds it in release on macOS and uploads the binary as the
`forms-demo-macos` artifact on every run.

## Localization

Each example crate owns its own `locales/{en,fr-FR,zh-CN}.yml` files plus
`rust_i18n::i18n!("locales", fallback = "en")` at its crate root — `t!` keys
are crate-local, so nothing is shared between them. Example apps switch the
locale through the `gpui-form-i18n` bridge helpers (`init`, `change_locale`);
generated and app rendering resolve text through `rust_i18n::t!` and
app-owned keys.
