#!/usr/bin/env bash
# Runs the session-start hub notice against repositories it builds itself.
#
# Both halves are worth pinning, and the silent one more than the loud one: a
# hook that spoke up in every lane would be noise in forty sessions a day and
# would be turned off, and a hook that had gone silent everywhere looks exactly
# like a repository where everyone is behaving. So this checks that it speaks in
# the hub, and that it says nothing in a lane, in a lone checkout, or outside a
# repository at all.
set -uo pipefail

here=$(dirname "$(readlink -f "$0")")
hook="$here/session-start-hub.sh"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

repo="$work/rcad"
git init -q -b main "$repo"
git -C "$repo" -c user.email=t@t -c user.name=t commit -q --allow-empty -m root

failed=0
total=0

wrong() {
	printf '%s\n' "$1"
	failed=$((failed + 1))
}

say_from() {
	printf '{"cwd":"%s"}' "$1" | "$hook" 2>/dev/null
}

total=$((total + 1))
out=$(say_from "$repo")
if [ -n "$out" ]; then
	wrong "a lone checkout is not a hub, but the hook spoke: '$out'"
fi

git -C "$repo" worktree add -q "$work/lane" -b lane

total=$((total + 1))
out=$(say_from "$repo")
if [ -z "$out" ]; then
	wrong "a session in the hub of a repository with lanes was told nothing"
elif ! grep -qF "$repo" <<<"$out"; then
	wrong "the notice does not name the hub, so it cannot be acted on: '$out'"
elif ! grep -qF "EnterWorktree" <<<"$out"; then
	wrong "the notice does not say what to do instead: '$out'"
fi

total=$((total + 1))
out=$(say_from "$work/lane")
if [ -n "$out" ]; then
	wrong "a session already in a lane was nagged anyway: '$out'"
fi

# The payload is the harness's, not ours, and a session can start anywhere.
total=$((total + 1))
out=$(say_from "$work")
if [ -n "$out" ]; then
	wrong "a directory outside any repository drew a notice: '$out'"
fi

total=$((total + 1))
out=$(printf '{}' | "$hook" 2>/dev/null)
if [ $? -ne 0 ]; then
	wrong "a payload with no cwd made the hook fail rather than fall back to \$PWD"
fi

if [ "$failed" -gt 0 ]; then
	echo "$failed of $total checks came out wrong"
	exit 1
fi

echo "$total checks, all as expected"
