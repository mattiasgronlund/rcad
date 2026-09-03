default: check check-enums check-wildcards check-wildcards-selftest check-ignored lint test deny

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
# How many wildcard match arms stand over one of our own enums, or over an
# array, a tuple, or a reference of them (docs/decisions.md §40). Raise or
# lower it only with the commit that does.
#
# Two mechanisms sit behind this one number, because they see different
# things. `clippy::wildcard_enum_match_arm` asks only what the scrutinee's
# own type is, so a match over `Surface` is seen and a match over
# `[Surface; 2]` or `(&Surface, &Surface)` is not — the scrutinee there is an
# array or a tuple, and clippy asks nothing about what is nested inside one.
# Two real defects hid in exactly that gap this milestone: a panic reachable
# from a real KiCad file through `Import::read`, and a fourth list of
# surface pairs that had drifted out of step with the other three. Both
# matched over a composite rather than over one enum.
#
# `crates/rcad-step/tools/composite_wildcards.awk` is the second mechanism: a
# line-based scan for an arm that names one of our enum's variants inside a
# `[`, a `(`, or a `&`, standing in the same match as a bare `_`. It counts how
# many of that enum's own variants the match never names anywhere in its arms
# — not the match itself — so a fully-named composite match, the same as a
# fully-named single-enum one, counts zero. The proper mechanism for this is
# rustc's own `non_exhaustive_omitted_patterns`, unstable on the 1.96 this repo
# pins; the script's own header comment says so first.
#
# `check-wildcards-selftest`, below, pins two small fixtures the scan must
# catch and two it must not, against a made-up enum rather than one of ours —
# so the scan keeps being proven even on a tree where nothing real trips it.
#
# A count and not a denial: rustc resolves a lint level at the enclosing
# expression, so `#[allow]` on a match arm never reaches it and the only
# granularity clippy leaves is the whole function; the text scan has no
# per-site silencing at all.
#
# The number moves for three reasons and all three want looking at. A new
# wildcard raises it, of either kind. So does a new *variant*, because a site
# that names every variant there is today starts matching a known one
# tomorrow — which is the whole point of naming them. That used to be true of
# the clippy half only; §43.8 item 8 found the composite half counted sites
# and not variants, so a fourth surface-pair list could drift out of step with
# the other three and this number would never say so. It counts uncovered
# variants now, and moves with a new one on the composite side too.
#
# It falls when a refusal becomes an answer, which is the fourth reason and the
# happiest one: a test that matched an `Error` it expected, with `other =>
# panic!` for the rest, stops matching anything once the case it named is
# answered. That is what took it from 88 to 87 — a block corner on a ball's
# skin, which the mesher used to refuse.
#
# **A fifth reason, and the one that makes this number not a ratchet: the
# instrument changed.** 90 → 87 over the unit called "the door" looks like three
# arms going away and is not. Measured both ways on both trees:
#
#     old awk, before   82 direct + 8 composite = 90
#     new awk, before   82 direct + 6 composite = 88
#     old awk, after    81 direct + 8 composite = 89
#     new awk, after    81 direct + 6 composite = 87
#
# So the composite half moved **entirely** because the scan started counting
# uncovered variants instead of sites, and not at all because the tree did. Only
# one point of the three — the direct half's 82 → 81 — is a real arm going away.
# A ratchet that resets its own instrument in the same commit range as the tree
# has lost its baseline, so the bridging figures are written here rather than
# left to be re-derived. Whoever changes the scan again owes the same four rows.
wildcard_arms := "87"

