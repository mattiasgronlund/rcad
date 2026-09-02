default: check check-enums check-wildcards lint test deny

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
# How many wildcard match arms stand over one of our own enums
# (docs/decisions.md §40). Raise or lower it only with the commit that does.
#
# A count and not a denial, because the lint cannot be turned off in place:
# rustc resolves a lint level at the enclosing expression, so `#[allow]` on a
# match arm never reaches it, and the only granularity left is the whole
# function.
#
# The number moves for two reasons and both want looking at. A new wildcard
# raises it. So does a new *variant*, because a site that names every variant
# there is today starts matching a known one tomorrow — which is the whole point
# of naming them.
wildcard_arms := "79"

check-wildcards:
    @find crates -name lib.rs -path '*/src/*' -exec touch {} +
    @out=$(cargo clippy --workspace --all-targets -- \
            -W clippy::wildcard_enum_match_arm 2>&1) \
        || { echo "$out" | grep '^error' | head -5; \
             echo "clippy did not finish, so this counted nothing"; exit 1; }; \
    found=$(echo "$out" | grep -c '^warning: wildcard match'); \
    if [ "$found" != "{{wildcard_arms}}" ]; then \
        echo "wildcard arms over our own enums: $found, expected {{wildcard_arms}}"; \
        echo "a new one, or a new variant reaching an old one — see docs/decisions.md §40"; \
        exit 1; \
    fi

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

# The Part 21 reader, over arbitrary bytes (docs/decisions.md §42.12 criterion 5).
#
# Same nightly-compiler caveat as `fuzz` above, and the same repeated RUSTFLAGS.
# What runs on every commit instead is
# `crates/rcad-step/tests/what_the_reader_refuses_and_by_what_name.rs`, which
# covers the shapes somebody thought of; this covers the ones nobody did.
#
# No `-max_len`, unlike `fuzz`: the reader's promise is about any length of
# anything, and the truncation cases live at every length there is.
fuzz-reader duration="60":
    cd fuzz && RUSTFLAGS="-C target-cpu=x86-64-v3" \
        cargo +nightly fuzz run no_step_file_makes_the_reader_panic -- \
        -max_total_time={{duration}}

bench:
    cargo bench --workspace

fmt:
    cargo fmt --all
    taplo fmt
