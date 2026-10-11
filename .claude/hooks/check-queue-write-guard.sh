#!/usr/bin/env bash
# Runs queue-write-guard.sh against queue-write-cases.json and reports every case
# that came out wrong: each writing form refused, each reading form allowed, and
# each form that runs `pin` or `unpin` asked about, since those are Mattias's
# (docs/decisions.md §430).
#
# The cases are JSON in the shape `.claude/patching-cases.local.json` uses, with
# `tool` and `file_path` added for an Edit or a Write, and live in a file rather
# than here for the patching hook's reason: a script holding `sed -i ...
# docs/queue.md` as a literal is a command line a guard refuses.
#
# No plugin and no Claude Code needed, so it runs on CI as it runs here
# (docs/decisions.md §263).
set -uo pipefail

here=$(dirname "$(readlink -f "$0")")
hook="$here/queue-write-guard.sh"
cases="$here/queue-write-cases.json"

failed=0
total=$(jq length "$cases")

for index in $(seq 0 $((total - 1))); do
	expect=$(jq -r ".[$index].expect" "$cases")
	why=$(jq -r ".[$index].why" "$cases")
	payload=$(jq -c ".[$index] | if has(\"command\")
		then {tool_name: \"Bash\", tool_input: {command: .command}}
		else {tool_name: .tool, tool_input: {file_path: .file_path}} end" "$cases")

	out=$("$hook" <<<"$payload")
	status=$?
	if [ "$status" -ne 0 ]; then
		printf 'the guard exited %s on: %s\n' "$status" "$why"
		failed=$((failed + 1))
		continue
	fi
	got=$(jq -r '.hookSpecificOutput.permissionDecision // empty' <<<"$out")
	[ -n "$got" ] || got=allow

	if [ "$got" != "$expect" ]; then
		printf 'wanted %s, got %s: %s\n' "$expect" "$got" "$why"
		failed=$((failed + 1))
	elif [ "$got" = deny ] && ! jq -r '.hookSpecificOutput.permissionDecisionReason' <<<"$out" | grep -qF 'just queue take'; then
		printf 'a refusal that does not name the just queue verbs: %s\n' "$why"
		failed=$((failed + 1))
	elif [ "$got" = ask ] && ! jq -r '.hookSpecificOutput.permissionDecisionReason' <<<"$out" | grep -qF "Mattias's call (docs/decisions.md §430)"; then
		printf 'an ask that does not say pinning is Mattias'"'"'s call: %s\n' "$why"
		failed=$((failed + 1))
	fi
done

if [ "$failed" -gt 0 ]; then
	echo "$failed of $total queue guard cases came out wrong"
	exit 1
fi

echo "$total queue guard cases, all as expected"
