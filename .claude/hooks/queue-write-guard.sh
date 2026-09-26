#!/usr/bin/env bash
# Refuses Edit, Write and any Bash command that writes docs/queue.md, and names
# the `just queue` verb to use instead (docs/decisions.md §263).
#
# The judgement is in tooling/queue/guard.py, beside the verbs it points at, so
# the rule and the recipes that replace hand edits are one directory. This file
# only finds it: a PreToolUse hook is started from wherever the session sits.
set -uo pipefail

exec python3 "$(dirname "$(readlink -f "$0")")/../../tooling/queue/guard.py"
