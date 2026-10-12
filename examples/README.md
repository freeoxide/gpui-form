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
- a nested form (`component(nested)` `Address` composed into `PostEditor`)
  and a scalar-item repeated field (`component(list(item = input))` tags with
  `each(...)` element validators) — milestone M3 shapes, exercised by the
  `typed_paths` tests (composed `address.city` paths, typed `tags.2` item
  paths)
- field dependencies (milestone M4): `User.state` records
  `depends_on = country`, and `User.email` is conditionally required via
  `required_if = subscribe_newsletter` (a desugared koruma validator, so the
  violation surfaces through the typed error surface)
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
from the holder and typed paths (shown live in the footer). The app drives the
generated `SignupFormState` facade: inputs mark touched on blur and on
change (the root view tracks focus and every action button releases field
focus on click, so blur fires on click-away too), other component kinds on
change, the footer lists the live changed/touched fields
through `FieldPath` Display, and per-field errors render through
`error_for(&SignupFormPath::..)` + `issue_keys()` — so `render` never runs
koruma validation or builds a throwaway dirty-check clone. The M2 submit
lifecycle is wired end-to-end: a Submit button drives the opt-in
`SignupFormSubmission` tracker (`#[gpui_form(submit)]`) through
`begin()` → a spawned `SubmitAction` future with simulated latency →
`finish()`, counting every attempted submit (the re-entry guard is the only
gate; an invalid form finishes the attempt immediately with the typed
koruma error, so `submit_count()` increments on failed attempts too),
disables while submitting, syncs the baseline on success, and
renders `last_error` banners by re-keying the typed koruma error through
`SignupFormFieldErrors` into the localized `validation.*` keys; a simulated
async availability check on the email field implements the
`AsyncFieldValidation` contract with a generation guard against stale
completions; and the facade's snapshot-compare validation cache feeds a live
`is_valid` footer indicator. Standalone runtime
components add a date picker and a multiple-mode file picker whose pluralized
count text follows the locale. Milestone M3 adds a nested `Address` form
(two app-owned inputs writing into the nested
`AddressFormValueHolder`, touched tracking keyed through the nested root
path `address_root()` — composed `address.city`-style paths have no dense
field index in v1) and a tags list with a rendered new-tag editor plus
add/remove buttons over the generated
`add_tags`/`remove_tags`/`tags_items` holder API — per-item `each(...)`
failures render with their failing indices, and the count text follows the
app's localized-count pattern. Milestone M4 adds the field-dependency
showcase: a `state` select recording `depends_on = country` that the country
handler resets (holder value AND widget) on every change, an email field made
conditionally required by the newsletter checkbox through
`required_if = newsletter` (the localized `validation.required` error appears
and disappears live as the checkbox flips), and a "Save changes (touched
only)" button printing the PATCH tri-state filtered to the touched fields —
the plan's D10 outcome: the patch IS the partial payload. Milestone M5 adds a
footer "Schema" debug toggle printing the exported `SignupFormSchema::schema()`
document (component kinds, validations, dependencies, nested/list composition)
— the devtools seed; its tests prove the document reads without the GPUI
runtime. Validation errors render through `validation.*`
`t!` keys — including newtype `Inner` errors — with unit tests asserting the
French/Chinese strings, plus per-crate key-parity tests. EN/FR/中文 buttons
drive the shared `gpui-form-i18n` locale; the footer shows the live locale,
dirty/changed/touched/valid/submitting state, submit attempts, picked paths
count, and the typed field paths.

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
