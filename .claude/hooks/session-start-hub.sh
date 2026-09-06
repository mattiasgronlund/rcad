#!/usr/bin/env bash
# Tells a session that started in the hub to go and get a worktree of its own.
#
# The main working tree owns `.git` and is where every session lands by default,
# and it is the one tree `git worktree lock` refuses to describe — so a session
# working there is invisible to `just lanes` and to every other session
# (docs/decisions.md §59). `just check-hub` fails over what gets left behind;
# this says it at the top of the session, while moving still costs nothing.
#
# Silent when the repository has a single worktree, because then it is a
# checkout and not a hub, and silent when the session is already in a lane,
# which is the case this hook wants to be the normal one.
set -uo pipefail

payload=$(cat)
cwd=$(jq -r '.cwd // empty' <<<"$payload" 2>/dev/null || true)
[ -n "$cwd" ] || cwd=$PWD
[ -d "$cwd" ] || exit 0

listing=$(git -C "$cwd" worktree list --porcelain 2>/dev/null) || exit 0
[ -n "$listing" ] || exit 0

here=$(git -C "$cwd" rev-parse --show-toplevel 2>/dev/null) || exit 0
hub=$(awk '/^worktree /{print $2; exit}' <<<"$listing")
trees=$(grep -c '^worktree ' <<<"$listing")

[ "$here" = "$hub" ] || exit 0
[ "$trees" -gt 1 ] || exit 0

echo "This session started in the hub, $hub, and nobody works there. It is the"
echo "main working tree: git refuses to lock it, so nothing can record that you"
echo "are in it and no other session can see you (docs/decisions.md §59)."
echo "Call EnterWorktree before editing anything. \`just check-hub\` fails over"
echo "work left in the hub, and \`just lanes\` will not show you holding it."
