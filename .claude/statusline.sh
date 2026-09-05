#!/usr/bin/env bash
# Two lines that answer "how close am I to a handoff" and "what is waiting on
# me", without either question costing a token.
#
# This exists because the obvious way to check — `!cat docs/open-questions.md` —
# is not free. Shell mode adds its output to the session and Claude answers it:
# on 2026-09-05 that put 21 command outputs and 5,710 characters into context
# that nobody needed the model to read. The status line is terminal UI. It never
# reaches the model, so it can be re-read as often as you like.
#
# Line 1 tracks the same budget `context-budget.sh` enforces, so the handoff is
# visible on the way in rather than announced on arrival.
# Line 2 is the open-question index, and the file is the long form.
set -uo pipefail

payload=$(cat)
budget=${CLAUDE_CONTEXT_BUDGET:-300000}

get() { jq -r "$1 // empty" <<<"$payload" 2>/dev/null; }

model=$(get '.model.display_name')
used=$(get '.context_window.total_input_tokens')
root=$(get '.workspace.project_dir')
tree=$(get '.workspace.git_worktree')
[ -n "$root" ] || root=$(get '.cwd')
branch=$(git -C "$root" rev-parse --abbrev-ref HEAD 2>/dev/null)

dim=$'\033[2m'
off=$'\033[0m'
green=$'\033[32m'
yellow=$'\033[33m'
red=$'\033[31m'

# --- line 1: context against the handoff budget
line1="${model:-claude}"
[ -n "$tree" ] && line1="$line1 ${dim}·${off} $tree"
[ -n "$branch" ] && [ "$branch" != HEAD ] && line1="$line1 ${dim}·${off} $branch"

case "$used" in
'' | *[!0-9]*) used='' ;;
esac
if [ -n "$used" ]; then
	pct=$((used * 100 / budget))
	filled=$((pct / 10))
	[ "$filled" -gt 10 ] && filled=10
	[ "$filled" -lt 0 ] && filled=0
	colour=$green
	[ "$pct" -ge 70 ] && colour=$yellow
	[ "$pct" -ge 100 ] && colour=$red
	bar=""
	for i in $(seq 1 10); do
		if [ "$i" -le "$filled" ]; then bar="$bar▓"; else bar="$bar░"; fi
	done
	note=""
	[ "$pct" -ge 100 ] && note=" hand off"
	line1="$line1 ${dim}·${off} ${colour}$((used / 1000))k/$((budget / 1000))k ${bar} ${pct}%${note}${off}"
fi
printf '%s\n' "$line1"

# --- line 2: what is waiting on the user
questions="$root/docs/open-questions.md"
if [ ! -r "$questions" ]; then
	printf '%s\n' "${dim}no docs/open-questions.md${off}"
	exit 0
fi

# Only the Waiting section counts. The file explains its own format in a fenced
# example, and an example question is not a question.
mapfile -t open < <(awk '/^## Waiting/ {f = 1; next} /^## / {f = 0} f' "$questions" 2>/dev/null |
	grep -oP '^- \[ \]\s+\**\K(Q\d+)\**\s*[—-]\s*[^→]*' |
	sed -e 's/\*\*//g' -e 's/[[:space:]]\+$//')

if [ "${#open[@]}" -eq 0 ]; then
	printf '%s\n' "${green}✓${off} ${dim}nothing waiting on you${off}"
	exit 0
fi

shown=""
for q in "${open[@]:0:3}"; do
	short=${q:0:38}
	[ "${#q}" -gt 38 ] && short="$short…"
	[ -n "$shown" ] && shown="$shown ${dim}·${off} "
	shown="$shown$short"
done
more=""
[ "${#open[@]}" -gt 3 ] && more=" ${dim}+$(("${#open[@]}" - 3)) more${off}"

printf '%s\n' "${yellow}❓ ${#open[@]} open${off} ${dim}—${off} $shown$more"
