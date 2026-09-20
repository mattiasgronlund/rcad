#!/usr/bin/env bash
# Runs `just check-where-the-work-is` right after a write to §0's own file, so
# the 50-line cap, the `reviewed:` date and the citation walk are answered in
# the same turn as the write instead of at the next `just gate`.
#
# §152's check-question-file.sh did this first, for `open-questions.md`;
# docs/decisions.md §178 measured why §0 wants the same shape: 34 refusals over
# four days, 21 trim episodes, and 11 of those found only by a whole `just
# gate` — a 98-second median run and a whole-context call — rather than by a
# check that costs a fraction of a second.
#
# It runs the public recipe rather than a second copy of the awk that decides
# the cap: two copies of a rule kept in step by a comment is the drift this
# repo keeps ending (§178.3), and §173 is what it costs when one copy reads
# the environment differently.
#
# Matches only §0's own file, not every file under docs/decisions.d/. §178.6
# asked whether a write to any section should speak, since the citation arm
# can be broken by a rename elsewhere; docs/decisions.md §178.7 has the
# argument for keeping it narrow.
set -uo pipefail

payload=$(cat)
file=$(jq -r '.tool_input.file_path // empty' <<<"$payload")

case "$file" in
*docs/decisions.d/000-where-the-work-is.md) ;;
*) exit 0 ;;
esac

# A hook that cannot find what it runs looks exactly like a clean file, which
# is worse than the round trip this hook exists to remove — so a resolution
# failure is reported, not swallowed. Same shape as check-question-file.sh's
# "checker not found" case.
repo=$(git -C "$(dirname "$file")" rev-parse --show-toplevel 2>/dev/null) || repo=""
if [ -z "$repo" ]; then
	echo "decisions-zero.sh: can't find the repo root for $file (git rev-parse --show-toplevel failed) — not checked" >&2
	exit 2
fi

if ! command -v just >/dev/null 2>&1; then
	echo "decisions-zero.sh: 'just' is not on PATH — $file was not checked" >&2
	exit 2
fi

# stdout and stderr are kept apart, and that is the whole of what the clean
# line is worth. The recipe says everything it has to say about §0 on stdout;
# stderr is whatever `just` said on the way to running it, and a bare `just`
# here evaluates every top-level assignment first (§146) — `hub_aligned`
# shells out to a script, and a tree that does not have it gets three lines of
# `No such file or directory` for free. Folded into one stream, those land
# inside `additionalContext`, which is the single field this hook exists to
# keep to a single line. A failure is the other way round: there stderr is the
# diagnosis, so the block carries both.
err=$(mktemp)
trap 'rm -f "$err"' EXIT
out=$(cd "$repo" && just check-where-the-work-is 2>"$err")
status=$?

if [ "$status" -ne 0 ]; then
	detail=$(cat "$err"; printf '%s\n' "$out")
	jq -n --arg reason "$(printf '`just check-where-the-work-is` found a problem after this write to %s:\n\n%s' "$file" "$detail")" \
		'{decision: "block", reason: $reason}'
	exit 0
fi

# A clean §0 says so, with its headroom, rather than saying nothing.
#
# `additionalContext` and not plain stdout: for a PostToolUse hook Claude Code
# writes stdout to the debug log and the model never sees it, which would be
# the same silence under a different name. §152 measured that silence
# directly: 148 hand `question-check` calls after that hook landed, three of
# which found anything.
jq -n --arg context "$(printf '`just check-where-the-work-is` ran on %s as part of this write: %s' "${file##*/}" "$out")" \
	'{hookSpecificOutput: {hookEventName: "PostToolUse", additionalContext: $context}}'
exit 0
