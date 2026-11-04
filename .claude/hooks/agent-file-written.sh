#!/usr/bin/env bash
# Points a write to an agent file at docs/where-a-rule-goes.md, in the same
# turn as the write (docs/decisions.md §479).
#
# A claude process reads its agent types once, when it starts, and CLAUDE.md
# when its session starts or clears (§266.6), so an edit to one leaves every
# running dispatcher on the old text. §479 has the edits that cost slots, and
# why most of their rules belonged in a tool instead; the guide says where, and
# this hook puts it in front of whoever is writing one.
#
# AGENTS.md is a link to CLAUDE.md, and the payload names whichever path the
# write was given, so both names match.
#
# Shaped as decisions-zero.sh: the answer goes in `additionalContext`, since a
# PostToolUse hook's plain stdout goes to the debug log and the model never
# sees it. It runs nothing and reads nothing but the path, so a write to any
# other file costs one jq call.
#
# `just queue add` and `writes` print the same line when a write set names one
# of these files, and check-queue-api-selftest holds the two to the same text.
set -uo pipefail

payload=$(cat)
file=$(jq -r '.tool_input.file_path // empty' <<<"$payload")

case "$file" in
CLAUDE.md | */CLAUDE.md | AGENTS.md | */AGENTS.md | .claude/agents/*.md | */.claude/agents/*.md) ;;
*) exit 0 ;;
esac

pointer='A running claude reads .claude/agents/*.md once, when its process starts, and CLAUDE.md (AGENTS.md links to it) when its session starts or clears (§266.6), so a change to one leaves every running dispatcher on the old text: docs/where-a-rule-goes.md says what belongs in them and where the rest goes, and the commit that changes one says why its rule had to go there.'

jq -n --arg context "$pointer" \
	'{hookSpecificOutput: {hookEventName: "PostToolUse", additionalContext: $context}}'
exit 0
