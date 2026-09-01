default: check check-enums lint test deny

check:
    cargo check --workspace --all-targets

# Every public enum either grows or says why it does not (docs/decisions.md §40).
#
# `#[non_exhaustive]`, or a doc comment saying "Exhaustive on purpose" — which
# `Orientation` is, and is the only one (§3.6). A new public enum written
# without either compiles, passes every test, and nothing else would notice.
#
# A `pub enum` is judged against what stands between it and the blank line above
# it, which is where rustfmt puts the item's own attributes and doc comment.
check-enums:
    @! awk '/^$/ {ok = 0} \
            /^#\[non_exhaustive\]/ {ok = 1} \
            /Exhaustive on purpose/ {ok = 1} \
            /^[ \t]*pub enum/ {if (!ok) print FILENAME ":" FNR ": " $0; ok = 0}' \
            $(find crates -path '*/src/*' -name '*.rs') | grep . \
        || (echo "^ a public enum that neither grows nor says why not (docs/decisions.md §40)"; exit 1)

test:
    cargo nextest run --workspace
    cargo test --workspace --doc

lint:
    cargo fmt --all -- --check
    taplo fmt --check
    cargo clippy --workspace --all-targets -- -D warnings

deny:
    cargo deny check

# The property fuzzer, over a pair of surfaces (docs/decisions.md §24.14).
#
# NOT part of `just default`, and not because anybody chose to leave it out:
# `cargo-fuzz` builds with a sanitizer that needs a nightly compiler and
# `rust-toolchain.toml` pins a stable one, so this is the one recipe here that
# uses a compiler the repo does not pin. What runs on every commit is the same
# property over the same decoder, in
# `crates/rcad-geom/tests/every_pair_of_surfaces_the_fuzzer_draws_is_named_honestly.rs`.
#
# RUSTFLAGS has to be repeated: cargo-fuzz sets it as an environment variable,
# and that shadows the `.cargo/config.toml` entry `inari` needs to build at all.
fuzz duration="60":
    cd fuzz && RUSTFLAGS="-C target-cpu=x86-64-v3" \
        cargo +nightly fuzz run where_two_surfaces_cross -- \
        -max_total_time={{duration}} -max_len=71

bench:
    cargo bench --workspace

fmt:
    cargo fmt --all
    taplo fmt
