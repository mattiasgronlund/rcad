#!/usr/bin/env bash
# Makes the directories cargo copies its finished binaries into links to the
# zram disk, so the copy out of the build directory is a hard link and not tens
# of megabytes written to the SSD on every link (docs/decisions.d/476).
#
#   link-targets.sh <tree>
#
# `target/` itself stays a real directory on disk: the tooling keeps what it
# writes there, and a reboot empties the zram disk (476.2). Only the root
# workspace's `target/debug`, `target/release` and `target/ci` go, and the fuzz
# workspace's `fuzz/target/x86_64-unknown-linux-gnu/release`. 476.5 says how
# that list was read off a build. `target/ci` is for solidcraft's `ci` profile,
# which rcad has none of, so in an rcad tree it links an empty directory
# (476.7). The fuzz workspace builds on the nightly toolchain, so it is asked
# with `cargo +nightly metadata`.
#
# The links go under `<artifacts>/<xx>/<hash>`, the last two parts of the build
# directory `cargo metadata` names, so the key pairs with `<targets>/<xx>/<hash>`
# the way inari's whole-`target` links do, and one boot step makes the
# destinations of both (476.3). A tree whose build directory is not
# `<targets>/<xx>/<hash>` is left alone: with build-dir unset, as on CI, cargo
# builds into `target/` and a link would only move the build's copies away from
# it.
#
# A directory that is already a real directory is reported and kept;
# `just clean-targets` replaces it once the tree is idle, under its rules for
# idle trees (476.6). A link whose destination is missing, as after every boot,
# gets its destination made again.
#
# Called by the post-checkout hook beside this file on `git worktree add`, and
# by `just link-targets-to-zram`. The roots come from the environment so the
# selftest can point both at a scratch directory.
#
# Prints one line per directory to stdout. Exits 1 only when it could not make
# a link or a destination it should have; a tree it skips is not a failure.
set -uo pipefail

tree=${1:?usage: link-targets.sh <tree>}
targets=${RCAD_ZRAM_TARGETS:-/var/cache/zram/targets}
artifacts=${RCAD_ZRAM_ARTIFACTS:-/var/cache/zram/artifacts}

failed=0

link_workspace() {
    local where=$1
    shift
    local toolchain=()
    case "${1:-}" in +*) toolchain=("$1"); shift ;; esac
    [ -f "$where/Cargo.toml" ] || { echo "skipped $where: no Cargo.toml"; return; }
    local metadata
    # The rustup proxy installs a missing pinned toolchain before it runs
    # anything, which on a hook would be a download inside `git worktree add`.
    # So no install, and a bound on the wait.
    metadata=$(cd "$where" && RUSTUP_AUTO_INSTALL=0 timeout 20 cargo "${toolchain[@]}" metadata \
        --format-version 1 --no-deps --offline 2>/dev/null) || {
        echo "skipped $where: cargo metadata failed, so its build directory is not known"
        return
    }
    local dirs
    dirs=$(python3 -I -c 'import json, sys
m = json.load(sys.stdin)
print(m["build_directory"])
print(m["target_directory"])' <<<"$metadata") || {
        echo "skipped $where: cargo metadata named no build_directory and target_directory"
        return
    }
    local build target
    build=$(sed -n 1p <<<"$dirs")
    target=$(sed -n 2p <<<"$dirs")
    local key=${build#"$targets"/}
    if [ "$key" = "$build" ] || ! grep -qxE '[0-9a-f]{2}/[0-9a-f]+' <<<"$key"; then
        echo "skipped $where: its build directory $build is not one under $targets"
        return
    fi
    if [ -L "$target" ] || { [ -e "$target" ] && [ ! -d "$target" ]; }; then
        echo "skipped $where: $target is not a real directory"
        return
    fi
    local name
    for name in "$@"; do
        link_one "$target/$name" "$artifacts/$key/$name"
    done
}

link_one() {
    local path=$1 dest=$2
    if [ -L "$path" ]; then
        local text
        text=$(readlink "$path")
        if [ "$text" != "$dest" ]; then
            echo "kept (a link to $text, not to $dest): $path"
            return
        fi
        if [ -d "$dest" ]; then
            echo "linked already: $path -> $dest"
            return
        fi
        if mkdir -p "$dest"; then
            echo "made its missing destination again: $path -> $dest"
        else
            echo "could not make the destination of $path: $dest" >&2
            failed=1
        fi
        return
    fi
    if [ -e "$path" ]; then
        echo "kept (a real directory, which Q3070 replaces in an idle tree): $path"
        return
    fi
    if ! mkdir -p "$dest"; then
        echo "could not make $dest, so $path is not linked" >&2
        failed=1
        return
    fi
    # -T, so a directory that appears at $path in between is not linked into.
    if mkdir -p "$(dirname "$path")" && ln -sT "$dest" "$path" 2>/dev/null; then
        echo "linked: $path -> $dest"
    elif [ "$(readlink "$path" 2>/dev/null)" = "$dest" ]; then
        echo "linked already: $path -> $dest"
    else
        echo "could not link $path to $dest" >&2
        failed=1
    fi
}

link_workspace "$tree" debug release ci
link_workspace "$tree/fuzz" +nightly x86_64-unknown-linux-gnu/release
exit "$failed"
