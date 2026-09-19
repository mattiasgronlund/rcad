#!/usr/bin/env bash
# Re-parks the hub on `main` when it is parked behind it, so a `just` command
# typed in the hub reads today's files and not the commit the hub stopped at.
#
# Nobody works in the hub (docs/decisions.md §60), so nothing ever moves its
# HEAD — §60.9 says so in as many words. The hub therefore drifts, silently:
# its files match its own old commit, `git status` is empty, and `check-hub`
# reports "parked and clean" and passes. A reader who types `just decision
# 144.5` there is told the section does not exist, which is true of the file in
# front of them and false of the repository (§146).
#
# Called from the `justfile`'s `hub_aligned` assignment, which `just` evaluates
# before any recipe runs, and handed the directory the justfile was found in.
#
# Three things this deliberately does not do:
#
#   * It does not fetch. This runs before every recipe, and a network round trip
#     on `just t` would be the cost §144 is about, paid in latency. `main` is
#     the branch this repo lands on locally and then pushes, so it is never
#     behind `origin/main` here — and if it ever is, that is a push somebody
#     owes, not something a re-park should paper over.
#   * It does not touch a hub with a branch checked out. That is `check-hub`'s
#     failure and it names the fix; detaching it here would move somebody off a
#     branch without saying so.
#   * It does not move a hub holding anything that is not already on `main`,
#     byte for byte. Stale and leftovers look identical to `status` and want
#     opposite fixes (§60.9, §60.10), and the fix for stale is `clean -fd`.
#
# Always exits 0. It runs ahead of every recipe in the file, so a failure here
# must never be the reason `just test` did not run.
set -uo pipefail

dir=${1:-.}
[ -d "$dir" ] || exit 0

listing=$(git -C "$dir" worktree list --porcelain 2>/dev/null) || exit 0
[ -n "$listing" ] || exit 0

here=$(git -C "$dir" rev-parse --show-toplevel 2>/dev/null) || exit 0
hub=$(awk '/^worktree /{print $2; exit}' <<<"$listing")
trees=$(grep -c '^worktree ' <<<"$listing")

# The same two questions `session-start-hub.sh` asks, in the same order: is this
# the main working tree, and is it a hub at all. A lone checkout is a checkout.
[ "$here" = "$hub" ] || exit 0
[ "$trees" -gt 1 ] || exit 0

git -C "$hub" symbolic-ref --quiet HEAD >/dev/null 2>&1 && exit 0

target=$(git -C "$hub" rev-parse --verify --quiet main) || exit 0
[ "$(git -C "$hub" rev-parse HEAD)" != "$target" ] || exit 0

dirt=$(git -C "$hub" status --porcelain)
unsafe=""
if [ -n "$dirt" ]; then
    if grep -qv '^??' <<<"$dirt"; then
        unsafe="a modified file"
    else
        # Every untracked path has to be on `main` with the same bytes, not
        # merely the same name: the re-park below deletes them, and a name-only
        # answer deletes content that exists nowhere else (§60.10).
        while IFS= read -r -d '' path; do
            on_main=$(git -C "$hub" rev-parse --verify --quiet "main:$path")
            if [ -n "$on_main" ] \
                && [ "$on_main" = "$(git -C "$hub" hash-object -- "$hub/$path")" ]; then
                continue
            fi
            unsafe="$path"
            break
        done < <(git -C "$hub" ls-files --others --exclude-standard -z)
    fi
fi

if [ -n "$unsafe" ]; then
    echo "hub-align: $hub is parked behind main, and holds work main does not: $unsafe." >&2
    echo "hub-align: it was left where it is. \`just check-hub\` there tells stale from" >&2
    echo "hub-align: leftovers and names the fix; this refuses to guess between them." >&2
    exit 0
fi

# `clean` first is not optional and not tidiness: checkout refuses to overwrite
# an untracked file even when its content already matches the commit being moved
# to, so the removal has to come first and the checkout restores every one of
# them byte-identical (§60.9). Without `-x`, ignored files — a fuzz campaign's
# artifacts among them — are left alone.
was=$(git -C "$hub" rev-parse --short HEAD)
justfile_before=$(git -C "$hub" hash-object -- "$hub/justfile" 2>/dev/null)
git -C "$hub" clean -fd >/dev/null 2>&1 || exit 0
git -C "$hub" checkout --detach main >/dev/null 2>&1 || exit 0
justfile_after=$(git -C "$hub" hash-object -- "$hub/justfile" 2>/dev/null)

echo "hub-align: the hub was parked at $was, behind main; re-parked it detached on" >&2
echo "hub-align: $(git -C "$hub" rev-parse --short HEAD). Nothing else moves it (§60.9)." >&2

# The justfile `just` parsed for this run is the one that was on disk when it
# started, so a run that changed it is running yesterday's recipes against
# today's files. Saying so is cheaper than the confusion of a recipe that is
# missing, or present and behaving differently from the one in the tree.
if [ "$justfile_before" != "$justfile_after" ]; then
    echo "hub-align: the justfile moved with it, and this command already parsed the" >&2
    echo "hub-align: old one — run it again to use the recipes that are now there." >&2
fi

exit 0
