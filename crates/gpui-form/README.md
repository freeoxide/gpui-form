# gpui-form

`gpui-form` is the facade crate of a type-safe form-generation ecosystem for
`gpui` and [`gpui-kit`](https://github.com/longbridge/gpui-kit), centered on
`#[derive(GpuiForm)]`. It re-exports the derive macros plus `core`, `runtime`,
and `schema`, and keeps compatibility re-exports such as `custom`,
`date_picker`, `file_picker`, `infinite_select`, and `numeric`.

The `gpui-form` package on crates.io is the upstream `freeoxide/gpui-form`
lineage, not this fork — install from git to pick up this workspace.

## Compatibility

| `gpui-form` | `gpui-kit` | `gpui` |
| :---------- | :--------- | :----- |
| **git** | | |
| `branch = "master"` | `0.7.0` | `0.3.7` (`gpui-pre`) |

## Installation

```toml
[dependencies]
gpui = { package = "gpui-pre", version = "0.3.7" }
gpui-kit = "0.7.0"
gpui-form = { git = "https://github.com/stayhydated/gpui-form" }
```

## Feature Flags

- `derive` (default): the `GpuiForm`, `SelectItem`, and `CustomComponentState` proc macros.
- `inventory`: register `GpuiFormShape` metadata for prototyping and code generation.
- `mcp`: expose generated forms as MCP tools, resources, and prompts (`gpui_form::mcp`).
- `phone`: parser-backed phone-number validation helpers (`gpui_form::phone`).
- `serde`: `Serialize`/`Deserialize` on generated value holders, for persistence and dirty tracking.

## Documentation

Full documentation — quick start, component syntax, phone and Koruma
validation, localization, form-state persistence, runtime helpers, custom
components, prototyping — lives in the repository
[root `README.md`](../../README.md). Agent-oriented usage guidance is hosted at
[`.agents/skills/use-gpui-form`](../../.agents/skills/use-gpui-form/SKILL.md)
(API map: [`references/api-map.md`](../../.agents/skills/use-gpui-form/references/api-map.md)).
Runnable examples are indexed in [`examples/README.md`](../../examples/README.md).
