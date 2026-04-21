default:
    @just --list

fmt:
    cargo sort-derives
    cargo fmt
    taplo fmt
    uvx mdformat .

clippy:
    cargo clippy --workspace --all-features --exclude some-lib-forms

check:
    cargo check --workspace --all-features --exclude some-lib-forms

test:
    cargo test --workspace --all-features

test-publish:
    cargo publish --workspace --dry-run --allow-dirty

update_crate_paths:
    cargo crate-paths -c gpui -o __crate_paths__
    cargo crate-paths -c gpui-component -o __crate_paths__
    cargo crate-paths -c gpui-form -o __crate_paths__
