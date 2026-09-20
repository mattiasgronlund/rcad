#!/usr/bin/env bash
# Runs the worktree-create hook against a repository it builds itself and checks
# the two things it promises: the worktree lands beside the repo rather than
# inside it, and it comes back locked with a reason naming the agent.
#
# The lock half exists because the harness does not do it. Measured on
# 2026-09-06: a live `isolation: "worktree"` agent's own worktree had no `locked`
# line, so if this hook stops writing one, nothing records who owns a worktree
# and `just lanes` goes quiet while lanes are running.
set -uo pipefail

here=$(dirname "$(readlink -f "$0")")
hook="$here/worktree-create.sh"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

repo="$work/rcad"
git init -q -b main "$repo"
git -C "$repo" -c user.email=t@t -c user.name=t commit -q --allow-empty -m root

# The hook parses its payload with `jq`, so without one it produces no path, no
# worktree and no lock — three of the five checks below failing at once, each
# describing a symptom and none of them the cause. That is what a fresh
# container said, for as long as it took to go and look. `mise.toml` pins jq for
# exactly this; say so here rather than let it be diagnosed again.
# `docs/decisions.md` §173.7.
if ! command -v jq >/dev/null; then
	echo "worktree-create.sh needs jq and there is none on PATH:" >&2
	echo "  mise.toml pins it, so this is a shell that did not go through mise" >&2
	exit 1
fi

failed=0
total=0

wrong() {
	printf '%s\n' "$1"
	failed=$((failed + 1))
}

reason_for() {
	local tree=$1
	local note
	note="$(git -C "$tree" rev-parse --git-dir)/locked"
	[ -s "$note" ] && cat "$note"
}

total=$((total + 1))
# Kept off stdout, which is the answer being checked, but kept: discarding it
# is why a missing `jq` reached a reader as three symptoms and no cause.
out=$(cd "$repo" && printf '{"name":"agent-abc"}' | "$hook" 2>"$work/hook.err")
want="$work/rcad-worktrees/agent-abc"
if [ "$out" != "$want" ]; then
	wrong "the path on stdout should be exactly '$want', got '$out'"
fi

total=$((total + 1))
if [ ! -d "$want" ]; then
	wrong "no worktree was created at $want"
elif [ -d "$repo/.claude/worktrees" ]; then
	wrong "a worktree was put inside the tree, which is what §50.8 filled with 166 GB"
fi

total=$((total + 1))
held=$(reason_for "$want")
if [ -z "$held" ]; then
	wrong "the new worktree came back with no lock, so nothing records who owns it"
elif ! grep -q 'agent-abc' <<<"$held"; then
	wrong "the lock reason does not name the agent: '$held'"
elif ! grep -qE '[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z' <<<"$held"; then
	wrong "the lock reason carries no ISO date, so a stale lock cannot be told from a live one: '$held'"
fi

# A reason holding a non-ASCII byte reaches every porcelain reader C-quoted, as
# `\302\267` for a `·`. `just lanes` reads the lock file and so is unaffected,
# but `prune-worktrees` matches on the listing.
#
# `tr` and not `grep -qP`: `-P` is GNU grep's, and a grep without it exits
# non-zero on the option rather than on the search — which reads here as "no
# non-ASCII found" and passes the check by accident. The same shape as §173's
# `\<match\>`, in the same file as its fix, found while writing it.
total=$((total + 1))
if [ -n "$(printf '%s' "$held" | LC_ALL=C tr -d '\000-\177')" ]; then
	wrong "the lock reason is not ASCII, so porcelain readers see it escaped: '$held'"
fi

# Re-entering an existing directory is not a new owner, and overwriting the
# reason would erase the name of whoever is actually working in there.
total=$((total + 1))
sleep 1
(cd "$repo" && printf '{"name":"agent-abc"}' | "$hook" >/dev/null 2>&1)
again=$(reason_for "$want")
if [ "$again" != "$held" ]; then
	wrong "a second call rewrote the lock reason: '$held' became '$again'"
fi

if [ "$failed" -gt 0 ]; then
	echo "$failed of $total checks came out wrong"
	if [ -s "$work/hook.err" ]; then
		echo "--- what the hook said on stderr ---"
		cat "$work/hook.err"
	fi
	exit 1
fi

echo "$total checks, all as expected"
