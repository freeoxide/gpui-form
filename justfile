default:
    @just --list

fmt:
    cargo sort-derives
    cargo fmt
    taplo fmt
    uvx mdformat .

update_crate_paths:
    cargo crate-paths -c gpui -o crates/gpui-form-core/src/implementations/__crate_paths/gpui.rs
    cargo crate-paths -c gpui-component -o crates/gpui-form-core/src/implementations/__crate_paths/gpui_component.rs

clippy:
    cargo clippy --workspace --all-features

check:
    cargo check --workspace --all-features

test:
    cargo test --workspace --all-features

test-publish:
    cargo publish --workspace --dry-run --allow-dirty
