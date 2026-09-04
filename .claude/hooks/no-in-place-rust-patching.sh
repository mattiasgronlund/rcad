#!/usr/bin/env bash
# Blocks patching a .rs file through the shell: heredocs, `sed -i`, redirects.
# CLAUDE.md bans this outright; §49.5 found three of thirteen lanes doing it
# anyway, and recording it twice changed nothing. The stated risk is that a
# pattern which does not match fails quietly, which is what a diff review is
# worst at catching — so it is refused here instead.
#
# The command and its heredoc bodies are judged apart, because they are
# different things. A `git commit -F - <<'MSG'` whose message *describes* this
# rule is not patching anything, and the first version of this hook refused
# exactly that. So: the shell code is what gets searched for an in-place edit,
# and a body only matters when an interpreter is being fed one.
#
# Reading a .rs file is untouched: `sed -n '10,20p'`, grep, git diff all pass.
set -uo pipefail

command=$(jq -r '.tool_input.command // ""')

# Split the shell code from any heredoc bodies it carries.
code=$(awk '
	skip { if ($0 ~ "^[ \t]*" delim "[ \t]*$") skip = 0; next }
	{ print }
	match($0, /<<-?[ \t]*'\''?"?[A-Za-z_][A-Za-z0-9_]*/) {
		delim = substr($0, RSTART, RLENGTH)
		sub(/^<<-?[ \t]*'\''?"?/, "", delim)
		skip = 1
	}
' <<<"$command")
bodies=$(comm -13 <(sort -u <<<"$code") <(sort -u <<<"$command") 2>/dev/null || true)

deny() {
	jq -n --arg reason "$1" '{
		hookSpecificOutput: {
			hookEventName: "PreToolUse",
			permissionDecision: "deny",
			permissionDecisionReason: $reason
		}
	}'
	exit 0
}

if grep -qE '\b(sed|perl|ruby)\b[^|;]*[[:space:]]-[a-zA-Z]*i' <<<"$code" &&
	grep -q '\.rs\b' <<<"$code"; then
	deny "CLAUDE.md bans patching a .rs file in place. Use Read then Edit — Edit takes two strings and errors when a pattern does not match, where sed -i fails quietly."
fi

if grep -qE '>>?[[:space:]]*[^[:space:];|&]*\.rs\b' <<<"$code"; then
	deny "CLAUDE.md bans writing a .rs file from the shell. Use Write for a new file, Edit for a change — a redirect that goes wrong takes the file with it."
fi

if grep -qE '\b(python[0-9.]*|perl|ruby|node)\b[^|;]*<<' <<<"$code" &&
	grep -q '\.rs\b' <<<"$bodies"; then
	deny "CLAUDE.md bans heredoc patching of .rs files. In the M5 run this cost 145k tokens over 233 calls — a sixth of everything the run generated. Read then Edit instead."
fi

exit 0