check-wildcards:
    @find crates -name lib.rs -path '*/src/*' -exec touch {} +
    @out=$(cargo clippy --workspace --all-targets -- \
            -W clippy::wildcard_enum_match_arm 2>&1) \
        || { echo "$out" | grep '^error' | head -5; \
             echo "clippy did not finish, so this counted nothing"; exit 1; }; \
    direct=$(echo "$out" | grep -c '^warning: wildcard match'); \
    files=$(find crates -name '*.rs' \( -path '*/src/*' -o -path '*/tests/*' \
            -o -path '*/benches/*' -o -path '*/examples/*' \)); \
    enums=$(grep -hoE '^[ \t]*pub enum [A-Za-z0-9_]+' $(find crates -path '*/src/*' -name '*.rs') \
            | awk '{print $NF}' | sort -u | paste -sd'|'); \
    composite=$(awk -v enums="$enums" -f crates/rcad-step/tools/composite_wildcards.awk $files \
            | grep '^COUNT=' | cut -d= -f2); \
    found=$((direct + composite)); \
    if [ "$found" != "{{wildcard_arms}}" ]; then \
        echo "wildcard arms over our own enums: $found ($direct direct, $composite composite), expected {{wildcard_arms}}"; \
        echo "a new one, or a new variant reaching an old one — see docs/decisions.md §40"; \
        exit 1; \
    fi

# Pins the composite-wildcard scan against fixtures it does not read off the
# tree, so it stays proven even on a tree where nothing real trips it.
#
# `Toy` is not one of our enums — the fixtures pass it to the script by name
# so the test is about the *script*, not about what today's `crates` happen
# to hold, and `toy_enum.rs` gives it a two-variant body of its own so the
# scan has a real total to judge the others against. `commented.rs` pins the
# bug the script had while this recipe was being written: a doc comment that
# says the word "match" opened a match this script never closed, and every
# line after it stopped being read. Every fixture below names only `Toy::A`
# and never `Toy::B`, so each is one variant short — 1 apiece, 3 in all — not
# because three sites were found, but because three fixtures each leave one
# of `Toy`'s two variants uncovered.
check-wildcards-selftest:
    #!/usr/bin/env bash
    set -euo pipefail
    dir=$(mktemp -d)
    trap 'rm -rf "$dir"' EXIT
    cat > "$dir/bad_array.rs" <<'RS'
    fn f(pair: [Toy; 2]) -> bool {
        match pair {
            [Toy::A, Toy::A] => true,
            _ => false,
        }
    }
    RS
    cat > "$dir/bad_tuple_ref.rs" <<'RS'
    fn f(one: &Toy, other: &Toy) -> bool {
        match (one, other) {
            (&Toy::A, &Toy::A) => true,
            _ => false,
        }
    }
    RS
    cat > "$dir/good_named.rs" <<'RS'
    fn f(pair: [Toy; 2]) -> bool {
        match pair {
            [Toy::A, Toy::A] => true,
            [Toy::A, Toy::B] | [Toy::B, Toy::A] | [Toy::B, Toy::B] => false,
        }
    }
    RS
    cat > "$dir/good_single_enum_wildcard.rs" <<'RS'
    fn f(one: Toy) -> bool {
        match one {
            Toy::A => true,
            _ => false,
        }
    }
    RS
    cat > "$dir/commented.rs" <<'RS'
    // Whatever the caller does, match it against Toy::A here.
    fn f(pair: [Toy; 2]) -> bool {
        match pair {
            [Toy::A, Toy::A] => true,
            _ => false,
        }
    }
    RS
    cat > "$dir/toy_enum.rs" <<'RS'
    pub enum Toy {
        A,
        B,
    }
    RS
    got=$(awk -v enums="Toy" -f crates/rcad-step/tools/composite_wildcards.awk \
        "$dir/bad_array.rs" "$dir/bad_tuple_ref.rs" "$dir/good_named.rs" \
        "$dir/good_single_enum_wildcard.rs" "$dir/commented.rs" "$dir/toy_enum.rs" | sort)
    want=$(printf '%s\n' \
        "$dir/bad_array.rs:2" \
        "$dir/bad_tuple_ref.rs:2" \
        "$dir/commented.rs:3" \
        "COUNT=3" | sort)
    if [ "$got" != "$want" ]; then
        echo "composite_wildcards.awk drifted from its own fixtures:"
        echo "--- got ---"; echo "$got"
        echo "--- want ---"; echo "$want"
        exit 1
    fi

