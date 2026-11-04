#!/usr/bin/env bash
# Runs agent-file-written.sh against one write to each name it matches and one
# it does not, the same shape as check-decisions-zero.sh but with no fixture
# repo: the hook reads nothing but the path in its payload, so a path is the
# whole fixture (docs/decisions.md §479).
#
# The line is checked whole, not for a substring, for check-decisions-zero.sh's
# reason: `additionalContext` is all this hook puts in front of a model.
# `just queue add` and `writes` print the same line, and
# `check-queue-api-selftest` checks theirs against what this hook prints.
#
# Runs unconditionally, with no plugin skip: none of this needs Claude Code.
set -uo pipefail

here=$(dirname "$(readlink -f "$0")")
hook="$here/agent-file-written.sh"

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

payload() {
	printf '{"tool_input":{"file_path":"%s"}}' "$1"
}

failed=0
total=0
wrong() {
	printf '%s\n' "$1"
	failed=$((failed + 1))
}

want='CLAUDE.md, AGENTS.md and .claude/agents/*.md are read once, when a claude process starts, so a change to one leaves every running dispatcher on the old text until a new process starts: docs/where-a-rule-goes.md says what belongs in them and where the rest goes, and the commit that changes one says why its rule had to go there.'

for file in /home/someone/rcad/CLAUDE.md /home/someone/rcad/AGENTS.md /home/someone/rcad/.claude/agents/lane.md; do
	total=$((total + 1))
	out=$(payload "$file" | "$hook" 2>"$work/err")
	status=$?
	got=$(jq -r '.hookSpecificOutput.additionalContext // empty' <<<"$out" 2>/dev/null)
	event=$(jq -r '.hookSpecificOutput.hookEventName // empty' <<<"$out" 2>/dev/null)
	if [ "$status" -ne 0 ]; then
		wrong "a write to $file exited $status, wanted 0 — stderr: $(cat "$work/err")"
	elif [ "$event" != PostToolUse ]; then
		wrong "a write to $file did not answer as a PostToolUse hook: $out"
	elif [ "$got" != "$want" ]; then
		wrong "$(printf 'a write to %s did not carry exactly the pointer:\n--- got ---\n%s\n--- want ---\n%s' "$file" "$got" "$want")"
	fi
done

# The guide itself is where a rule's place is argued, not an agent file, so a
# write to it is silent.
other=/home/someone/rcad/docs/where-a-rule-goes.md
total=$((total + 1))
out=$(payload "$other" | "$hook" 2>"$work/err")
status=$?
if [ "$status" -ne 0 ]; then
	wrong "a write to $other exited $status, wanted 0 — stderr: $(cat "$work/err")"
elif [ -n "$out" ]; then
	wrong "a write to $other was not silent: $out"
fi

if [ "$failed" -gt 0 ]; then
	echo "$failed of $total checks came out wrong"
	exit 1
fi
echo "$total checks, all as expected"
