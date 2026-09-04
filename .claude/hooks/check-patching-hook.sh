#!/usr/bin/env bash
# Runs the patching hook against its cases and reports any that came out wrong.
#
# The cases live in a JSON file rather than in this script, and that is not
# tidiness: a runner holding `sed -i ... .rs` as a literal is a command line the
# hook itself refuses, so a self-test written inline cannot run. The hook
# blocked two earlier drafts of its own test before this arrangement.
set -uo pipefail

here=$(dirname "$(readlink -f "$0")")
hook="$here/no-in-place-rust-patching.sh"
cases="$here/patching-cases.json"

failed=0
total=$(jq length "$cases")

for index in $(seq 0 $((total - 1))); do
	command=$(jq -r ".[$index].command" "$cases")
	expect=$(jq -r ".[$index].expect" "$cases")
	why=$(jq -r ".[$index].why" "$cases")

	got=$(jq -n --arg c "$command" '{tool_input: {command: $c}}' |
		"$hook" | jq -r '.hookSpecificOutput.permissionDecision // empty')
	[ -n "$got" ] || got=allow

	if [ "$got" != "$expect" ]; then
		printf 'wanted %s, got %s: %s\n' "$expect" "$got" "$why"
		printf '  %s\n' "$(head -1 <<<"$command")"
		failed=$((failed + 1))
	fi
done

if [ "$failed" -gt 0 ]; then
	echo "$failed of $total cases came out wrong"
	exit 1
fi

echo "$total cases, all as expected"
