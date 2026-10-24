#!/usr/bin/env bash
# Set as RUSTC while building a fuzz target. cargo-fuzz hands every crate in
# the build the same coverage flags through RUSTFLAGS, and there is no
# allow-list or deny-list on the sanitizer-coverage pass itself
# (docs/decisions.d/267): the only working way to leave one crate
# uninstrumented is to catch its compile here and append a second
# `-sanitizer-coverage-level`, which wins only because it lands after
# cargo-fuzz's own `-Cllvm-args=-sanitizer-coverage-level=4` on the same
# command line.
#
# `rcad-fuzz-timing` is the one crate this is for: its own code must not be
# what steers libFuzzer (docs/decisions.d/267.1-267.3).
#
# `inari` is the other: its internal fast/slow dispatch for tiny products
# gives the same bits either way, so instrumenting it buys libFuzzer nothing
# and the ASan build pays for every one of those lanes
# (docs/decisions.d/465.5, 465.9). Debug assertions and overflow checks go
# too, since the fork has none that rcad's own fuzzing wants on: the kernel's
# own arithmetic carries its own checks, in `Interval` and the predicates, not
# in `inari`.
set -euo pipefail

real_rustc=$(rustup which --toolchain nightly rustc)

prev=""
for arg in "$@"; do
    if [ "$prev" = "--crate-name" ] && [ "$arg" = "rcad_fuzz_timing" ]; then
        exec "$real_rustc" "$@" -Cllvm-args=-sanitizer-coverage-level=0
    fi
    if [ "$prev" = "--crate-name" ] && [ "$arg" = "inari" ]; then
        exec "$real_rustc" "$@" -Cllvm-args=-sanitizer-coverage-level=0 \
            -Cdebug-assertions=off -Coverflow-checks=off
    fi
    prev="$arg"
done

exec "$real_rustc" "$@"
