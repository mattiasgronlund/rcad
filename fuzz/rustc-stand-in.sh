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
set -euo pipefail

real_rustc=$(rustup which --toolchain nightly rustc)

prev=""
for arg in "$@"; do
    if [ "$prev" = "--crate-name" ] && [ "$arg" = "rcad_fuzz_timing" ]; then
        exec "$real_rustc" "$@" -Cllvm-args=-sanitizer-coverage-level=0
    fi
    prev="$arg"
done

exec "$real_rustc" "$@"
