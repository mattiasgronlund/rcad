default: check lint test deny

check:
    cargo check --workspace --all-targets

test:
    cargo nextest run --workspace
    cargo test --workspace --doc

lint:
    cargo fmt --all -- --check
    taplo fmt --check
    cargo clippy --workspace --all-targets -- -D warnings

deny:
    cargo deny check

bench:
    cargo bench --workspace

fmt:
    cargo fmt --all
    taplo fmt
