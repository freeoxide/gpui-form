# Publishing the workspace crates

How the nine `freeoxide-gpui-form*` crates land on crates.io, through the
**Publish crates** workflow (`.github/workflows/publish.yml`). The example
crates (`some-lib`, `forms-demo`, `prototyping`) set `publish = false` and
are never uploaded.

## When to dispatch

Dispatch manually: Actions → **Publish crates** → Run workflow. Nothing
publishes on push; the workflow has no push tag.

Prerequisites, checked by the workflow before anything uploads:

1. `version` in the root `Cargo.toml` `[workspace.package]` is the version
   being released — all nine crates inherit it, so one number releases at
   a time (the lockstep gate fails the run if they ever disagree).
2. Every intra-set dependency requirement is exactly `^<version>` (they
   are, via the `[workspace.dependencies]` table — only a hand-edited
   manifest could break this; the gate fails the run up front rather than
   mid-publish).
3. The root `CHANGELOG.md` has a `## [<version>]` heading. This workspace
   keeps ONE changelog for all crates, so every new/update crate is gated
   on the same heading; move the entry out of `[Unreleased]` before
   dispatching. A crate still under `[Unreleased]` fails the preflight
   naming the crate, before anything uploads.

`CARGO_REGISTRY_TOKEN` must exist as a repository secret.

## How dry-run works

`dry_run` is a checkbox **defaulting to true**. In a dry run every crate
is verified with `cargo publish --dry-run` (packaging + verification
build, no upload) and recorded as `would-publish-new` /
`would-publish-update`. The one downgraded failure is
`dry-run-order-blocked`: an intra-set dependency that is not yet on
crates.io at the required version ("no matching package named
`freeoxide-gpui-form-*`" or "failed to select a version"). That is the
expected state before the first real publish — the earlier crates in the
set cannot resolve their intra-set deps until those are uploaded — so it
is a warning, not a failure. Any other dry-run failure fails the run.

A dry run is the right first dispatch for every release: it exercises the
gates, the registry preflight, and the packaging of all nine crates with
zero risk. Uncheck `dry_run` only when it is green.

## The release set

`PUBLISH_SET` in the workflow env, dependency-first (cargo publish
resolves every intra-set dependency against the live registry):

1. `freeoxide-gpui-form-schema` — no intra-set deps
2. `freeoxide-gpui-form-core` — no intra-set deps
3. `freeoxide-gpui-form-i18n` — no intra-set deps
4. `freeoxide-gpui-form-component-derive` — no intra-set deps
5. `freeoxide-gpui-form-codegen` (schema)
6. `freeoxide-gpui-form-component` (component-derive, i18n)
7. `freeoxide-gpui-form-derive` (codegen)
8. `freeoxide-gpui-form` (component, core, derive, schema)
9. `freeoxide-gpui-form-prototyping-core` (schema)

The workflow's python gate re-derives this from `cargo metadata` on every
run and fails on any membership, order, or version-lockstep mistake.

One nuance the gate encodes: intra-set **dev**-dependencies that are
path-only (no `version`, e.g. the facade dev-dep used by the component
crate's tests) are dropped from the packaged manifest by cargo and never
resolve against the registry — they neither order-constrain nor need
`^version`. A dev-dependency that *does* carry a version would resolve
against crates.io at publish time; the component↔facade dev cycle is
exactly why those dev-deps are path-only in this workspace. Keep them
that way.

## What a real run does

Per crate, in order, after fmt/clippy/test gates over the set and the
full workspace test suite:

- **Preflight** against the sparse index (cross-checked with the
  crates.io API): already at this version → **skip**; older only →
  **update**; absent → **new**; a *newer* registry version → refuse and
  abort before the first upload (lockstep regression). New/update crates
  must pass the changelog-heading gate.
- **Publish**: package (`cargo publish --dry-run`), upload, then wait
  until the sparse index resolves the new version before the next crate
  starts.
- **Failures**: auth (401/403/credential) aborts the whole run pointing
  at `CARGO_REGISTRY_TOKEN`; HTTP 429 sleeps past the server's
  "try again after" timestamp (plus margin) and retries the same crate,
  up to five attempts; transient errors (5xx, network, timeouts) retry
  three times with backoff. A crate that still fails stops the run — no
  later crate is attempted.
- **Tags**: every crate that reached the registry this run (published or
  skipped) gets a `<crate>-v<version>` tag created and pushed, unless it
  already exists.
- **Summary**: per-crate dispositions, anything not attempted, and
  counts. The job fails if anything has a `failed:*` disposition or was
  never attempted.

Re-dispatching after any failure resumes where the run stopped: the plan
is rebuilt from the live registry, already-published crates skip, and the
concurrency group (`publish-crates`) serializes runs.

## Rolling back a bad version

No unpublish exists; the rollback is a manual yank on purpose:

```console
$ cargo yank freeoxide-gpui-form@0.10.0
```

A yanked version keeps working for existing lockfiles but cannot be newly
resolved, and the number can never be re-uploaded — the fix is always
bump, changelog entry, dispatch.

## Verifying locally

- `bash scripts/test-publish-state.sh` — the classification state
  machine's unit tests, no network needed.
- `cargo package -p freeoxide-gpui-form-schema --list --allow-dirty` —
  what ships in a leaf crate's tarball (crates with intra-set deps cannot
  package until those are published).
- `cargo publish --dry-run -p freeoxide-gpui-form-schema --allow-dirty` —
  full packaging verification for a leaf crate, no upload.
