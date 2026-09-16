#!/usr/bin/env bash
# Says so at the top of the session when the `guardrails` plugin is not there.
#
# The two hooks that plugin carries — no in-place shell patching, and the
# context budget that asks for a handoff — are the kind that are invisible when
# they are absent. A missing patching hook looks like a session nobody happened
# to catch; a missing budget hook looks like a session that simply never crossed
# the line. On 2026-09-08 session 83dcf00a ran to 322 000 tokens of context with
# the budget hook not installed at all, and the first anyone knew of it was
# Mattias asking why the nagging had stopped. `just check-plugins` would have
# said so, and nothing says `just check-plugins` unless you run the gate.
#
# This runs from `.claude/hooks/`, which is this repo's own and cannot go
# missing with the plugin it is reporting on. That is the whole design: the
# thing that notices an absence must not be the thing that is absent.
#
# It asks `just` for the path rather than resolving one itself. `plugins_root`
# at justfile:1181 already walks CLAUDE_PLUGINS_ROOT, then the marketplace
# clone, then the checkout, and a second copy of that walk here is a second
# record beside the thing itself (docs/decisions.md §60) — and would be the copy
# that goes stale, because the justfile's is the one under test.
#
# Silent when the plugin is where the repo expects it, which is the normal case.
# Never blocks a session: it cannot tell the difference between "not installed"
# and "this machine has no `just`", so where it cannot tell, it says that
# instead of guessing.
set -uo pipefail

payload=$(cat)
cwd=$(jq -r '.cwd // empty' <<<"$payload" 2>/dev/null || true)
[ -n "$cwd" ] || cwd=$PWD
[ -d "$cwd" ] || exit 0

plugin=$(cd "$cwd" && mise exec -- just --evaluate guardrails_plugin 2>/dev/null) || {
    echo "Could not ask \`just\` where the guardrails plugin is, so this session"
    echo "does not know whether the patching hook and the context budget are"
    echo "running. Run \`just check-plugins\` before trusting either."
    exit 0
}

[ -n "$plugin" ] || exit 0
[ -d "$plugin" ] || {
    echo "The \`guardrails\` plugin is NOT installed: nothing is at $plugin."
    echo
    echo "Two hooks are therefore not running, and both are silent when absent:"
    echo "  - the one that refuses in-place shell patching of tracked files, so"
    echo "    CLAUDE.md's rule about heredocs is prose only this session;"
    echo "  - the context budget, so nothing will ask you for a handoff. In the"
    echo "    week of 2026-09-07, 35% of the token spend was calls billed above"
    echo "    200 000 tokens of context, and a call at 300 000 costs 2.6x the"
    echo "    same call at the median."
    echo
    echo "Fix it with \`just check-plugins\`, which names the install command."
    exit 0
}

exit 0
