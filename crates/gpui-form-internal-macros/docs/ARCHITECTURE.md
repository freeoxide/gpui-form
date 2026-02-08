# gpui-form-internal-macros Architecture

## Purpose

`gpui-form-internal-macros` provides small derive macros used internally by `gpui-form-core` to reduce boilerplate.

## Key macros

- `ComponentDefinitions`: generates `XComponent` wrapper structs for each variant in the `Components` enum and implements `ComponentDefinition`.
- `ComponentOption`: marks option structs as `ComponentOption`.

## Data flow

`gpui-form-core` derives these macros in `components.rs`, and then the generated wrapper structs are used by `ComponentLayout` implementations to emit field tokens.

## Extension points

If a new component is added, the macros usually require no changes because they derive from the updated enum/structs.
