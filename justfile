default:
    @just --list

fmt:
    cargo sort-derives
    cargo fmt
    taplo fmt
    uvx mdformat .

update_crate_paths:
    cargo crate-paths -c gpui -o crates/gpui-form-core/src/implementations/__crate_paths
    cargo crate-paths -c gpui-component -o crates/gpui-form-core/src/implementations/__crate_paths

clippy:
    cargo clippy --workspace --all-features --exclude some-lib-forms

check:
    cargo check --workspace --all-features --exclude some-lib-forms

test:
    cargo test --workspace --all-features

test-publish:
    cargo publish --workspace --dry-run --allow-dirty