# How many tests are marked `#[ignore]` today. Move it with the commit that
# moves the tree, the same way `wildcard_arms` above works.
#
# Zero is a legitimate answer, and it is where this should end up: it says the
# last gap somebody parked behind an `#[ignore]` has been closed, and
# `test-ignored` then has nothing to run and says so.
ignored_tests := "4"

# An `#[ignore]`d test is a gap somebody parked, and this is what stops it being
# forgotten.
#
# Nothing ran the ignored tests at all before this: `cargo nextest run
# --workspace` skips them silently, so the one `#[ignore]` in the tree could be
# fixed and un-ignored, or a second one added, and no recipe here would say a
# word either way.
#
# **The count is pinned rather than the run being left to speak for itself**,
# because a run that selects nothing is the failure this recipe exists to
# prevent and an empty selection is exactly what a passing run looks like. A
# typo in the filter, a flag this nextest does not have, a crate that did not
# build — each of those selects nothing, and `cargo nextest run --run-ignored
# ignored-only` printing "0 tests run" would be read as good news. A number that
# has to match says which of the two happened.
#
# `test-ignored` below is what actually runs them, and it is not part of
# `default`: today's one ignored test **fails**, which is why it is ignored, so
# a `default` that ran it would be red on purpose.
check-ignored:
    @out=$(cargo nextest list --workspace --run-ignored ignored-only 2>/dev/null) \
        || { echo "nextest could not list the ignored tests, so this counted nothing"; exit 1; }; \
    listed=$(echo "$out" | grep -c .); \
    if [ "$listed" != "{{ignored_tests}}" ]; then \
        echo "$out"; \
        echo "^ tests marked #[ignore]: $listed, expected {{ignored_tests}}"; \
        echo "a new one raises this number; one fixed and un-ignored lowers it"; \
        exit 1; \
    fi

# Run them. Fails loudly if the selection is empty rather than reporting a run
# of no tests as a pass.
test-ignored:
    @if [ "{{ignored_tests}}" = "0" ]; then \
        echo "no test is marked #[ignore], so there is nothing here to run"; \
        exit 0; \
    fi; \
    cargo nextest run --workspace --run-ignored ignored-only --no-tests fail

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

# The Part 21 writer, over an arbitrary `Model` (docs/decisions.md §42.12).
#
# Same nightly-compiler caveat as `fuzz` above. `Model` is `Deserialize`
# (`crates/rcad-topo/src/serialize.rs`), so this target draws bytes through
# `postcard::from_bytes` — the same route a stranger's saved file takes — and
# whatever `Model` comes out, sound or not, goes straight into
# `rcad_step::write`, which owes an `Err` and never a panic either way.
fuzz-writer duration="60":
    cd fuzz && RUSTFLAGS="-C target-cpu=x86-64-v3" \
        cargo +nightly fuzz run no_model_makes_the_writer_panic -- \
        -max_total_time={{duration}}

# The fuse, over two arbitrary `Model`s (docs/decisions.md §17).
#
# Same nightly-compiler caveat as `fuzz` above. Decodes two `Model`s per run
# and, for each that passes `Model::validate`, fuses a byte-chosen split of
# its own solids against each other — see
# `fuzz/fuzz_targets/two_solids_fused_by_a_stranger.rs` for why two `Model`s
# and not one cross-fused pair.
fuzz-fuse duration="60":
    cd fuzz && RUSTFLAGS="-C target-cpu=x86-64-v3" \
        cargo +nightly fuzz run two_solids_fused_by_a_stranger -- \
        -max_total_time={{duration}}

bench:
    cargo bench --workspace

fmt:
    cargo fmt --all
    taplo fmt
