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

## some-lib-forms

Storybook-style GPUI app that renders generated forms around the example types.
The checked-in `location_form` example shows the runtime-owned
`InfiniteSelectState` flow with `form_fields()` instead of manual child-select
rebuilding.
It includes a manual `Feature Audit` story for the recent headless additions:
`FormState` dirty tracking/reset, generated typed field paths, generated holder
debug data, and pointers to the generated User story for layout sections and
`number_input(as = f64)`.
It also includes a manual `Phone Verification` story that proves dynamic
country-driven phone validation in the UI with the `phonenumber` parser rather
than regex-only checks. The story uses the shared `gpui_form::phone` helper
instead of local phone-validation boilerplate, with separate fields for general
global validation and strict selected-country matching.

Run it with:

```sh
cargo run -p some-lib-forms
```

To open the feature audit screen directly:

```sh
cargo run -p some-lib-forms -- "Feature Audit"
```

Type in the username field to see `FormState::is_dirty` flip, then use
`Reset to baseline` and `Mark current clean`. Open the generated `User` story
to test the layout sections plus `number_input(as = f64)` balance/debt fields.

To open the phone validation story directly:

```sh
cargo run -p some-lib-forms -- "Phone Verification"
```

Try `415 555 2671` with `United States`, then switch the country to `France`.
Try `01 42 68 53 00` with `France`, then switch the country to
`United States`.

## gpui-form-component-story

Storybook-style GPUI app for the reusable runtime components themselves:
infinite select, date picker, and file picker.

Run it with:

```sh
cargo run -p gpui-form-component-story
```

## prototyping

Generator example that walks `GpuiFormShape` inventory data and emits scaffolded
form files into `examples/prototyping/output`. Generated Storybook form titles
use the example app's active locale, and generated labels/descriptions resolve
through `rust_i18n::t!` keys listed in the emitted `<FORM>_I18N_KEYS` consts.

Run it with:

```sh
cargo run -p prototyping
```

## mcp-submit

stdio MCP server built on the facade's `mcp` feature: generated submit and
editor tools, schema metadata, resources, and prompt templates for a
`GpuiForm`-backed demo model, with an in-repo JSON-RPC test client.

Run its tests with:

```sh
cargo test -p mcp-submit
```

## forms-demo

Small standalone GPUI app built entirely on the current stack (no storybook
shell): a `#[derive(GpuiForm)]` signup form with koruma validation, the runtime
date picker, and EN/FR/中文 locale switch buttons that drive the shared
`gpui-form-i18n` locale live — labels, select options, and the date picker all
re-render in the chosen language.

Run it with:

```sh
cargo run -p forms-demo
```

CI builds it in release on macOS and uploads the binary as the
`forms-demo-macos` artifact on every run.

## Localization

Each example crate owns its own `locales/{en,fr-FR,zh-CN}.yml` files plus
`rust_i18n::i18n!("locales", fallback = "en")` at its crate root — `t!` keys
are crate-local, so nothing is shared between them. Story apps switch the
locale through the `gpui-form-i18n` bridge helpers (`init`, `change_locale`);
generated and story rendering resolve text through `rust_i18n::t!` and
app-owned keys.
