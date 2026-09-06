#!/usr/bin/env bash
# Creates agent worktrees beside the repo instead of inside it.
#
# The default is <repo>/.claude/worktrees/<name>, which put 38 worktrees and
# 166 GB inside the tree, untracked and matching every grep for a source file.
# The harness sweep never reclaimed them because it keeps any worktree holding
# unpushed commits and this repo had no remote (§50.8).
#
# Must print the final worktree path on stdout and nothing else.
set -euo pipefail

payload=$(cat)
log=${CLAUDE_WORKTREE_HOOK_LOG:-}
[ -n "$log" ] && printf '%s\n' "$payload" >>"$log"

repo=$(git rev-parse --show-toplevel)
name=$(jq -r '.name // .worktree_name // .worktreeName // .id // empty' <<<"$payload")
[ -n "$name" ] || name="agent-$(date +%s)-$$"

base="$(dirname "$repo")/$(basename "$repo")-worktrees"
mkdir -p "$base"
directory="$base/$name"

if [ ! -d "$directory" ]; then
	branch=$(jq -r '.branch // empty' <<<"$payload")
	[ -n "$branch" ] || branch="worktree-$name"
	git -C "$repo" worktree add "$directory" -b "$branch" >&2
fi

# The lock is this repo's record of who owns this worktree, and it is written
# here because here is the only place that knows the agent's name at the moment
# the worktree comes into being. `just lanes` reads it back. Measured on
# 2026-09-06: the harness does **not** lock what it creates — a live
# `isolation: "worktree"` agent's own worktree had no `locked` line — so without
# this nothing records the owner at all.
#
# **Nothing here releases it.** The dispatcher unlocks at merge, and until it
# does, `just prune-worktrees` keeps the worktree rather than reclaiming it.
# That is the safe direction to fail, but it means a lock left behind is
# rubbish that only a person clears — the ISO date in the reason is what tells
# a reader the difference between a live lane and one abandoned days ago.
#
# ASCII only. `git worktree list --porcelain` C-quotes a reason holding any
# non-ASCII byte, so a `·` reaches other readers as `\302\267`.
#
# A worktree that is already locked keeps the reason it has: a re-entry into an
# existing directory is not a new owner.
if ! git -C "$repo" worktree list --porcelain \
	| awk -v want="$directory" '/^worktree /{tree = $2} /^locked/{if (tree == want) found = 1} END{exit !found}'; then
	git -C "$repo" worktree lock \
		--reason "agent $name since $(date -u +%Y-%m-%dT%H:%M:%SZ)" \
		"$directory" >&2
fi

printf '%s\n' "$directory"
