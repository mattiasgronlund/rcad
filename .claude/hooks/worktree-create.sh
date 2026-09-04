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

printf '%s\n' "$directory"
